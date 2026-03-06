use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

use crate::config;
use crate::network::NetworkManager;
use crate::state::AppState;

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
                        if let Err(e) = engine.farm.auto_check_farm().await {
                            log::warn!("Farm check error: {}", e);
                        }
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
                        if let Err(e) = engine.friend.auto_check_friends().await {
                            log::warn!("Friend check error: {}", e);
                        }
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
                            if let Err(e) = engine.task.auto_claim_all().await {
                                log::warn!("Task claim error: {}", e);
                            }
                        }

                        if config.auto_claim_emails {
                            if let Err(e) = engine.email.auto_claim_all().await {
                                log::warn!("Email claim error: {}", e);
                            }
                        }

                        if config.auto_sell {
                            if let Err(e) = engine.warehouse.auto_sell_fruits().await {
                                log::warn!("Auto sell error: {}", e);
                            }
                        }
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

                        if let Err(e) = engine.vip.auto_claim_dailies().await {
                            log::warn!("Daily reward claim error: {}", e);
                        }

                        if let Err(e) = engine.mall.auto_claim_month_card().await {
                            log::warn!("Month card claim error: {}", e);
                        }

                        if let Err(e) = engine.mall.auto_claim_free_gifts().await {
                            log::warn!("Free gift claim error: {}", e);
                        }
                    }
                    _ = stop_rx.changed() => break,
                }
            }
        });
    }
}
