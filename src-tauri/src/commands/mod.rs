use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::auth::{MiniProgramLoginSession, QrLoginSession};
use crate::config::AutomationConfig;
use crate::network::NetworkManager;
use crate::services::automation::AutomationEngine;
use crate::state::{AppState, ConnectionStatus, Stats, UserState};

// ========== State types for Tauri ==========

pub struct TauriState {
    pub app_state: Arc<AppState>,
    pub network: Arc<NetworkManager>,
    pub engine: Arc<tokio::sync::Mutex<Option<Arc<AutomationEngine>>>>,
}

// ========== Response types ==========

#[derive(Serialize)]
pub struct StatusResponse {
    pub user: UserState,
    pub connection: ConnectionStatus,
    pub stats: Stats,
}

#[derive(Serialize)]
pub struct QrCodeResponse {
    pub qrsig: String,
    pub qrcode: String,
}

#[derive(Serialize)]
pub struct MpLoginCodeResponse {
    pub code: String,
    pub qrcode: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginStatusResponse {
    pub ret: String,
    pub msg: String,
    pub nickname: String,
}

// ========== Auth Commands ==========

/// Request QR code for QQ login
#[tauri::command]
pub async fn request_qr_code(
    preset: Option<String>,
) -> Result<QrCodeResponse, String> {
    let preset_key = preset.as_deref().unwrap_or("vip");
    let (qrsig, qrcode) = QrLoginSession::request_qr_code(preset_key)
        .await
        .map_err(|e| e.to_string())?;

    Ok(QrCodeResponse { qrsig, qrcode })
}

/// Check QR code scan status
#[tauri::command]
pub async fn check_qr_status(
    qrsig: String,
    preset: Option<String>,
) -> Result<LoginStatusResponse, String> {
    let preset_key = preset.as_deref().unwrap_or("vip");
    let status = QrLoginSession::check_status(&qrsig, preset_key)
        .await
        .map_err(|e| e.to_string())?;

    Ok(LoginStatusResponse {
        ret: status.ret,
        msg: status.msg,
        nickname: status.nickname,
    })
}

/// Request mini program login code
#[tauri::command]
pub async fn request_mp_login_code() -> Result<MpLoginCodeResponse, String> {
    let (code, qrcode) = MiniProgramLoginSession::request_login_code()
        .await
        .map_err(|e| e.to_string())?;

    Ok(MpLoginCodeResponse { code, qrcode })
}

/// Connect to game server with auth code and start automation
#[tauri::command]
pub async fn connect_and_login(
    code: String,
    state: State<'_, TauriState>,
) -> Result<UserState, String> {
    let network: Arc<NetworkManager> = Arc::clone(&state.network);
    let app_state: Arc<AppState> = Arc::clone(&state.app_state);

    // Save login code
    *app_state.login_code.write() = Some(code.clone());

    // Connect WebSocket
    network.connect(&code).await.map_err(|e| e.to_string())?;

    // Send login request
    network.send_login().await.map_err(|e| e.to_string())?;

    // Start heartbeat
    network.start_heartbeat();

    // Start automation engine
    let engine = Arc::new(AutomationEngine::new(
        Arc::clone(&network),
        Arc::clone(&app_state),
    ));
    *state.engine.lock().await = Some(Arc::clone(&engine));
    engine.start().await;

    let user = app_state.user.read().clone();
    Ok(user)
}

/// Disconnect from game server
#[tauri::command]
pub async fn disconnect(state: State<'_, TauriState>) -> Result<(), String> {
    // Stop automation
    if let Some(engine) = state.engine.lock().await.take() {
        engine.stop();
    }

    // Disconnect WebSocket
    state.network.disconnect().await;

    Ok(())
}

// ========== Status Commands ==========

/// Get current status
#[tauri::command]
pub async fn get_status(state: State<'_, TauriState>) -> Result<StatusResponse, String> {
    let app = &state.app_state;
    Ok(StatusResponse {
        user: app.user.read().clone(),
        connection: app.connection_status.read().clone(),
        stats: app.stats.read().clone(),
    })
}

// ========== Config Commands ==========

/// Get automation config
#[tauri::command]
pub async fn get_automation_config(
    state: State<'_, TauriState>,
) -> Result<AutomationConfig, String> {
    Ok(state.app_state.automation_config.read().clone())
}

/// Update automation config
#[tauri::command]
pub async fn set_automation_config(
    config: AutomationConfig,
    state: State<'_, TauriState>,
) -> Result<(), String> {
    *state.app_state.automation_config.write() = config;
    Ok(())
}

// ========== Farm Commands ==========

/// Get all lands info
#[tauri::command]
pub async fn get_all_lands(state: State<'_, TauriState>) -> Result<serde_json::Value, String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock
        .as_ref()
        .ok_or("Not connected")?;

    let reply = engine
        .farm()
        .get_all_lands()
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}

/// Manually harvest specific lands
#[tauri::command]
pub async fn harvest(
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<serde_json::Value, String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock.as_ref().ok_or("Not connected")?;

    let reply = engine
        .farm()
        .harvest(land_ids, 0)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}

/// Manually plant seeds
#[tauri::command]
pub async fn plant_seeds(
    seed_id: i64,
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<serde_json::Value, String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock.as_ref().ok_or("Not connected")?;

    let items = vec![crate::proto::plantpb::PlantItem {
        seed_id,
        land_ids,
        auto_slave: false,
    }];

    let reply = engine
        .farm()
        .plant(items)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}

// ========== Friend Commands ==========

/// Get friends list
#[tauri::command]
pub async fn get_friends(state: State<'_, TauriState>) -> Result<serde_json::Value, String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock.as_ref().ok_or("Not connected")?;

    let reply = engine
        .friend()
        .get_all_friends()
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}

// ========== Warehouse Commands ==========

/// Get backpack contents
#[tauri::command]
pub async fn get_bag(state: State<'_, TauriState>) -> Result<serde_json::Value, String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock.as_ref().ok_or("Not connected")?;

    let reply = engine
        .warehouse()
        .get_bag()
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}

/// Manually sell all fruits
#[tauri::command]
pub async fn sell_all_fruits(state: State<'_, TauriState>) -> Result<(), String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock.as_ref().ok_or("Not connected")?;

    engine
        .warehouse()
        .auto_sell_fruits()
        .await
        .map_err(|e| e.to_string())
}

// ========== Task Commands ==========

/// Get task info
#[tauri::command]
pub async fn get_tasks(state: State<'_, TauriState>) -> Result<serde_json::Value, String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock.as_ref().ok_or("Not connected")?;

    let reply = engine
        .task()
        .get_task_info()
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}

/// Manually claim all tasks
#[tauri::command]
pub async fn claim_all_tasks(state: State<'_, TauriState>) -> Result<(), String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock.as_ref().ok_or("Not connected")?;

    engine
        .task()
        .auto_claim_all()
        .await
        .map_err(|e| e.to_string())
}

// ========== Shop Commands ==========

/// Get shop list
#[tauri::command]
pub async fn get_shop_info(
    shop_id: i64,
    state: State<'_, TauriState>,
) -> Result<serde_json::Value, String> {
    let engine_lock = state.engine.lock().await;
    let engine = engine_lock.as_ref().ok_or("Not connected")?;

    let reply = engine
        .mall()
        .get_mall_list(shop_id as i32)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}
