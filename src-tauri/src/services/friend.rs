use std::sync::Arc;

use prost::Message;

use crate::config::PlantPhase;
use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::{friendpb, plantpb, visitpb};
use crate::state::AppState;

pub struct FriendService {
    network: Arc<NetworkManager>,
    state: Arc<AppState>,
}

impl FriendService {
    pub fn new(network: Arc<NetworkManager>, state: Arc<AppState>) -> Self {
        Self { network, state }
    }

    /// Get all friends list
    pub async fn get_all_friends(&self) -> AppResult<friendpb::GetAllReply> {
        let req = friendpb::GetAllRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::GET_ALL_FRIENDS, req.encode_to_vec())
            .await?;
        Ok(friendpb::GetAllReply::decode(reply_bytes.as_slice())?)
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
            is_all: false,
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

    // ========== Automation ==========

    /// Auto-check all friends and perform actions
    pub async fn auto_check_friends(&self) -> AppResult<()> {
        let config = self.state.automation_config.read().clone();
        let friends_reply = self.get_all_friends().await?;

        // Sort friends by priority: stealable > needs water > needs weed > needs insect
        let mut friends: Vec<&friendpb::GameFriend> = friends_reply
            .game_friends
            .iter()
            .filter(|f| !config.friend_blacklist.contains(&f.gid))
            .collect();

        friends.sort_by(|a, b| {
            let a_priority = friend_priority(a);
            let b_priority = friend_priority(b);
            b_priority.cmp(&a_priority)
        });

        for friend in friends {
            let plant = match &friend.plant {
                Some(p) => p,
                None => continue,
            };

            let has_action = plant.steal_plant_num > 0
                || plant.dry_num > 0
                || plant.weed_num > 0
                || plant.insect_num > 0;

            if !has_action {
                continue;
            }

            // Enter friend's farm
            let visit = match self.visit_farm(friend.gid).await {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Failed to visit {}: {}", friend.name, e);
                    continue;
                }
            };

            // Analyze lands
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

            // Perform actions
            if config.auto_steal && !steal_ids.is_empty() {
                log::info!("Stealing from {} ({} lands)", friend.name, steal_ids.len());
                let _ = self.steal(friend.gid, steal_ids).await;
            }

            if config.auto_help_water && !dry_ids.is_empty() {
                log::info!("Watering {}'s farm ({} lands)", friend.name, dry_ids.len());
                let _ = self.help_water(friend.gid, dry_ids).await;
            }

            if config.auto_help_weed && !weed_ids.is_empty() {
                log::info!(
                    "Removing weeds from {}'s farm ({} lands)",
                    friend.name,
                    weed_ids.len()
                );
                let _ = self.help_weed_out(friend.gid, weed_ids).await;
            }

            if config.auto_help_insecticide && !insect_ids.is_empty() {
                log::info!(
                    "Removing insects from {}'s farm ({} lands)",
                    friend.name,
                    insect_ids.len()
                );
                let _ = self.help_insecticide(friend.gid, insect_ids).await;
            }

            // Leave friend's farm
            let _ = self.leave_farm(friend.gid).await;

            // Small delay between friends
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(())
    }
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
