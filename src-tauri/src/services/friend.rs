use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;
use prost::Message;

use crate::config::PlantPhase;
use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::{friendpb, plantpb, visitpb};
use crate::state::AppState;

// Idle probe constants
const PROBE_BATCH_SIZE: usize = 12;
const PROBE_REDUCED_BATCH_SIZE: usize = 6;
const PROBE_SKIP_THRESHOLD: usize = 24;
const PROBE_MISS_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(20 * 60);
const PROBE_HIT_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(2 * 60);

/// Persistent state for idle friend probing across scan cycles
struct IdleProbeState {
    /// Round-robin cursor into the idle candidate list
    cursor: usize,
    /// Cooldown expiry per friend gid
    cooldown_until: HashMap<i64, Instant>,
}

pub struct FriendService {
    network: Arc<NetworkManager>,
    state: Arc<AppState>,
    probe_state: Mutex<IdleProbeState>,
}

impl FriendService {
    pub fn new(network: Arc<NetworkManager>, state: Arc<AppState>) -> Self {
        Self {
            network,
            state,
            probe_state: Mutex::new(IdleProbeState {
                cursor: 0,
                cooldown_until: HashMap::new(),
            }),
        }
    }

    /// Get all friends list (uses SyncAll for QQ platform)
    pub async fn get_all_friends(&self) -> AppResult<friendpb::SyncAllReply> {
        let req = friendpb::SyncAllRequest {
            open_ids: Vec::new(),
        };
        let reply_bytes = self
            .network
            .send_request(&codec::SYNC_ALL_FRIENDS, req.encode_to_vec())
            .await?;
        Ok(friendpb::SyncAllReply::decode(reply_bytes.as_slice())?)
    }

    /// Sync friends with specific open_ids
    pub async fn sync_all_friends(
        &self,
        open_ids: Vec<String>,
    ) -> AppResult<friendpb::SyncAllReply> {
        let req = friendpb::SyncAllRequest { open_ids };
        let reply_bytes = self
            .network
            .send_request(&codec::SYNC_ALL_FRIENDS, req.encode_to_vec())
            .await?;
        Ok(friendpb::SyncAllReply::decode(reply_bytes.as_slice())?)
    }

    /// Get friend applications
    pub async fn get_applications(&self) -> AppResult<friendpb::GetApplicationsReply> {
        let req = friendpb::GetApplicationsRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::GET_APPLICATIONS, req.encode_to_vec())
            .await?;
        Ok(friendpb::GetApplicationsReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Accept friend requests
    pub async fn accept_friends(
        &self,
        friend_gids: Vec<i64>,
    ) -> AppResult<friendpb::AcceptFriendsReply> {
        let req = friendpb::AcceptFriendsRequest { friend_gids };
        let reply_bytes = self
            .network
            .send_request(&codec::ACCEPT_FRIENDS, req.encode_to_vec())
            .await?;
        Ok(friendpb::AcceptFriendsReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Reject friend requests
    pub async fn reject_friends(&self, friend_gids: Vec<i64>) -> AppResult<()> {
        let req = friendpb::RejectFriendsRequest { friend_gids };
        self.network
            .send_request(&codec::REJECT_FRIENDS, req.encode_to_vec())
            .await?;
        Ok(())
    }

    /// Enter a friend's farm
    pub async fn visit_farm(&self, host_gid: i64) -> AppResult<visitpb::EnterReply> {
        let req = visitpb::EnterRequest {
            host_gid,
            reason: 2, // ENTER_REASON_FRIEND
        };
        let reply_bytes = self
            .network
            .send_request(&codec::VISIT_ENTER, req.encode_to_vec())
            .await?;
        Ok(visitpb::EnterReply::decode(reply_bytes.as_slice())?)
    }

    /// Leave a friend's farm
    pub async fn leave_farm(&self, host_gid: i64) -> AppResult<()> {
        let req = visitpb::LeaveRequest { host_gid };
        self.network
            .send_request(&codec::VISIT_LEAVE, req.encode_to_vec())
            .await?;
        Ok(())
    }

    /// Steal crops from friend's farm
    pub async fn steal(
        &self,
        host_gid: i64,
        land_ids: Vec<i64>,
    ) -> AppResult<plantpb::HarvestReply> {
        let req = plantpb::HarvestRequest {
            land_ids,
            host_gid,
            is_all: true,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::HARVEST, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.steals += 1);
        Ok(plantpb::HarvestReply::decode(reply_bytes.as_slice())?)
    }

    /// Help water friend's farm
    pub async fn help_water(
        &self,
        host_gid: i64,
        land_ids: Vec<i64>,
    ) -> AppResult<plantpb::WaterLandReply> {
        let req = plantpb::WaterLandRequest { land_ids, host_gid };
        let reply_bytes = self
            .network
            .send_request(&codec::WATER_LAND, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.help_waters += 1);
        Ok(plantpb::WaterLandReply::decode(reply_bytes.as_slice())?)
    }

    /// Help remove weeds from friend's farm
    pub async fn help_weed_out(
        &self,
        host_gid: i64,
        land_ids: Vec<i64>,
    ) -> AppResult<plantpb::WeedOutReply> {
        let req = plantpb::WeedOutRequest { land_ids, host_gid };
        let reply_bytes = self
            .network
            .send_request(&codec::WEED_OUT, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.help_weeds += 1);
        Ok(plantpb::WeedOutReply::decode(reply_bytes.as_slice())?)
    }

    /// Help remove insects from friend's farm
    pub async fn help_insecticide(
        &self,
        host_gid: i64,
        land_ids: Vec<i64>,
    ) -> AppResult<plantpb::InsecticideReply> {
        let req = plantpb::InsecticideRequest { land_ids, host_gid };
        let reply_bytes = self
            .network
            .send_request(&codec::INSECTICIDE, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.help_insects += 1);
        Ok(plantpb::InsecticideReply::decode(reply_bytes.as_slice())?)
    }

    /// Pre-check if an operation is allowed on a friend's farm
    /// Operation IDs: 10005=除草, 10006=除虫, 10007=浇水, 10008=偷菜
    async fn can_operate(&self, host_gid: i64, operation_id: i64) -> bool {
        let req = plantpb::CheckCanOperateRequest {
            host_gid,
            operation_id,
        };
        match self
            .network
            .send_request(&codec::CHECK_CAN_OPERATE, req.encode_to_vec())
            .await
        {
            Ok(bytes) => plantpb::CheckCanOperateReply::decode(bytes.as_slice())
                .map(|r| r.can_operate)
                .unwrap_or(true),
            Err(_) => true, // fallback: don't block on pre-check failure
        }
    }

    // ========== Automation ==========

    /// Visit a single friend's farm, perform actions, return whether any action was taken.
    async fn visit_and_act(&self, friend: &friendpb::GameFriend, config: &crate::config::AutomationConfig) -> bool {
        let visit = match self.visit_farm(friend.gid).await {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Failed to visit {}: {}", friend.name, e);
                return false;
            }
        };

        let mut steal_ids = Vec::new();
        let mut dry_ids = Vec::new();
        let mut weed_ids = Vec::new();
        let mut insect_ids = Vec::new();

        for land in &visit.lands {
            if !land.unlocked {
                continue;
            }
            if let Some(plant_info) = &land.plant {
                let current_phase = plant_info
                    .phases
                    .last()
                    .map(|p| PlantPhase::from_i32(p.phase))
                    .unwrap_or(PlantPhase::Unknown);

                if current_phase == PlantPhase::Mature && plant_info.stealable {
                    steal_ids.push(land.id);
                }
                if plant_info.dry_num > 0 {
                    dry_ids.push(land.id);
                }
                if !plant_info.weed_owners.is_empty() {
                    weed_ids.push(land.id);
                }
                if !plant_info.insect_owners.is_empty() {
                    insect_ids.push(land.id);
                }
            }
        }

        let mut acted = false;

        if config.auto_steal && !steal_ids.is_empty() && self.can_operate(friend.gid, 10008).await {
            log::info!("Stealing from {} ({} lands)", friend.name, steal_ids.len());
            let _ = self.steal(friend.gid, steal_ids).await;
            acted = true;
        }

        if config.auto_help_water && !dry_ids.is_empty() && self.can_operate(friend.gid, 10007).await {
            log::info!("Watering {}'s farm ({} lands)", friend.name, dry_ids.len());
            let _ = self.help_water(friend.gid, dry_ids).await;
            acted = true;
        }

        if config.auto_help_weed && !weed_ids.is_empty() && self.can_operate(friend.gid, 10005).await {
            log::info!("Removing weeds from {}'s farm ({} lands)", friend.name, weed_ids.len());
            let _ = self.help_weed_out(friend.gid, weed_ids).await;
            acted = true;
        }

        if config.auto_help_insecticide && !insect_ids.is_empty() && self.can_operate(friend.gid, 10006).await {
            log::info!("Removing insects from {}'s farm ({} lands)", friend.name, insect_ids.len());
            let _ = self.help_insecticide(friend.gid, insect_ids).await;
            acted = true;
        }

        let _ = self.leave_farm(friend.gid).await;
        acted
    }

    /// Auto-check all friends with idle probe optimization.
    /// Priority friends (preview shows actions) are always visited.
    /// Idle friends are probed in rotating batches with cooldowns.
    pub async fn auto_check_friends(&self) -> AppResult<()> {
        let config = self.state.automation_config.read().clone();
        let friends_reply = self.get_all_friends().await?;

        let help_enabled = config.auto_help_water || config.auto_help_weed || config.auto_help_insecticide;

        // Classify friends into priority vs idle
        let mut priority_friends = Vec::new();
        let mut idle_candidates = Vec::new();

        for friend in &friends_reply.game_friends {
            if config.friend_blacklist.contains(&friend.gid) {
                continue;
            }
            if is_priority_friend(friend, &config, help_enabled) {
                priority_friends.push(friend);
            } else {
                idle_candidates.push(friend);
            }
        }

        // Sort priority friends (most actionable first)
        priority_friends.sort_by(|a, b| friend_priority(b).cmp(&friend_priority(a)));

        // Select idle probe candidates
        let probe_budget = idle_probe_budget(priority_friends.len());
        let probe_targets = self.select_probe_candidates(&idle_candidates, probe_budget);

        log::info!(
            "好友巡查: {} 优先, {} 探测 (共{}好友)",
            priority_friends.len(),
            probe_targets.len(),
            friends_reply.game_friends.len()
        );
        self.state.push_log("info", format!(
            "好友巡查: {} 优先, {} 探测",
            priority_friends.len(),
            probe_targets.len()
        ));

        // Visit priority friends
        for friend in &priority_friends {
            self.visit_and_act(friend, &config).await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // Visit probe targets and update cooldowns
        for friend in &probe_targets {
            let acted = self.visit_and_act(friend, &config).await;
            let cooldown = if acted { PROBE_HIT_COOLDOWN } else { PROBE_MISS_COOLDOWN };
            self.probe_state.lock().cooldown_until.insert(friend.gid, Instant::now() + cooldown);
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(())
    }

    /// Select idle probe candidates using round-robin cursor, skipping those in cooldown.
    fn select_probe_candidates<'a>(
        &self,
        candidates: &[&'a friendpb::GameFriend],
        budget: usize,
    ) -> Vec<&'a friendpb::GameFriend> {
        if budget == 0 || candidates.is_empty() {
            return Vec::new();
        }

        let mut state = self.probe_state.lock();
        let now = Instant::now();

        // Clean up expired cooldowns
        state.cooldown_until.retain(|_, until| *until > now);

        let len = candidates.len();
        let mut selected = Vec::with_capacity(budget);
        let mut checked = 0;
        let start = state.cursor % len;

        while selected.len() < budget && checked < len {
            let idx = (start + checked) % len;
            let friend = candidates[idx];
            checked += 1;

            if let Some(until) = state.cooldown_until.get(&friend.gid) {
                if *until > now {
                    continue;
                }
            }
            selected.push(friend);
        }

        // Advance cursor for next cycle
        state.cursor = (start + checked) % len;

        selected
    }

    /// Reset probe state (called when automation stops)
    pub fn reset_probe_state(&self) {
        let mut state = self.probe_state.lock();
        state.cursor = 0;
        state.cooldown_until.clear();
    }
}

/// Check if a friend has actionable state visible from the preview data
fn is_priority_friend(
    friend: &friendpb::GameFriend,
    config: &crate::config::AutomationConfig,
    help_enabled: bool,
) -> bool {
    let plant = match &friend.plant {
        Some(p) => p,
        None => return false,
    };

    if config.auto_steal && plant.steal_plant_num > 0 {
        return true;
    }
    if help_enabled && (plant.dry_num > 0 || plant.weed_num > 0 || plant.insect_num > 0) {
        return true;
    }
    false
}

/// Calculate friend priority for sorting (higher = more important)
fn friend_priority(friend: &friendpb::GameFriend) -> i64 {
    let plant = match &friend.plant {
        Some(p) => p,
        None => return 0,
    };

    let mut priority = 0;
    if plant.steal_plant_num > 0 {
        priority += 1000;
    }
    if plant.dry_num > 0 {
        priority += 100;
    }
    if plant.weed_num > 0 {
        priority += 50;
    }
    if plant.insect_num > 0 {
        priority += 50;
    }
    priority
}

/// Determine how many idle friends to probe based on priority count
fn idle_probe_budget(priority_count: usize) -> usize {
    if priority_count >= PROBE_SKIP_THRESHOLD {
        0
    } else if priority_count >= PROBE_REDUCED_BATCH_SIZE {
        PROBE_REDUCED_BATCH_SIZE
    } else {
        PROBE_BATCH_SIZE
    }
}
