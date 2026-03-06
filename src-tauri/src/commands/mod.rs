use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::auth::{MiniProgramLoginSession, QrLoginSession};
use crate::config::AutomationConfig;
use crate::network::NetworkManager;
use crate::services::automation::AutomationEngine;
use crate::state::{AppState, ConnectionStatus, LogEntry, Stats, UserState};

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
    // Disconnect WebSocket first — this makes any in-flight requests fail,
    // which unblocks the engine lock if another command is holding it.
    state.network.disconnect().await;

    // Now stop automation (lock should be available since requests have failed)
    if let Some(engine) = state.engine.lock().await.take() {
        engine.stop();
    }

    // Clear user state and stats so the frontend reflects disconnected state
    state.app_state.reset();
    state.app_state.emit_status();

    state.app_state.push_log("info", "已断开连接，Code 接收服务仍在运行 (端口 7788)");

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

// ========== Log Commands ==========

/// Get logs (optionally filtered by timestamp)
#[tauri::command]
pub async fn get_logs(
    since: Option<i64>,
    state: State<'_, TauriState>,
) -> Result<Vec<LogEntry>, String> {
    Ok(state.app_state.get_logs(since))
}

// ========== Code Receiver Commands ==========

/// Restart the code receiver HTTP server (e.g. after port conflict)
#[tauri::command]
pub async fn restart_code_receiver(state: State<'_, TauriState>) -> Result<(), String> {
    crate::code_receiver::start(
        Arc::clone(&state.network),
        Arc::clone(&state.app_state),
        Arc::clone(&state.engine),
    );
    Ok(())
}

// ========== Helpers ==========

/// Clone the engine Arc and immediately release the mutex lock.
/// This prevents holding the lock across network await points.
async fn get_engine(state: &State<'_, TauriState>) -> Result<Arc<AutomationEngine>, String> {
    state
        .engine
        .lock()
        .await
        .as_ref()
        .cloned()
        .ok_or_else(|| "Not connected".to_string())
}

// ========== Farm Commands ==========

#[derive(Serialize)]
pub struct LandView {
    pub id: i64,
    pub unlocked: bool,
    pub level: i64,
    pub max_level: i64,
    pub status: String, // "locked" | "empty" | "growing" | "mature" | "dead"
    pub seed_id: i64,
    pub seed_name: String,
    pub phase: i32,
    pub phase_name: String,
    pub mature_in_sec: i64,
    pub total_grow_sec: i64,
    pub fruit_num: i64,
    pub need_water: bool,
    pub need_weed: bool,
    pub need_insect: bool,
}

#[derive(Serialize)]
pub struct FarmView {
    pub lands: Vec<LandView>,
    pub summary: FarmSummary,
}

#[derive(Serialize)]
pub struct FarmSummary {
    pub total: usize,
    pub unlocked: usize,
    pub mature: usize,
    pub growing: usize,
    pub empty: usize,
    pub dead: usize,
    pub need_water: usize,
    pub need_weed: usize,
    pub need_insect: usize,
}

/// Get all lands info (processed for UI)
#[tauri::command]
pub async fn get_all_lands(state: State<'_, TauriState>) -> Result<FarmView, String> {
    use crate::config::PlantPhase;

    let engine = get_engine(&state).await?;

    let reply = engine
        .farm()
        .get_all_lands()
        .await
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().timestamp();
    let mut lands = Vec::new();
    let mut summary = FarmSummary {
        total: reply.lands.len(),
        unlocked: 0, mature: 0, growing: 0, empty: 0, dead: 0,
        need_water: 0, need_weed: 0, need_insect: 0,
    };

    for land in &reply.lands {
        if !land.unlocked {
            lands.push(LandView {
                id: land.id, unlocked: false, level: land.level, max_level: land.max_level,
                status: "locked".into(), seed_id: 0, seed_name: String::new(),
                phase: 0, phase_name: "未开垦".into(), mature_in_sec: 0, total_grow_sec: 0,
                fruit_num: 0, need_water: false, need_weed: false, need_insect: false,
            });
            continue;
        }

        summary.unlocked += 1;

        if let Some(plant) = &land.plant {
            // Find the current phase: last phase whose begin_time <= now
            // (Server sends all future phases too, so phases.last() may be "mature" that hasn't started yet)
            let current_phase_info = plant.phases.iter().rev()
                .find(|p| p.begin_time > 0 && p.begin_time <= now)
                .or_else(|| plant.phases.first());
            let current_phase = current_phase_info
                .map(|p| PlantPhase::from_i32(p.phase))
                .unwrap_or(PlantPhase::Unknown);
            let phase_val = current_phase_info.map(|p| p.phase).unwrap_or(0);

            let need_water = plant.dry_num > 0;
            let need_weed = !plant.weed_owners.is_empty();
            let need_insect = !plant.insect_owners.is_empty();

            if need_water { summary.need_water += 1; }
            if need_weed { summary.need_weed += 1; }
            if need_insect { summary.need_insect += 1; }

            // Calculate time until mature
            let mature_in_sec = if plant.grow_sec > 0 && !matches!(current_phase, PlantPhase::Mature | PlantPhase::Dead) {
                let plant_start = plant.phases.first().map(|p| p.begin_time).unwrap_or(now);
                let mature_at = plant_start + plant.grow_sec;
                (mature_at - now).max(0)
            } else {
                0
            };

            let status = match current_phase {
                PlantPhase::Mature => { summary.mature += 1; "mature" }
                PlantPhase::Dead => { summary.dead += 1; "dead" }
                _ => { summary.growing += 1; "growing" }
            };

            lands.push(LandView {
                id: land.id, unlocked: true, level: land.level, max_level: land.max_level,
                status: status.into(), seed_id: plant.id - 1_000_000, seed_name: plant.name.clone(),
                phase: phase_val, phase_name: current_phase.name().into(),
                mature_in_sec, total_grow_sec: plant.grow_sec,
                fruit_num: plant.fruit_num,
                need_water, need_weed, need_insect,
            });
        } else {
            summary.empty += 1;
            lands.push(LandView {
                id: land.id, unlocked: true, level: land.level, max_level: land.max_level,
                status: "empty".into(), seed_id: 0, seed_name: String::new(),
                phase: 0, phase_name: "空地".into(), mature_in_sec: 0, total_grow_sec: 0,
                fruit_num: 0, need_water: false, need_weed: false, need_insect: false,
            });
        }
    }

    Ok(FarmView { lands, summary })
}

/// Manually harvest specific lands
#[tauri::command]
pub async fn harvest(
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<serde_json::Value, String> {
    let engine = get_engine(&state).await?;

    let reply = engine
        .farm()
        .harvest(land_ids, 0)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}

/// Manually water lands
#[tauri::command]
pub async fn water_lands(
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<(), String> {
    let engine = get_engine(&state).await?;
    let gid = state.app_state.user.read().gid;
    engine.farm().water(land_ids, gid).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Manually remove weeds
#[tauri::command]
pub async fn weed_out_lands(
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<(), String> {
    let engine = get_engine(&state).await?;
    let gid = state.app_state.user.read().gid;
    engine.farm().weed_out(land_ids, gid).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Manually remove insects
#[tauri::command]
pub async fn insecticide_lands(
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<(), String> {
    let engine = get_engine(&state).await?;
    let gid = state.app_state.user.read().gid;
    engine.farm().insecticide(land_ids, gid).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Manually remove dead plants
#[tauri::command]
pub async fn remove_dead_plants(
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<(), String> {
    let engine = get_engine(&state).await?;
    engine.farm().remove_plant(land_ids).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Plant seeds one land at a time (server requires individual planting)
async fn plant_one_by_one(
    engine: &AutomationEngine,
    seed_id: i64,
    land_ids: &[i64],
) -> Result<i64, String> {
    let mut success = 0i64;
    for &land_id in land_ids {
        let items = vec![crate::proto::plantpb::PlantItem {
            seed_id,
            land_ids: vec![land_id],
            auto_slave: false,
        }];
        match engine.farm().plant(items).await {
            Ok(_) => success += 1,
            Err(e) => log::warn!("Plant land#{} failed: {}", land_id, e),
        }
        if land_ids.len() > 1 {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
    if success == 0 {
        return Err("所有土地种植失败".to_string());
    }
    Ok(success)
}

/// Auto plant empty lands: sell fruits, query shop for best seed, buy if needed, then plant
#[tauri::command]
pub async fn auto_plant_empty(
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<String, String> {
    let engine = get_engine(&state).await?;
    let need = land_ids.len() as i64;
    let preferred = state.app_state.automation_config.read().preferred_seed_id;

    // Step 1: Sell fruits first to maximize available gold
    if let Err(e) = engine.warehouse().auto_sell_fruits().await {
        log::warn!("Auto sell before planting failed: {}", e);
    } else {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    // Step 2: Check bag for existing seeds
    let bag = engine.warehouse().get_bag().await.map_err(|e| e.to_string())?;
    let bag_items = bag.item_bag.as_ref().map(|b| &b.items[..]).unwrap_or(&[]);

    // Step 3: Query seed shop (shop_id=2) for real-time seed data
    let shop = engine.shop().get_shop_info(2).await.map_err(|e| e.to_string())?;
    let mut unlocked_seeds: Vec<_> = shop.goods_list.iter()
        .filter(|g| g.unlocked)
        .collect();
    // Sort by price descending so we try the most expensive (highest level) first
    unlocked_seeds.sort_by(|a, b| b.price.cmp(&a.price));

    // Try preferred seed from bag first
    if let Some(pref_id) = preferred {
        if bag_items.iter().any(|i| i.id == pref_id && i.count >= need) {
            let ok = plant_one_by_one(&engine, pref_id, &land_ids).await?;
            let msg = format!("已种植 (背包已有) x{}", ok);
            state.app_state.push_log("info", &msg);
            return Ok(msg);
        }
    }

    // Try any seed from bag (highest price first = best seed)
    for goods in &unlocked_seeds {
        if bag_items.iter().any(|i| i.id == goods.item_id && i.count >= need) {
            let ok = plant_one_by_one(&engine, goods.item_id, &land_ids).await?;
            let msg = format!("已种植 (背包已有) x{}", ok);
            state.app_state.push_log("info", &msg);
            return Ok(msg);
        }
    }

    // Step 4: Need to buy - find best affordable seed
    let gold = state.app_state.user.read().gold;

    // Helper: buy seed and plant
    let buy_and_plant = |goods_id: i64, seed_id: i64, to_buy: i64, price: i64| {
        let engine = Arc::clone(&engine);
        let land_ids = land_ids.clone();
        async move {
            log::info!("Buying goods_id={} num={} price={}", goods_id, to_buy, price);
            let buy_reply = engine.shop().buy_goods(goods_id, to_buy, price).await
                .map_err(|e| format!("购买失败: {}", e))?;
            // Use actual seed ID from buy reply if available
            let actual_seed_id = buy_reply.get_items.first()
                .map(|item| item.id)
                .filter(|&id| id > 0)
                .unwrap_or(seed_id);
            log::info!("Bought seed_id={}, planting on {} lands", actual_seed_id, land_ids.len());
            let ok = plant_one_by_one(&engine, actual_seed_id, &land_ids).await?;
            Ok::<(i64, i64), String>((ok, price * to_buy))
        }
    };

    // If preferred seed is set, try it first
    if let Some(pref_id) = preferred {
        if let Some(goods) = unlocked_seeds.iter().find(|g| g.item_id == pref_id) {
            let have = bag_items.iter().find(|i| i.id == pref_id).map(|i| i.count).unwrap_or(0);
            let to_buy = (need - have).max(0);
            if to_buy > 0 && goods.price * to_buy <= gold {
                let (ok, cost) = buy_and_plant(goods.id, pref_id, to_buy, goods.price).await?;
                let msg = format!("购买种子 x{}，花费 {} 金币。已种植 x{}", to_buy, cost, ok);
                state.app_state.push_log("info", &msg);
                return Ok(msg);
            }
        }
    }

    // Try highest-price unlocked seed that we can afford (descending by price)
    for goods in &unlocked_seeds {
        let have = bag_items.iter().find(|i| i.id == goods.item_id).map(|i| i.count).unwrap_or(0);
        let to_buy = (need - have).max(0);
        if to_buy == 0 || goods.price * to_buy <= gold {
            if to_buy > 0 {
                let (ok, cost) = buy_and_plant(goods.id, goods.item_id, to_buy, goods.price).await?;
                let msg = format!("购买种子 x{}，花费 {} 金币。已种植 x{}", to_buy, cost, ok);
                state.app_state.push_log("info", &msg);
                return Ok(msg);
            }
            // Have enough in bag already
            let ok = plant_one_by_one(&engine, goods.item_id, &land_ids).await?;
            let msg = format!("已种植 (背包已有) x{}", ok);
            state.app_state.push_log("info", &msg);
            return Ok(msg);
        }
    }

    Err(format!("金币不足，无法购买任何种子 (当前 {} 金币)", gold))
}

/// Manually plant seeds
#[tauri::command]
pub async fn plant_seeds(
    seed_id: i64,
    land_ids: Vec<i64>,
    state: State<'_, TauriState>,
) -> Result<serde_json::Value, String> {
    let engine = get_engine(&state).await?;

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
    let engine = get_engine(&state).await?;

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
    let engine = get_engine(&state).await?;

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
    let engine = get_engine(&state).await?;

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
    let engine = get_engine(&state).await?;

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
    let engine = get_engine(&state).await?;

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
    let engine = get_engine(&state).await?;

    let reply = engine
        .mall()
        .get_mall_list(shop_id as i32)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_value(&reply).map_err(|e| e.to_string())
}
