use std::sync::Arc;
use std::time::Instant;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

use crate::network::NetworkManager;
use crate::services::automation::AutomationEngine;
use crate::state::AppState;

const PORT: u16 = 7788;
const DEDUP_WINDOW_SECS: u64 = 60;
const MAX_BIND_RETRIES: u32 = 5;
const RETRY_DELAY_SECS: u64 = 3;

/// Serialize all connect attempts so concurrent requests don't race
static CONNECT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Start a tiny HTTP server on port 7788 that accepts POST /code with JSON { "code": "..." }
/// When a code is received, it connects to the game server and starts automation.
pub fn start(
    network: Arc<NetworkManager>,
    app_state: Arc<AppState>,
    engine: Arc<tokio::sync::Mutex<Option<Arc<AutomationEngine>>>>,
) {
    let last_code: Arc<std::sync::Mutex<Option<(String, Instant)>>> = Arc::new(std::sync::Mutex::new(None));

    tauri::async_runtime::spawn(async move {
        let listener = match bind_with_retry(&app_state).await {
            Some(l) => l,
            None => return,
        };

        loop {
            let (stream, addr) = match listener.accept().await {
                Ok(v) => {
                    log::info!("[CodeReceiver] Accepted connection from {}", v.1);
                    v
                }
                Err(e) => {
                    log::error!("[CodeReceiver] Accept error: {}", e);
                    continue;
                }
            };

            let network = Arc::clone(&network);
            let app_state = Arc::clone(&app_state);
            let engine = Arc::clone(&engine);
            let last_code = Arc::clone(&last_code);

            tauri::async_runtime::spawn(async move {
                log::info!("[CodeReceiver] Handling request from {}", addr);
                let (reader, mut writer) = stream.into_split();
                let mut buf_reader = BufReader::new(reader);
                let mut headers = String::new();
                let mut content_length: usize = 0;

                // Read HTTP headers
                loop {
                    let mut line = String::new();
                    match buf_reader.read_line(&mut line).await {
                        Ok(0) | Err(_) => return,
                        _ => {}
                    }
                    if line.trim().is_empty() {
                        break;
                    }
                    if line.to_lowercase().starts_with("content-length:") {
                        content_length = line
                            .split(':')
                            .nth(1)
                            .and_then(|v| v.trim().parse().ok())
                            .unwrap_or(0);
                    }
                    headers.push_str(&line);
                }

                // Read body
                let mut body = vec![0u8; content_length];
                if content_length > 0 {
                    if buf_reader.read_exact(&mut body).await.is_err() {
                        return;
                    }
                }

                let body_str = String::from_utf8_lossy(&body);

                // Extract code from JSON body
                let code = extract_code(&body_str);

                if let Some(code) = code {
                    // Acquire connect lock to prevent concurrent connection attempts
                    let _connect_guard = CONNECT_LOCK.lock().await;

                    // Dedup: only skip if already logged in AND same code was used recently
                    let is_logged_in = app_state.is_logged_in();
                    if is_logged_in {
                        let is_dup = {
                            let guard = last_code.lock().unwrap();
                            matches!(&*guard, Some((prev, ts)) if prev == &code && ts.elapsed().as_secs() < DEDUP_WINDOW_SECS)
                        };
                        if is_dup {
                            log::debug!("Skipping duplicate code (already logged in) from {}", addr);
                            app_state.push_log("info", format!("已登录，跳过重复 code (来自 {})", addr));
                            let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{\"ok\":true,\"skipped\":true}";
                            let _ = writer.write_all(resp.as_bytes()).await;
                            return;
                        }
                    }

                    // Record code IMMEDIATELY for dedup (before connecting)
                    *last_code.lock().unwrap() = Some((code.clone(), Instant::now()));

                    log::info!("Received code from {}: {}...", addr, &code[..code.len().min(30)]);
                    app_state.push_log(
                        "info",
                        format!("收到来自 {} 的 code: {}...", addr, &code[..code.len().min(20)]),
                    );

                    // Disconnect existing connection if any
                    log::info!("[CodeReceiver] Disconnecting old connection...");
                    if let Some(old_engine) = engine.lock().await.take() {
                        old_engine.stop();
                    }
                    network.disconnect().await;
                    app_state.reset();
                    log::info!("[CodeReceiver] Old connection cleaned up, connecting with new code...");

                    // Connect with new code
                    match do_connect(&network, &app_state, &engine, &code).await {
                        Ok(_) => {
                            let user = app_state.user.read().clone();
                            app_state.push_log(
                                "info",
                                format!("连接成功! 用户: {} Lv.{}", user.name, user.level),
                            );
                            let resp = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{{\"ok\":true,\"user\":\"{}\"}}",
                                user.name
                            );
                            let _ = writer.write_all(resp.as_bytes()).await;
                        }
                        Err(e) => {
                            app_state.push_log("error", format!("连接失败: {}", e));
                            let resp = format!(
                                "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{{\"ok\":false,\"error\":\"{}\"}}",
                                e.replace('"', "'")
                            );
                            let _ = writer.write_all(resp.as_bytes()).await;
                        }
                    }
                } else {
                    // Handle CORS preflight or bad requests
                    if headers.contains("OPTIONS") {
                        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST\r\nAccess-Control-Allow-Headers: Content-Type\r\nConnection: close\r\n\r\n";
                        let _ = writer.write_all(resp.as_bytes()).await;
                    } else {
                        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{\"ok\":false,\"error\":\"missing code\"}";
                        let _ = writer.write_all(resp.as_bytes()).await;
                    }
                }
            });
        }
    });
}

async fn bind_with_retry(app_state: &Arc<AppState>) -> Option<TcpListener> {
    for attempt in 1..=MAX_BIND_RETRIES {
        match TcpListener::bind(format!("0.0.0.0:{}", PORT)).await {
            Ok(l) => {
                log::info!("Code receiver listening on 0.0.0.0:{}", PORT);
                app_state.push_log("info", format!("Code 接收服务已启动，端口 {}", PORT));
                return Some(l);
            }
            Err(e) => {
                if attempt < MAX_BIND_RETRIES {
                    log::warn!(
                        "Failed to bind port {} (attempt {}/{}): {}, retrying in {}s...",
                        PORT, attempt, MAX_BIND_RETRIES, e, RETRY_DELAY_SECS
                    );
                    app_state.push_log(
                        "warn",
                        format!(
                            "端口 {} 被占用 ({}/{}), {}秒后重试...",
                            PORT, attempt, MAX_BIND_RETRIES, RETRY_DELAY_SECS
                        ),
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(RETRY_DELAY_SECS)).await;
                } else {
                    log::error!("Failed to bind port {} after {} attempts: {}", PORT, MAX_BIND_RETRIES, e);
                    app_state.push_log(
                        "error",
                        format!(
                            "端口 {} 启动失败（已重试 {} 次）: {}。请手动释放端口后点击「重启接收服务」",
                            PORT, MAX_BIND_RETRIES, e
                        ),
                    );
                }
            }
        }
    }
    None
}

async fn do_connect(
    network: &Arc<NetworkManager>,
    app_state: &Arc<AppState>,
    engine_slot: &Arc<tokio::sync::Mutex<Option<Arc<AutomationEngine>>>>,
    code: &str,
) -> Result<(), String> {
    // Abort any in-progress auto-reconnect and close old connection
    network.disconnect().await;

    *app_state.login_code.write() = Some(code.to_string());

    network.connect(code).await.map_err(|e| e.to_string())?;
    network.send_login().await.map_err(|e| e.to_string())?;
    network.start_heartbeat();

    let new_engine = Arc::new(AutomationEngine::new(
        Arc::clone(network),
        Arc::clone(app_state),
    ));
    *engine_slot.lock().await = Some(Arc::clone(&new_engine));
    new_engine.start().await;

    Ok(())
}

fn extract_code(body: &str) -> Option<String> {
    // Simple JSON extraction: {"code": "xxx"}
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
        return v["code"].as_str().map(|s| s.to_string());
    }
    // Fallback: treat whole body as code if it looks like one
    let trimmed = body.trim();
    if !trimmed.is_empty() && !trimmed.contains(' ') && trimmed.len() > 10 {
        return Some(trimmed.to_string());
    }
    None
}
