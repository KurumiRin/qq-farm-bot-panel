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

    // Notify frontend to load all page data now that we're logged in
    crate::state::emit_data_changed("farm");
    crate::state::emit_data_changed("friends");
    crate::state::emit_data_changed("inventory");
    crate::state::emit_data_changed("tasks");

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

/// Clear all logs
#[tauri::command]
pub async fn clear_logs(state: State<'_, TauriState>) -> Result<(), String> {
    state.app_state.clear_logs();
    Ok(())
}

// ========== Code Receiver Commands ==========

/// Get current login code
#[tauri::command]
pub async fn get_login_code(state: State<'_, TauriState>) -> Result<Option<String>, String> {
    Ok(state.app_state.login_code.read().clone())
}

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
    // Economics
    pub est_gold: i64,   // estimated net profit (gold)
    pub est_exp: i64,    // estimated exp
    pub seasons: i64,    // 1 or 2 season plant
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
                est_gold: 0, est_exp: 0, seasons: 0,
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

            let seed_id = plant.id - 1_000_000;
            let econ = crate::plant_econ::get_plant_econ(seed_id);
            // Log buff values for debugging (first land only)
            if lands.is_empty() {
                if let Some(buff) = &land.buff {
                    log::debug!("Land #{} (level {}) buff: yield_bonus={}, exp_bonus={}, time_reduction={}",
                        land.id, land.level, buff.plant_yield_bonus, buff.plant_exp_bonus, buff.planting_time_reduction);
                }
            }
            // Buff values are in per-ten-thousand (万分比): 1000 = 10%
            let (yield_pct, exp_pct) = land.buff.as_ref()
                .map(|b| (b.plant_yield_bonus / 100, b.plant_exp_bonus / 100))
                .unwrap_or((0, 0));
            lands.push(LandView {
                id: land.id, unlocked: true, level: land.level, max_level: land.max_level,
                status: status.into(), seed_id, seed_name: plant.name.clone(),
                phase: phase_val, phase_name: current_phase.name().into(),
                mature_in_sec, total_grow_sec: plant.grow_sec,
                fruit_num: plant.fruit_num,
                need_water, need_weed, need_insect,
                est_gold: econ.map(|e| e.net_profit_with_bonus(yield_pct)).unwrap_or(0),
                est_exp: econ.map(|e| e.total_exp_with_bonus(exp_pct)).unwrap_or(0),
                seasons: econ.map(|e| e.seasons).unwrap_or(1),
            });
        } else {
            summary.empty += 1;
            lands.push(LandView {
                id: land.id, unlocked: true, level: land.level, max_level: land.max_level,
                status: "empty".into(), seed_id: 0, seed_name: String::new(),
                phase: 0, phase_name: "空地".into(), mature_in_sec: 0, total_grow_sec: 0,
                fruit_num: 0, need_water: false, need_weed: false, need_insect: false,
                est_gold: 0, est_exp: 0, seasons: 0,
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
            Err(e) => {
                log::warn!("Plant land#{} failed: {}", land_id, e);
                // Stop immediately on connection errors to avoid long hangs
                if !engine.is_connected() {
                    break;
                }
            }
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

    // Step 2: Get bag and shop data
    let bag = engine.warehouse().get_bag().await.map_err(|e| e.to_string())?;
    let bag_items = bag.item_bag.as_ref().map(|b| &b.items[..]).unwrap_or(&[]);

    let shop = engine.shop().get_shop_info(2).await.map_err(|e| e.to_string())?;
    let mut unlocked_seeds: Vec<_> = shop.goods_list.iter()
        .filter(|g| g.unlocked)
        .collect();
    unlocked_seeds.sort_by(|a, b| b.price.cmp(&a.price));

    let gold = state.app_state.user.read().gold;

    // Step 3: Try preferred seed from bag first
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

    // Step 4: Need to buy — try preferred first, then best affordable
    let buy_and_plant = |goods_id: i64, seed_id: i64, to_buy: i64, price: i64| {
        let engine = Arc::clone(&engine);
        let land_ids = land_ids.clone();
        async move {
            log::info!("Buying goods_id={} num={} price={}", goods_id, to_buy, price);
            let buy_reply = engine.shop().buy_goods(goods_id, to_buy, price).await
                .map_err(|e| format!("购买失败: {}", e))?;
            let actual_seed_id = buy_reply.get_items.first()
                .map(|item| item.id).filter(|&id| id > 0).unwrap_or(seed_id);
            let ok = plant_one_by_one(&engine, actual_seed_id, &land_ids).await?;
            Ok::<(i64, i64), String>((ok, price * to_buy))
        }
    };

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

#[derive(Serialize)]
pub struct FriendView {
    pub gid: i64,
    pub name: String,
    pub level: i64,
    pub avatar_url: String,
    pub steal_count: i64,
    pub dry_count: i64,
    pub weed_count: i64,
    pub insect_count: i64,
}

#[derive(Serialize)]
pub struct FriendsView {
    pub friends: Vec<FriendView>,
    pub application_count: i64,
}

/// Get friends list (processed for UI)
#[tauri::command]
pub async fn get_friends(state: State<'_, TauriState>) -> Result<FriendsView, String> {
    let engine = get_engine(&state).await?;

    let reply = engine
        .friend()
        .get_all_friends()
        .await
        .map_err(|e| e.to_string())?;

    let my_gid = state.app_state.user.read().gid;
    let friends: Vec<FriendView> = reply
        .game_friends
        .iter()
        .filter(|f| f.gid != my_gid && f.name != "小小农夫" && f.remark != "小小农夫")
        .map(|f| {
            let plant = f.plant.as_ref();
            FriendView {
                gid: f.gid,
                name: if f.remark.is_empty() { f.name.clone() } else { f.remark.clone() },
                level: f.level,
                avatar_url: f.avatar_url.clone(),
                steal_count: plant.map(|p| p.steal_plant_num).unwrap_or(0),
                dry_count: plant.map(|p| p.dry_num).unwrap_or(0),
                weed_count: plant.map(|p| p.weed_num).unwrap_or(0),
                insect_count: plant.map(|p| p.insect_num).unwrap_or(0),
            }
        })
        .collect();

    Ok(FriendsView {
        friends,
        application_count: reply.application_count,
    })
}

/// Visit a friend's farm and perform all available actions
#[tauri::command]
pub async fn visit_and_act_friend(
    gid: i64,
    state: State<'_, TauriState>,
) -> Result<String, String> {
    let engine = get_engine(&state).await?;
    let my_gid = state.app_state.user.read().gid;

    let visit = engine.friend().visit_farm(gid).await.map_err(|e| e.to_string())?;

    struct StealInfo {
        land_id: i64,
        name: String,
        fruit_num: i64,
        seed_id: i64,
    }

    let mut steal_infos: Vec<StealInfo> = Vec::new();
    let mut dry_ids = Vec::new();
    let mut weed_ids = Vec::new();
    let mut insect_ids = Vec::new();

    let now = chrono::Utc::now().timestamp();
    for land in &visit.lands {
        if !land.unlocked { continue; }
        if let Some(p) = &land.plant {
            let phase = p.phases.iter().rev()
                .find(|ph| ph.begin_time > 0 && ph.begin_time <= now)
                .or_else(|| p.phases.first())
                .map(|ph| crate::config::PlantPhase::from_i32(ph.phase))
                .unwrap_or(crate::config::PlantPhase::Unknown);

            if phase == crate::config::PlantPhase::Mature && p.stealable && !p.stealers.contains(&my_gid) {
                steal_infos.push(StealInfo {
                    land_id: land.id,
                    name: p.name.clone(),
                    fruit_num: p.fruit_num,
                    seed_id: p.id,
                });
            }
            if p.dry_num > 0 { dry_ids.push(land.id); }
            if !p.weed_owners.is_empty() { weed_ids.push(land.id); }
            if !p.insect_owners.is_empty() { insect_ids.push(land.id); }
        }
    }

    let mut actions = Vec::new();

    // Pre-check if operation is allowed (server requires this before friend ops)
    // Operation IDs: 10005=除草, 10006=除虫, 10007=浇水, 10008=偷菜
    macro_rules! can_operate {
        ($op_id:expr) => {
            engine.farm().check_can_operate(gid, $op_id).await
                .map(|r| r.can_operate).unwrap_or(true)
        };
    }

    if !steal_infos.is_empty() && can_operate!(10008) {
        let steal_ids: Vec<i64> = steal_infos.iter().map(|s| s.land_id).collect();
        match engine.friend().steal(gid, steal_ids).await {
            Ok(reply) => {
                // Aggregate per crop: name -> (total_stolen, fruit_price)
                let mut crop_map: std::collections::HashMap<String, (i64, i64)> = std::collections::HashMap::new();
                for info in &steal_infos {
                    let stolen = reply.land.iter()
                        .find(|l| l.id == info.land_id)
                        .and_then(|l| l.plant.as_ref())
                        .map(|p| (info.fruit_num - p.fruit_num).max(0))
                        .unwrap_or(info.fruit_num);
                    let price = crate::plant_econ::get_plant_econ(info.seed_id)
                        .map(|e| e.fruit_price)
                        .unwrap_or(0);
                    let entry = crop_map.entry(info.name.clone()).or_insert((0, price));
                    entry.0 += stolen;
                }
                let details: Vec<String> = crop_map.iter()
                    .filter(|(_, (count, _))| *count > 0)
                    .map(|(name, (count, price))| {
                        let value = count * price;
                        if value > 0 {
                            format!("{}x{} ≈{}金", name, count, value)
                        } else {
                            format!("{}x{}", name, count)
                        }
                    })
                    .collect();
                if details.is_empty() {
                    actions.push("偷菜 0".into());
                } else {
                    actions.push(format!("偷 {}", details.join("、")));
                }
            }
            Err(e) => log::warn!("Steal failed: {}", e),
        }
    }
    if !dry_ids.is_empty() && can_operate!(10007) {
        match engine.friend().help_water(gid, dry_ids.clone()).await {
            Ok(_) => actions.push(format!("浇水 {}块", dry_ids.len())),
            Err(e) => log::warn!("Water failed: {}", e),
        }
    }
    if !weed_ids.is_empty() && can_operate!(10005) {
        match engine.friend().help_weed_out(gid, weed_ids.clone()).await {
            Ok(_) => actions.push(format!("除草 {}块", weed_ids.len())),
            Err(e) => log::warn!("Weed failed: {}", e),
        }
    }
    if !insect_ids.is_empty() && can_operate!(10006) {
        match engine.friend().help_insecticide(gid, insect_ids.clone()).await {
            Ok(_) => actions.push(format!("除虫 {}块", insect_ids.len())),
            Err(e) => log::warn!("Insecticide failed: {}", e),
        }
    }

    let _ = engine.friend().leave_farm(gid).await;

    if actions.is_empty() {
        Ok("无可执行操作".into())
    } else {
        Ok(actions.join("，"))
    }
}

// ========== Warehouse Commands ==========

#[derive(Serialize)]
pub struct BagItemView {
    pub id: i64,
    pub count: i64,
    pub name: String,
    pub category: String, // "mutant_fruit" | "seed" | "fruit" | "fertilizer" | "currency" | "other"
    pub unit_price: i64,  // sell price per unit (0 if not sellable)
    pub price_unit: String, // "金豆豆" | "点券" | "金"
}

#[derive(Serialize)]
pub struct CurrencyView {
    pub id: i64,
    pub count: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct BagView {
    pub items: Vec<BagItemView>,
    pub currencies: Vec<CurrencyView>,
    pub mutant_fruit_count: usize,
    pub seed_count: usize,
    pub fruit_count: usize,
    pub fertilizer_count: usize,
    pub other_count: usize,
    pub normal_fert_secs: i64,
    pub organic_fert_secs: i64,
}

fn categorize_item(id: i64) -> &'static str {
    match id {
        // Gold & coupons shown as currency tags
        1 | 1001 | 1002 => "currency",
        // Internal counters — hide completely
        1003..=1999 | 3001..=3999 => "hidden",
        20000..=29999 => "seed",
        40000..=49999 => "fruit",
        80001..=80099 => "fertilizer",
        // Mutant fruits (type 17): 1040000+ range, sell for 金豆豆
        1040000..=1049999 => "mutant_fruit",
        _ => "other",
    }
}

/// Get the price unit display string for an item
fn price_unit_for(id: i64) -> &'static str {
    match id {
        1040000..=1049999 => "金豆豆",
        _ => "金",
    }
}

/// Get backpack contents (processed for UI)
#[tauri::command]
pub async fn get_bag(state: State<'_, TauriState>) -> Result<BagView, String> {
    let engine = get_engine(&state).await?;

    let reply = engine
        .warehouse()
        .get_bag()
        .await
        .map_err(|e| e.to_string())?;

    let raw_items = reply.item_bag.map(|b| b.items).unwrap_or_default();

    // Sync coupon from bag (id=1002) into user state
    if let Some(coupon_item) = raw_items.iter().find(|i| i.id == 1002 && i.count > 0) {
        let mut user = state.app_state.user.write();
        if user.coupon != coupon_item.count {
            user.coupon = coupon_item.count;
            drop(user);
            state.app_state.emit_status();
        }
    }

    let mut items = Vec::new();
    let mut currencies = Vec::new();

    for item in raw_items.iter().filter(|i| i.count > 0) {
        let cat = categorize_item(item.id);
        if cat == "hidden" { continue; }
        let name = crate::item_names::get_item_name(item.id)
            .unwrap_or(cat)
            .to_string();
        if cat == "currency" {
            currencies.push(CurrencyView { id: item.id, count: item.count, name });
        } else {
            let unit_price = match cat {
                "fruit" | "seed" | "mutant_fruit" => crate::item_prices::get_item_price(item.id),
                _ => 0,
            };
            let price_unit = price_unit_for(item.id).to_string();
            items.push(BagItemView { id: item.id, count: item.count, name, category: cat.into(), unit_price, price_unit });
        }
    }

    items.sort_by(|a, b| {
        let order = |c: &str| match c { "mutant_fruit" => 0, "fruit" => 1, "seed" => 2, "fertilizer" => 3, _ => 4 };
        order(&a.category).cmp(&order(&b.category))
            .then(b.count.cmp(&a.count))
            .then(a.id.cmp(&b.id))
    });

    let mutant_fruit_count = items.iter().filter(|i| i.category == "mutant_fruit").count();
    let seed_count = items.iter().filter(|i| i.category == "seed").count();
    let fruit_count = items.iter().filter(|i| i.category == "fruit").count();
    let fertilizer_count = items.iter().filter(|i| i.category == "fertilizer").count();
    let other_count = items.iter().filter(|i| i.category == "other").count();

    // Calculate total fertilizer time in seconds
    let (mut normal_fert_secs, mut organic_fert_secs) = (0i64, 0i64);
    for item in raw_items.iter().filter(|i| i.count > 0) {
        let hours = match item.id {
            80001 => 1, 80002 => 4, 80003 => 8, 80004 => 12,
            _ => 0,
        };
        normal_fert_secs += hours * 3600 * item.count;
        let hours = match item.id {
            80011 => 1, 80012 => 4, 80013 => 8, 80014 => 12,
            _ => 0,
        };
        organic_fert_secs += hours * 3600 * item.count;
    }

    Ok(BagView { items, currencies, mutant_fruit_count, seed_count, fruit_count, fertilizer_count, other_count, normal_fert_secs, organic_fert_secs })
}

/// Manually sell all fruits
#[tauri::command]
pub async fn sell_all_fruits(state: State<'_, TauriState>) -> Result<String, String> {
    let engine = get_engine(&state).await?;

    let sold = engine
        .warehouse()
        .auto_sell_fruits()
        .await
        .map_err(|e| e.to_string())?;

    if sold == 0 {
        Ok("没有果实可出售".into())
    } else {
        Ok(format!("已出售 {} 个果实", sold))
    }
}

/// Sell a specific item by id and count
#[tauri::command]
pub async fn sell_item(item_id: i64, count: i64, state: State<'_, TauriState>) -> Result<String, String> {
    let engine = get_engine(&state).await?;

    // Get bag to find the item's uid
    let bag = engine.warehouse().get_bag().await.map_err(|e| e.to_string())?;
    let raw_items = bag.item_bag.map(|b| b.items).unwrap_or_default();
    let item = raw_items.iter().find(|i| i.id == item_id && i.count > 0)
        .ok_or_else(|| "物品不存在或数量为0".to_string())?;

    let sell_count = count.min(item.count);
    let mut sell_item = crate::proto::corepb::Item {
        id: item_id,
        count: sell_count,
        ..Default::default()
    };
    if item.uid > 0 {
        sell_item.uid = item.uid;
    }

    engine.warehouse().sell_items(vec![sell_item]).await
        .map_err(|e| format!("出售失败: {}", e))?;

    let name = crate::item_names::get_item_name(item_id).unwrap_or("物品");
    let msg = format!("已出售 {} x{}", name, sell_count);
    state.app_state.push_log("info", &msg);
    Ok(msg)
}

/// Use an item (e.g. fertilizer)
#[tauri::command]
pub async fn use_item(item_id: i64, count: i64, state: State<'_, TauriState>) -> Result<String, String> {
    let engine = get_engine(&state).await?;

    let batch_item = crate::proto::corepb::Item {
        id: item_id,
        count,
        ..Default::default()
    };
    engine.warehouse().batch_use(vec![batch_item]).await
        .map_err(|e| format!("使用失败: {}", e))?;

    let name = crate::item_names::get_item_name(item_id).unwrap_or("物品");

    // Calculate fertilizer time description
    let time_desc = match item_id {
        80001 | 80011 => "1小时",
        80002 | 80012 => "4小时",
        80003 | 80013 => "8小时",
        80004 | 80014 => "12小时",
        _ => "",
    };

    let msg = if !time_desc.is_empty() {
        format!("已使用 {} x{}，获得 {} 加速", name, count, time_desc)
    } else {
        format!("已使用 {} x{}", name, count)
    };
    state.app_state.push_log("info", &msg);
    Ok(msg)
}

// ========== Task Commands ==========

#[derive(Serialize)]
pub struct TaskView {
    pub id: i64,
    pub desc: String,
    pub progress: i64,
    pub total_progress: i64,
    pub is_claimed: bool,
    pub is_unlocked: bool,
    pub task_type: i32,
    pub share_multiple: i64,
    pub rewards: Vec<RewardView>,
}

#[derive(Serialize)]
pub struct RewardView {
    pub id: i64,
    pub count: i64,
    pub name: String,
}

#[derive(Serialize)]
pub struct ActiveRewardView {
    pub point_id: i64,
    pub need_progress: i64,
    pub status: i32,
    pub rewards: Vec<RewardView>,
}

#[derive(Serialize)]
pub struct ActiveView {
    pub active_type: i32,
    pub progress: i64,
    pub rewards: Vec<ActiveRewardView>,
}

#[derive(Serialize)]
pub struct TasksView {
    pub growth_tasks: Vec<TaskView>,
    pub daily_tasks: Vec<TaskView>,
    pub tasks: Vec<TaskView>,
    pub actives: Vec<ActiveView>,
}

fn make_reward_view(items: &[crate::proto::corepb::Item]) -> Vec<RewardView> {
    items.iter().map(|item| {
        let name = match item.id {
            1 | 1001 => "金币",
            1004 => "钻石",
            1101 => "经验",
            _ => crate::item_names::get_item_name(item.id).unwrap_or("未知物品"),
        };
        RewardView { id: item.id, count: item.count, name: name.to_string() }
    }).collect()
}

fn make_task_view(task: &crate::proto::taskpb::Task) -> TaskView {
    TaskView {
        id: task.id,
        desc: task.desc.clone(),
        progress: task.progress,
        total_progress: task.total_progress,
        is_claimed: task.is_claimed,
        is_unlocked: task.is_unlocked,
        task_type: task.task_type,
        share_multiple: task.share_multiple,
        rewards: make_reward_view(&task.rewards),
    }
}

/// Get task info (structured)
#[tauri::command]
pub async fn get_tasks(state: State<'_, TauriState>) -> Result<TasksView, String> {
    let engine = get_engine(&state).await?;

    let reply = engine
        .task()
        .get_task_info()
        .await
        .map_err(|e| e.to_string())?;

    let info = reply.task_info.unwrap_or_default();

    // Merge all tasks, deduplicate by ID, and re-categorize by task_type
    // task_type 1 = growth, 2 = daily, others = misc
    let mut seen = std::collections::HashSet::new();
    let mut growth_tasks = Vec::new();
    let mut daily_tasks = Vec::new();
    let mut tasks = Vec::new();
    for task in info.growth_tasks.iter()
        .chain(info.daily_tasks.iter())
        .chain(info.tasks.iter())
    {
        if !seen.insert(task.id) { continue; }
        let view = make_task_view(task);
        match task.task_type {
            1 => growth_tasks.push(view),
            2 => daily_tasks.push(view),
            _ => tasks.push(view),
        }
    }

    // Only include actives that have non-zero progress
    let actives: Vec<_> = info.actives.iter()
        .filter(|a| a.progress > 0)
        .map(|a| ActiveView {
            active_type: a.r#type,
            progress: a.progress,
            rewards: a.rewards.iter().map(|r| ActiveRewardView {
                point_id: r.point_id,
                need_progress: r.need_progress,
                status: r.status,
                rewards: make_reward_view(&r.rewards),
            }).collect(),
        }).collect();

    Ok(TasksView { growth_tasks, daily_tasks, tasks, actives })
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
