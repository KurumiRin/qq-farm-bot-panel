use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use rand::Rng;
use tokio::sync::watch;

use crate::config::{item_ids, FertilizerStrategy};
use crate::network::{NetworkManager, NetworkEvent};
use crate::state::{self, AppState};

use super::email::EmailService;
use super::farm::FarmService;
use super::friend::FriendService;
use super::mall::MallService;
use super::shop::ShopService;
use super::task::TaskService;
use super::vip::VipService;
use super::warehouse::WarehouseService;

/// Orchestrates all automation loops
pub struct AutomationEngine {
    network: Arc<NetworkManager>,
    farm: FarmService,
    friend: FriendService,
    warehouse: WarehouseService,
    task: TaskService,
    email: EmailService,
    mall: MallService,
    shop: ShopService,
    vip: VipService,
    state: Arc<AppState>,
    stop_tx: watch::Sender<bool>,
    stop_rx: watch::Receiver<bool>,
}

impl AutomationEngine {
    pub fn new(network: Arc<NetworkManager>, state: Arc<AppState>) -> Self {
        let (stop_tx, stop_rx) = watch::channel(false);
        Self {
            network: Arc::clone(&network),
            farm: FarmService::new(Arc::clone(&network), Arc::clone(&state)),
            friend: FriendService::new(Arc::clone(&network), Arc::clone(&state)),
            warehouse: WarehouseService::new(Arc::clone(&network), Arc::clone(&state)),
            task: TaskService::new(Arc::clone(&network), Arc::clone(&state)),
            email: EmailService::new(Arc::clone(&network), Arc::clone(&state)),
            mall: MallService::new(Arc::clone(&network)),
            shop: ShopService::new(Arc::clone(&network)),
            vip: VipService::new(Arc::clone(&network)),
            state,
            stop_tx,
            stop_rx,
        }
    }

    /// Stop all automation loops
    pub fn stop(&self) {
        let _ = self.stop_tx.send(true);
    }

    /// Get services for direct access
    pub fn farm(&self) -> &FarmService {
        &self.farm
    }

    pub fn friend(&self) -> &FriendService {
        &self.friend
    }

    pub fn warehouse(&self) -> &WarehouseService {
        &self.warehouse
    }

    pub fn task(&self) -> &TaskService {
        &self.task
    }

    pub fn email(&self) -> &EmailService {
        &self.email
    }

    pub fn mall(&self) -> &MallService {
        &self.mall
    }

    pub fn shop(&self) -> &ShopService {
        &self.shop
    }

    pub fn vip(&self) -> &VipService {
        &self.vip
    }

    pub fn is_connected(&self) -> bool {
        self.network.is_connected()
    }

    /// Smart planting: remove dead plants, find best seed, buy if needed, plant
    pub async fn auto_plant_empty_lands(&self, dead_ids: &[i64], empty_ids: &[i64]) {
        // Remove dead plants first
        if !dead_ids.is_empty() {
            log::info!("Removing {} dead plants before planting", dead_ids.len());
            if let Err(e) = self.farm.remove_plant(dead_ids.to_vec()).await {
                log::warn!("Remove dead plants failed: {}", e);
            } else {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }

        let mut all_ids: Vec<i64> = Vec::with_capacity(dead_ids.len() + empty_ids.len());
        all_ids.extend_from_slice(dead_ids);
        all_ids.extend_from_slice(empty_ids);
        if all_ids.is_empty() {
            return;
        }
        let config = self.state.automation_config.read().clone();
        let need = all_ids.len() as i64;

        // Sell fruits before planting to maximize gold for buying seeds
        if config.auto_sell {
            let _ = self.warehouse.auto_sell_fruits().await;
        }

        // Get bag and shop data
        let bag = match self.warehouse.get_bag().await {
            Ok(b) => b,
            Err(e) => { log::warn!("Get bag failed: {}", e); return; }
        };
        let bag_items = bag.item_bag.as_ref().map(|b| &b.items[..]).unwrap_or(&[]);

        let shop = match self.shop.get_shop_info(2).await {
            Ok(s) => s,
            Err(e) => { log::warn!("Get shop failed: {}", e); return; }
        };
        let mut unlocked_seeds: Vec<_> = shop.goods_list.iter()
            .filter(|g| g.unlocked)
            .collect();
        unlocked_seeds.sort_by(|a, b| b.price.cmp(&a.price));

        let gold = self.state.user.read().gold;
        let bag_count = |seed_id: i64| -> i64 {
            bag_items.iter().find(|i| i.id == seed_id).map(|i| i.count).unwrap_or(0)
        };

        // Try preferred seed from bag
        if let Some(pref_id) = config.preferred_seed_id {
            if bag_count(pref_id) >= need {
                let ok = self.plant_one_by_one(pref_id, &all_ids).await;
                log::info!("Auto-planted preferred seed (from bag) x{}", ok);
                return;
            }
        }

        // Try any seed from bag (highest price first)
        for goods in &unlocked_seeds {
            if bag_count(goods.item_id) >= need {
                let ok = self.plant_one_by_one(goods.item_id, &all_ids).await;
                log::info!("Auto-planted (from bag) x{}", ok);
                return;
            }
        }

        // Need to buy — try preferred first, then best affordable
        let candidates: Vec<_> = if let Some(pref_id) = config.preferred_seed_id {
            let mut v: Vec<_> = unlocked_seeds.iter()
                .filter(|g| g.item_id == pref_id).copied().collect();
            v.extend(unlocked_seeds.iter().filter(|g| g.item_id != pref_id).copied());
            v
        } else {
            unlocked_seeds
        };

        for goods in &candidates {
            let have = bag_count(goods.item_id);
            let to_buy = (need - have).max(0);
            if to_buy > 0 && goods.price * to_buy <= gold {
                match self.shop.buy_goods(goods.id, to_buy, goods.price).await {
                    Ok(buy_reply) => {
                        let seed_id = buy_reply.get_items.first()
                            .map(|item| item.id).filter(|&id| id > 0)
                            .unwrap_or(goods.item_id);
                        let ok = self.plant_one_by_one(seed_id, &all_ids).await;
                        log::info!("Auto-planted (bought {} seeds, cost {}) x{}", to_buy, goods.price * to_buy, ok);
                        return;
                    }
                    Err(e) => { log::warn!("Buy seed failed: {}", e); continue; }
                }
            }
        }

        log::warn!("Auto-plant failed: no affordable seeds (gold={})", gold);
    }

    /// Plant one land at a time with delay.
    /// Tracks occupied slave lands from 2x2 crops to avoid planting on them.
    async fn plant_one_by_one(&self, seed_id: i64, land_ids: &[i64]) -> usize {
        let mut ok = 0;
        let mut occupied: HashSet<i64> = HashSet::new();

        for &land_id in land_ids {
            // Skip if this land was already occupied by a previous 2x2 plant
            if occupied.contains(&land_id) {
                continue;
            }

            let items = vec![crate::proto::plantpb::PlantItem {
                seed_id,
                land_ids: vec![land_id],
                auto_slave: false,
            }];
            match self.farm.plant(items).await {
                Ok(reply) => {
                    ok += 1;
                    // Check if this plant occupied slave lands (2x2 crop)
                    for land in &reply.land {
                        if land.id == land_id {
                            for &slave_id in &land.slave_land_ids {
                                if slave_id > 0 {
                                    occupied.insert(slave_id);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Plant land#{} failed: {}", land_id, e);
                }
            }
            if land_ids.len() > 1 {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
        ok
    }

    /// Apply fertilizer to multi-season crops after harvest
    async fn fertilize_multi_season(&self, land_ids: &[i64]) {
        let config = self.state.automation_config.read().clone();
        let strategy = &config.fertilizer_strategy;
        if *strategy == FertilizerStrategy::None {
            return;
        }

        log::info!("多季补肥: {} 块地", land_ids.len());
        self.state.push_log("info", format!("多季补肥: {} 块地", land_ids.len()));

        if *strategy == FertilizerStrategy::Normal || *strategy == FertilizerStrategy::Both {
            match self.farm.fertilize(land_ids.to_vec(), item_ids::NORMAL_FERTILIZER).await {
                Ok(_) => log::info!("多季补肥: 普通化肥施肥完成"),
                Err(e) => log::warn!("多季补肥: 普通化肥失败: {}", e),
            }
        }

        if *strategy == FertilizerStrategy::Organic || *strategy == FertilizerStrategy::Both {
            match self.farm.fertilize(land_ids.to_vec(), item_ids::ORGANIC_FERTILIZER).await {
                Ok(_) => log::info!("多季补肥: 有机化肥施肥完成"),
                Err(e) => log::warn!("多季补肥: 有机化肥失败: {}", e),
            }
        }
    }

    /// Handle farm check result: sell, plant, fertilize
    async fn handle_farm_check_result(&self, result: super::farm::FarmCheckResult) {
        let config = self.state.automation_config.read().clone();

        // Sell fruits after harvest
        if config.auto_sell {
            if let Err(e) = self.warehouse.auto_sell_fruits().await {
                log::warn!("Auto sell after farm check: {}", e);
            }
        }

        // Plant empty lands
        let total = result.dead_ids.len() + result.empty_ids.len();
        if total > 0 {
            self.state.push_log("info", format!("发现 {} 块空地，自动种植中...", total));
            self.auto_plant_empty_lands(&result.dead_ids, &result.empty_ids).await;
        }

        // Multi-season fertilizer
        if !result.growing_after_harvest.is_empty() {
            self.fertilize_multi_season(&result.growing_after_harvest).await;
        }
    }

    /// Start all automation loops
    pub async fn start(self: Arc<Self>) {
        log::info!("Starting automation engine");
        self.state.push_log("info", "自动化引擎已启动");

        // Farm check loop (randomized interval from config)
        let engine = Arc::clone(&self);
        let mut stop_rx = self.stop_rx.clone();
        tokio::spawn(async move {
            loop {
                let intervals = engine.state.automation_config.read().intervals.clone();
                let delay = random_interval_secs(intervals.farm_min, intervals.farm_max);
                engine.state.push_log("info", format!("下次巡田: {}秒后", delay));
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(delay)) => {
                        if !engine.state.is_logged_in() { continue; }
                        engine.state.push_log("info", "开始巡田检查");
                        match engine.farm.auto_check_farm().await {
                            Ok(result) => {
                                engine.state.push_log("info", "巡田检查完成");
                                engine.handle_farm_check_result(result).await;
                            }
                            Err(e) => {
                                log::warn!("Farm check error: {}", e);
                                engine.state.push_log("error", format!("巡田出错: {}", e));
                            }
                        }
                        state::emit_data_changed("farm");
                        state::emit_data_changed("inventory");
                    }
                    _ = stop_rx.changed() => break,
                }
            }
        });

        // Friend check loop (randomized interval from config)
        let engine = Arc::clone(&self);
        let mut stop_rx = self.stop_rx.clone();
        tokio::spawn(async move {
            loop {
                let intervals = engine.state.automation_config.read().intervals.clone();
                let delay = random_interval_secs(intervals.friend_min, intervals.friend_max);
                engine.state.push_log("info", format!("下次好友巡查: {}秒后", delay));
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(delay)) => {
                        if !engine.state.is_logged_in() { continue; }
                        engine.state.push_log("info", "开始好友巡查");
                        match engine.friend.auto_check_friends().await {
                            Ok(_) => engine.state.push_log("info", "好友巡查完成"),
                            Err(e) => {
                                log::warn!("Friend check error: {}", e);
                                engine.state.push_log("error", format!("好友巡查出错: {}", e));
                            }
                        }
                        state::emit_data_changed("friends");
                    }
                    _ = stop_rx.changed() => break,
                }
            }
        });

        // Task/email/daily rewards check loop (every 5 minutes)
        let engine = Arc::clone(&self);
        let mut stop_rx = self.stop_rx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !engine.state.is_logged_in() { continue; }
                        let config = engine.state.automation_config.read().clone();

                        if config.auto_claim_tasks {
                            match engine.task.auto_claim_all().await {
                                Ok(_) => engine.state.push_log("info", "自动领取任务完成"),
                                Err(e) => {
                                    log::warn!("Task claim error: {}", e);
                                    engine.state.push_log("error", format!("领取任务出错: {}", e));
                                }
                            }
                        }

                        if config.auto_claim_emails {
                            match engine.email.auto_claim_all().await {
                                Ok(_) => engine.state.push_log("info", "自动领取邮件完成"),
                                Err(e) => {
                                    log::warn!("Email claim error: {}", e);
                                    engine.state.push_log("error", format!("领取邮件出错: {}", e));
                                }
                            }
                        }

                        state::emit_data_changed("tasks");
                    }
                    _ = stop_rx.changed() => break,
                }
            }
        });

        // Daily rewards check loop (every hour)
        let engine = Arc::clone(&self);
        let mut stop_rx = self.stop_rx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !engine.state.is_logged_in() { continue; }

                        match engine.vip.auto_claim_dailies().await {
                            Ok(_) => engine.state.push_log("info", "领取每日奖励完成"),
                            Err(e) => {
                                log::warn!("Daily reward claim error: {}", e);
                                engine.state.push_log("error", format!("领取每日奖励出错: {}", e));
                            }
                        }

                        match engine.mall.auto_claim_month_card().await {
                            Ok(_) => engine.state.push_log("info", "领取月卡奖励完成"),
                            Err(e) => {
                                log::warn!("Month card claim error: {}", e);
                                engine.state.push_log("error", format!("领取月卡出错: {}", e));
                            }
                        }

                        match engine.mall.auto_claim_free_gifts().await {
                            Ok(_) => engine.state.push_log("info", "领取免费礼包完成"),
                            Err(e) => {
                                log::warn!("Free gift claim error: {}", e);
                                engine.state.push_log("error", format!("领取免费礼包出错: {}", e));
                            }
                        }
                    }
                    _ = stop_rx.changed() => break,
                }
            }
        });

        // Server push notification event loop
        if let Some(mut event_rx) = self.network.take_event_rx().await {
            let engine = Arc::clone(&self);
            let mut stop_rx = self.stop_rx.clone();
            tokio::spawn(async move {
                // Debounce: minimum interval between push-triggered farm checks
                let mut last_farm_check = tokio::time::Instant::now() - Duration::from_secs(10);
                let debounce = Duration::from_millis(500);

                loop {
                    tokio::select! {
                        event = event_rx.recv() => {
                            let Some(event) = event else { break };
                            match event {
                                NetworkEvent::LandsChanged { lands } => {
                                    let count = lands.len();
                                    log::info!("[Push] LandsNotify: {} lands changed", count);
                                    engine.state.push_log("info", format!("收到农田变化推送 ({}块地)", count));
                                    state::emit_data_changed("farm");
                                    let now = tokio::time::Instant::now();
                                    if now.duration_since(last_farm_check) >= debounce && engine.state.is_logged_in() {
                                        last_farm_check = now;
                                        match engine.farm.auto_check_farm().await {
                                            Ok(result) => {
                                                engine.handle_farm_check_result(result).await;
                                            }
                                            Err(e) => log::warn!("Push-triggered farm check failed: {}", e),
                                        }
                                        state::emit_data_changed("farm");
                                        state::emit_data_changed("inventory");
                                    }
                                }
                                NetworkEvent::Kickout { reason } => {
                                    engine.state.push_log("error", format!("被踢出游戏: {}", reason));
                                    engine.network.disconnect().await;
                                }
                                NetworkEvent::BasicNotify { .. } => {
                                    // State already updated in handle_notify
                                }
                                NetworkEvent::FriendApplicationReceived => {
                                    engine.state.push_log("info", "收到好友申请");
                                    state::emit_data_changed("friends");
                                }
                                NetworkEvent::FriendAdded { names } => {
                                    engine.state.push_log("info", format!("新好友: {}", names.join(", ")));
                                    state::emit_data_changed("friends");
                                }
                                NetworkEvent::TaskInfoNotify => {
                                    state::emit_data_changed("tasks");
                                }
                                NetworkEvent::GoodsUnlockNotify => {
                                    engine.state.push_log("info", "新商品已解锁");
                                    state::emit_data_changed("inventory");
                                }
                            }
                        }
                        _ = stop_rx.changed() => break,
                    }
                }
                log::info!("[Push] Event loop stopped");
            });
        }
    }
}

/// Returns a random duration in seconds between min and max (inclusive).
/// Ensures min <= max and at least 1 second.
fn random_interval_secs(min: u64, max: u64) -> u64 {
    let min = min.max(1);
    let max = max.max(min);
    if min == max {
        return min;
    }
    rand::rng().random_range(min..=max)
}
