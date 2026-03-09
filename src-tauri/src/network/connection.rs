use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use prost::Message;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::tungstenite;

use crate::config::{self, item_ids};
use crate::error::{AppError, AppResult};
use crate::proto::{gatepb, userpb, plantpb, friendpb};
use crate::state::{AppState, ConnectionStatus};

use super::codec::{self, ServiceRoute};
use super::crypto::CryptoWasm;

type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    tungstenite::Message,
>;

type PendingCallback = oneshot::Sender<Result<(Vec<u8>, gatepb::Meta), AppError>>;

/// Event types emitted by the network layer
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    Kickout { reason: String },
    LandsChanged { lands: Vec<plantpb::LandInfo> },
    BasicNotify { level: i64, gold: i64, exp: i64 },
    FriendApplicationReceived,
    FriendAdded { names: Vec<String> },
    TaskInfoNotify,
    GoodsUnlockNotify,
}

pub struct NetworkManager {
    state: Arc<AppState>,
    client_seq: AtomicI64,
    server_seq: AtomicI64,
    pending: Arc<DashMap<i64, PendingCallback>>,
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    connected: AtomicBool,
    event_tx: parking_lot::Mutex<mpsc::UnboundedSender<NetworkEvent>>,
    event_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<NetworkEvent>>>>,
    /// Incremented on disconnect to cancel in-flight auto-reconnect
    reconnect_generation: AtomicI64,
    /// WASM crypto for encrypting outgoing message bodies
    crypto: Option<CryptoWasm>,
}

impl NetworkManager {
    pub fn new(state: Arc<AppState>) -> Arc<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let crypto = match CryptoWasm::new() {
            Ok(c) => {
                log::info!("[Crypto] WASM encryption initialized");
                Some(c)
            }
            Err(e) => {
                log::warn!("[Crypto] WASM init failed, sending unencrypted: {}", e);
                None
            }
        };
        Arc::new(Self {
            state,
            client_seq: AtomicI64::new(1),
            server_seq: AtomicI64::new(0),
            pending: Arc::new(DashMap::new()),
            ws_sink: Arc::new(Mutex::new(None)),
            connected: AtomicBool::new(false),
            event_tx: parking_lot::Mutex::new(event_tx),
            event_rx: Arc::new(Mutex::new(Some(event_rx))),
            reconnect_generation: AtomicI64::new(0),
            crypto,
        })
    }

    pub fn event_sender(&self) -> mpsc::UnboundedSender<NetworkEvent> {
        self.event_tx.lock().clone()
    }

    /// Take the event receiver (can only be called once).
    /// Used by AutomationEngine to consume push notification events.
    pub async fn take_event_rx(&self) -> Option<mpsc::UnboundedReceiver<NetworkEvent>> {
        self.event_rx.lock().await.take()
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Reject all pending requests with an explicit error instead of silently dropping.
    fn reject_all_pending(&self, reason: &str) {
        let keys: Vec<i64> = self.pending.iter().map(|e| *e.key()).collect();
        for seq in keys {
            if let Some((_, tx)) = self.pending.remove(&seq) {
                let _ = tx.send(Err(AppError::Other(reason.into())));
            }
        }
    }

    /// Establish WebSocket connection (without starting read loop)
    pub(crate) async fn connect_ws(&self, code: &str) -> AppResult<futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >> {
        self.state.set_connection_status(ConnectionStatus::Connecting);

        let url = format!(
            "{}?platform={}&os={}&ver={}&code={}&openID=",
            config::SERVER_URL,
            config::PLATFORM,
            config::OS,
            config::CLIENT_VERSION,
            code,
        );

        let request = tungstenite::http::Request::builder()
            .uri(&url)
            .header("User-Agent", config::USER_AGENT)
            .header("Origin", config::ORIGIN)
            .header("Sec-WebSocket-Key", tungstenite::handshake::client::generate_key())
            .header("Sec-WebSocket-Version", "13")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Host", "gate-obt.nqf.qq.com")
            .body(())
            .map_err(|e| AppError::WebSocket(e.to_string()))?;

        let (ws_stream, _response) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| AppError::WebSocket(e.to_string()))?;

        let (sink, stream) = ws_stream.split();
        *self.ws_sink.lock().await = Some(sink);
        self.connected.store(true, Ordering::Relaxed);

        self.state.set_connection_status(ConnectionStatus::Connected);
        log::info!("[WS] Connected to server");

        Ok(stream)
    }

    /// Connect to the game server and start processing messages
    pub async fn connect(self: &Arc<Self>, code: &str) -> AppResult<()> {
        let stream = self.connect_ws(code).await?;
        self.spawn_read_loop(stream);
        Ok(())
    }

    /// Spawn the message processing loop as a new task
    pub(crate) fn spawn_read_loop(
        self: &Arc<Self>,
        stream: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    ) {
        let self_clone = Arc::clone(self);
        tokio::spawn(async move {
            self_clone.read_loop(stream).await;
        });
    }

    /// Send login request after connection
    pub async fn send_login(self: &Arc<Self>) -> AppResult<()> {
        self.state.set_connection_status(ConnectionStatus::LoggingIn);

        let req = userpb::LoginRequest {
            sharer_id: 0,
            sharer_open_id: String::new(),
            device_info: Some(userpb::DeviceInfo {
                client_version: config::CLIENT_VERSION.to_string(),
                sys_software: "iOS 26.2.1".to_string(),
                network: "wifi".to_string(),
                memory: 7672,
                device_id: "iPhone X<iPhone18,3>".to_string(),
                ..Default::default()
            }),
            share_cfg_id: 0,
            scene_id: "1256".to_string(),
            report_data: Some(userpb::ReportData {
                minigame_channel: "other".to_string(),
                minigame_platid: 2,
                ..Default::default()
            }),
        };

        let reply_bytes = self.send_request(&codec::LOGIN, req.encode_to_vec()).await?;
        let reply = userpb::LoginReply::decode(reply_bytes.as_slice())?;

        if let Some(basic) = &reply.basic {
            log::info!(
                "Login reply basic: name={} lv={} gid={} gold={} exp={}",
                basic.name, basic.level, basic.gid, basic.gold, basic.exp
            );

            self.state.update_user_from_login(
                basic.gid,
                basic.name.clone(),
                basic.level,
                basic.gold,
                basic.exp,
                basic.avatar_url.clone(),
            );

            if reply.time_now_millis > 0 {
                self.state.sync_server_time(reply.time_now_millis);
            }

            self.state.set_connection_status(ConnectionStatus::LoggedIn);
        }

        Ok(())
    }

    /// Send a protobuf request and wait for response
    pub async fn send_request(
        &self,
        route: &ServiceRoute,
        body: Vec<u8>,
    ) -> AppResult<Vec<u8>> {
        if !self.connected.load(Ordering::Relaxed) {
            return Err(AppError::NotConnected);
        }

        let seq = self.client_seq.fetch_add(1, Ordering::Relaxed);
        let server_seq = self.server_seq.load(Ordering::Relaxed);

        // Encrypt body before encoding into gate message
        let encrypted_body = if let Some(crypto) = &self.crypto {
            match crypto.encrypt_buffer(&body) {
                Ok(enc) => enc,
                Err(e) => {
                    log::warn!("[Crypto] Encrypt failed, sending raw: {}", e);
                    body
                }
            }
        } else {
            body
        };

        let frame = codec::encode_gate_message(route, encrypted_body, seq, server_seq);

        let (tx, rx) = oneshot::channel();
        self.pending.insert(seq, tx);

        // Send the frame
        {
            let mut sink_guard = self.ws_sink.lock().await;
            if let Some(sink) = sink_guard.as_mut() {
                sink.send(tungstenite::Message::Binary(frame.into()))
                    .await
                    .map_err(|e| {
                        self.pending.remove(&seq);
                        AppError::WebSocket(e.to_string())
                    })?;
            } else {
                self.pending.remove(&seq);
                return Err(AppError::NotConnected);
            }
        }

        // Wait for response with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(config::REQUEST_TIMEOUT_MS),
            rx,
        )
        .await
        .map_err(|_| {
            self.pending.remove(&seq);
            AppError::Timeout(format!("{}.{}", route.service, route.method))
        })?
        .map_err(|_| AppError::Other("Response channel closed".into()))?;

        match result {
            Ok((body, _meta)) => Ok(body),
            Err(e) => Err(e),
        }
    }

    /// Send heartbeat
    pub async fn send_heartbeat(&self) -> AppResult<()> {
        let gid = self.state.user_gid();
        let req = userpb::HeartbeatRequest {
            gid,
            client_version: config::CLIENT_VERSION.to_string(),
        };

        let reply_bytes = self
            .send_request(&codec::HEARTBEAT, req.encode_to_vec())
            .await?;
        let reply = userpb::HeartbeatReply::decode(reply_bytes.as_slice())?;

        if reply.server_time > 0 {
            self.state.sync_server_time(reply.server_time);
        }

        Ok(())
    }

    /// Start heartbeat loop with dead-connection detection.
    /// If heartbeat fails consecutively, clear pending requests and force reconnect.
    pub fn start_heartbeat(self: &Arc<Self>) {
        let self_clone = Arc::clone(self);
        let generation = self.reconnect_generation.load(Ordering::Relaxed);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_millis(config::HEARTBEAT_INTERVAL_MS));
            let mut miss_count: u32 = 0;
            loop {
                interval.tick().await;
                if !self_clone.is_connected() {
                    break;
                }
                // Abort if generation changed (new connection replaced us)
                if self_clone.reconnect_generation.load(Ordering::Relaxed) != generation {
                    break;
                }
                match self_clone.send_heartbeat().await {
                    Ok(_) => { miss_count = 0; }
                    Err(e) => {
                        miss_count += 1;
                        log::warn!("Heartbeat failed ({}/{}): {}", miss_count, config::HEARTBEAT_MAX_MISS, e);
                        if miss_count >= config::HEARTBEAT_MAX_MISS {
                            log::error!("Heartbeat missed {} times, connection is dead. Forcing reconnect.", miss_count);
                            self_clone.state.push_log("warn", format!(
                                "心跳连续 {} 次无响应，连接可能已断开，正在清理...", miss_count
                            ));
                            // Reject all pending requests so they fail immediately
                            self_clone.reject_all_pending("心跳超时，连接已断开");
                            // Force close the sink to break the read_loop,
                            // which will trigger auto-reconnect
                            self_clone.connected.store(false, Ordering::Relaxed);
                            if let Some(mut sink) = self_clone.ws_sink.lock().await.take() {
                                let _ = sink.close().await;
                            }
                            break;
                        }
                    }
                }
            }
        });
    }

    /// Disconnect and clean up
    pub async fn disconnect(&self) {
        self.reconnect_generation.fetch_add(1, Ordering::Relaxed);
        self.connected.store(false, Ordering::Relaxed);

        // Close WebSocket
        if let Some(mut sink) = self.ws_sink.lock().await.take() {
            let _ = sink.close().await;
        }

        // Reject all pending requests with explicit error
        self.reject_all_pending("连接已断开");

        // Reset sequence numbers for fresh session
        self.client_seq.store(1, Ordering::Relaxed);
        self.server_seq.store(0, Ordering::Relaxed);

        // Recreate event channel so the next AutomationEngine can take the receiver
        let (new_tx, new_rx) = mpsc::unbounded_channel();
        *self.event_tx.lock() = new_tx;
        *self.event_rx.lock().await = Some(new_rx);

        self.state.set_connection_status(ConnectionStatus::Disconnected);
        log::info!("[WS] Disconnected");
    }

    /// WebSocket read loop - processes incoming messages
    async fn read_loop(
        self: Arc<Self>,
        mut stream: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    ) {
        // Capture generation at start so we can detect if a new connection replaced us
        let my_generation = self.reconnect_generation.load(Ordering::Relaxed);

        let mut close_reason = "unknown";
        while let Some(msg_result) = stream.next().await {
            match msg_result {
                Ok(tungstenite::Message::Binary(data)) => {
                    self.handle_message(&data);
                }
                Ok(tungstenite::Message::Close(_)) => {
                    close_reason = "server closed";
                    log::info!("[WS] Connection closed by server");
                    break;
                }
                Err(e) => {
                    close_reason = "read error";
                    log::error!("[WS] Read error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // If generation changed, a new connection has already been established.
        // Do NOT touch shared state — the new connection owns it now.
        let current_gen = self.reconnect_generation.load(Ordering::Relaxed);
        if current_gen != my_generation {
            log::info!("[WS] Old read_loop exiting (generation {} → {}), skipping cleanup", my_generation, current_gen);
            return;
        }

        self.connected.store(false, Ordering::Relaxed);
        self.reject_all_pending("连接已断开");
        *self.ws_sink.lock().await = None;

        // Auto-reconnect if we have a login code (not manually disconnected)
        let code = self.state.login_code.read().clone();
        if let Some(code) = code {
            let gen = self.reconnect_generation.load(Ordering::Relaxed);
            self.state.set_connection_status(ConnectionStatus::Reconnecting);
            self.state.push_log("warn", format!("连接断开 ({})，准备自动重连...", close_reason));
            auto_reconnect(self, code, gen).await;
        } else {
            self.state.set_connection_status(ConnectionStatus::Disconnected);
            self.state.push_log("warn", format!("连接断开 ({})", close_reason));
        }
    }

    /// Handle an incoming WebSocket message
    fn handle_message(&self, data: &[u8]) {
        let msg = match codec::decode_gate_message(data) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to decode gate message: {}", e);
                return;
            }
        };

        let meta = match &msg.meta {
            Some(m) => m,
            None => return,
        };

        // Update server_seq
        if meta.server_seq > 0 {
            self.server_seq
                .fetch_max(meta.server_seq, Ordering::Relaxed);
        }

        match meta.message_type {
            // Notify (3)
            3 => self.handle_notify(&msg),
            // Response (2)
            2 => {
                let error_code = meta.error_code;
                let client_seq = meta.client_seq;

                if let Some((_, tx)) = self.pending.remove(&client_seq) {
                    if error_code != 0 {
                        let _ = tx.send(Err(AppError::Server {
                            code: error_code,
                            message: meta.error_message.clone(),
                        }));
                    } else {
                        let _ = tx.send(Ok((msg.body, meta.clone())));
                    }
                } else if error_code != 0 {
                    log::warn!(
                        "Unmatched error response: {}.{} code={}",
                        meta.service_name,
                        meta.method_name,
                        error_code
                    );
                }
            }
            _ => {}
        }
    }

    /// Handle server push notifications
    fn handle_notify(&self, msg: &gatepb::Message) {
        if msg.body.is_empty() {
            return;
        }

        let event = match gatepb::EventMessage::decode(msg.body.as_slice()) {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Failed to decode EventMessage: {}", e);
                return;
            }
        };

        let msg_type = &event.message_type;

        if msg_type.contains("Kickout") {
            if let Ok(notify) = crate::proto::gatepb::KickoutNotify::decode(event.body.as_slice()) {
                let reason = if notify.reason_message.is_empty() {
                    "未知".to_string()
                } else {
                    notify.reason_message
                };
                log::warn!("Kicked out: {}", reason);
                self.state.push_log("error", format!("被踢出: {}", reason));
                // Clear login code so we don't auto-reconnect after kick
                *self.state.login_code.write() = None;
                let _ = self.event_tx.lock().send(NetworkEvent::Kickout { reason });
            }
        } else if msg_type.contains("LandsNotify") {
            if let Ok(notify) = plantpb::LandsNotify::decode(event.body.as_slice()) {
                let host_gid = notify.host_gid;
                let my_gid = self.state.user_gid();
                if !notify.lands.is_empty() && (host_gid == my_gid || host_gid == 0) {
                    let _ = self.event_tx.lock().send(NetworkEvent::LandsChanged {
                        lands: notify.lands,
                    });
                }
            }
        } else if msg_type.contains("ItemNotify") {
            if let Ok(notify) = crate::proto::itempb::ItemNotify::decode(event.body.as_slice()) {
                let mut changed = false;
                for item_chg in &notify.items {
                    if let Some(item) = &item_chg.item {
                        let id = item.id;
                        let count = item.count;
                        let delta = item_chg.delta;
                        log::debug!("[ItemNotify] id={} count={} delta={}", id, count, delta);
                        let mut user = self.state.user.write();
                        match id {
                            item_ids::EXP_ITEM => {
                                if count > 0 { user.exp = count; changed = true; }
                                else if delta != 0 { user.exp = (user.exp + delta).max(0); changed = true; }
                            }
                            item_ids::GOLD | item_ids::GOLD_ALT => {
                                if count > 0 { user.gold = count; changed = true; }
                                else if delta != 0 { user.gold = (user.gold + delta).max(0); changed = true; }
                            }
                            item_ids::COUPON => {
                                if count > 0 { user.coupon = count; changed = true; }
                                else if delta != 0 { user.coupon = (user.coupon + delta).max(0); changed = true; }
                            }
                            _ => {}
                        }
                    }
                }
                if changed {
                    // emit_status is sufficient — gold/exp/coupon are shown via
                    // UserState, no need to trigger a full bag refetch.
                    self.state.emit_status();
                }
            }
        } else if msg_type.contains("BasicNotify") {
            if let Ok(notify) = userpb::BasicNotify::decode(event.body.as_slice()) {
                if let Some(basic) = &notify.basic {
                    log::debug!("[BasicNotify] level={} gold={} exp={}", basic.level, basic.gold, basic.exp);
                    {
                        let mut user = self.state.user.write();
                        if basic.level > 0 { user.level = basic.level; }
                        if basic.gold > 0 { user.gold = basic.gold; }
                        if basic.exp > 0 { user.exp = basic.exp; }
                        let _ = self.event_tx.lock().send(NetworkEvent::BasicNotify {
                            level: user.level,
                            gold: user.gold,
                            exp: user.exp,
                        });
                    }
                    self.state.emit_status();
                }
            }
        } else if msg_type.contains("FriendApplicationReceived") {
            let _ = self.event_tx.lock().send(NetworkEvent::FriendApplicationReceived);
        } else if msg_type.contains("FriendAdded") {
            if let Ok(notify) = friendpb::FriendAddedNotify::decode(event.body.as_slice()) {
                let names: Vec<String> = notify
                    .friends
                    .iter()
                    .map(|f| {
                        if !f.name.is_empty() { f.name.clone() }
                        else { format!("GID:{}", f.gid) }
                    })
                    .collect();
                log::info!("New friends: {}", names.join(", "));
                let _ = self.event_tx.lock().send(NetworkEvent::FriendAdded { names });
            }
        } else if msg_type.contains("GoodsUnlock") {
            let _ = self.event_tx.lock().send(NetworkEvent::GoodsUnlockNotify);
        } else if msg_type.contains("TaskInfo") {
            let _ = self.event_tx.lock().send(NetworkEvent::TaskInfoNotify);
        }
    }
}

/// Auto reconnect with exponential backoff (free function to break async type cycle).
async fn auto_reconnect(nm: Arc<NetworkManager>, code: String, generation: i64) {
    let delays = [3u64, 5, 10, 15, 30, 60];
    for (attempt, &delay) in delays.iter().enumerate() {
        nm.state.push_log("info", format!("第 {} 次重连，{}秒后尝试...", attempt + 1, delay));
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;

        // Abort if generation changed (new connect/disconnect happened)
        if nm.reconnect_generation.load(Ordering::Relaxed) != generation {
            log::info!("[WS] Auto-reconnect cancelled (new connection initiated)");
            return;
        }

        // Abort if user manually disconnected during wait
        if nm.state.login_code.read().is_none() {
            nm.state.push_log("info", "已手动断开，取消重连");
            nm.state.set_connection_status(ConnectionStatus::Disconnected);
            return;
        }

        match nm.connect_ws(&code).await {
            Ok(stream) => {
                match nm.send_login().await {
                    Ok(_) => {
                        nm.start_heartbeat();
                        nm.state.push_log("info", "重连成功");
                        nm.spawn_read_loop(stream);
                        return;
                    }
                    Err(e) => {
                        log::warn!("Reconnect login failed: {}", e);
                        nm.state.push_log("error", format!("重连登录失败: {}", e));
                        nm.connected.store(false, std::sync::atomic::Ordering::Relaxed);
                        if let Some(mut sink) = nm.ws_sink.lock().await.take() {
                            let _ = sink.close().await;
                        }
                        nm.reject_all_pending("重连登录失败");
                    }
                }
            }
            Err(e) => {
                log::warn!("Reconnect attempt {} failed: {}", attempt + 1, e);
                nm.state.push_log("error", format!("重连失败: {}", e));
            }
        }
    }

    nm.state.set_connection_status(ConnectionStatus::Disconnected);
    nm.state.push_log("error", "多次重连失败，已放弃。请手动重新连接");
}
