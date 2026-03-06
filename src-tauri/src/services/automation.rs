use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

use crate::config;
use crate::network::NetworkManager;
use crate::state::{self, AppState};

use super::email::EmailService;
use super::farm::FarmService;
use super::friend::FriendService;
use super::mall::MallService;
use super::task::TaskService;
use super::vip::VipService;
use super::warehouse::WarehouseService;

/// Orchestrates all automation loops
pub struct AutomationEngine {
    farm: FarmService,
    friend: FriendService,
    warehouse: WarehouseService,
    task: TaskService,
    email: EmailService,
    mall: MallService,
    vip: VipService,
    state: Arc<AppState>,
    stop_tx: watch::Sender<bool>,
    stop_rx: watch::Receiver<bool>,
}

impl AutomationEngine {
    pub fn new(network: Arc<NetworkManager>, state: Arc<AppState>) -> Self {
        let (stop_tx, stop_rx) = watch::channel(false);
        Self {
            farm: FarmService::new(Arc::clone(&network), Arc::clone(&state)),
            friend: FriendService::new(Arc::clone(&network), Arc::clone(&state)),
            warehouse: WarehouseService::new(Arc::clone(&network), Arc::clone(&state)),
            task: TaskService::new(Arc::clone(&network), Arc::clone(&state)),
            email: EmailService::new(Arc::clone(&network), Arc::clone(&state)),
            mall: MallService::new(Arc::clone(&network)),
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

    pub fn vip(&self) -> &VipService {
        &self.vip
    }

    /// Start all automation loops
    pub async fn start(self: Arc<Self>) {
        log::info!("Starting automation engine");
        self.state.push_log("info", "自动化引擎已启动");

        // Farm check loop
        let engine = Arc::clone(&self);
        let mut stop_rx = self.stop_rx.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_millis(config::FARM_CHECK_INTERVAL_MS));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if !engine.state.is_logged_in() { continue; }
                        engine.state.push_log("info", "开始巡田检查");
                        match engine.farm.auto_check_farm().await {
                            Ok(_) => engine.state.push_log("info", "巡田检查完成"),
                            Err(e) => {
                                log::warn!("Farm check error: {}", e);
                                engine.state.push_log("error", format!("巡田出错: {}", e));
                            }
                        }
                        state::emit_data_changed("farm");
                    }
                    _ = stop_rx.changed() => break,
                }
            }
        });

        // Friend check loop
        let engine = Arc::clone(&self);
        let mut stop_rx = self.stop_rx.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_millis(config::FRIEND_CHECK_INTERVAL_MS));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
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

                        if config.auto_sell {
                            match engine.warehouse.auto_sell_fruits().await {
                                Ok(_) => engine.state.push_log("info", "自动出售果实完成"),
                                Err(e) => {
                                    log::warn!("Auto sell error: {}", e);
                                    engine.state.push_log("error", format!("自动出售出错: {}", e));
                                }
                            }
                        }
                        state::emit_data_changed("tasks");
                        state::emit_data_changed("inventory");
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
    }
}
