use std::sync::Arc;

use prost::Message;

use crate::config::PlantPhase;
use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::plantpb;
use crate::state::AppState;

/// Result of auto farm check
pub struct FarmCheckResult {
    pub dead_ids: Vec<i64>,
    pub empty_ids: Vec<i64>,
    /// Lands still growing after harvest (multi-season crops needing re-fertilize)
    pub growing_after_harvest: Vec<i64>,
}

pub struct FarmService {
    network: Arc<NetworkManager>,
    state: Arc<AppState>,
}

impl FarmService {
    pub fn new(network: Arc<NetworkManager>, state: Arc<AppState>) -> Self {
        Self { network, state }
    }

    /// Get all lands and operation limits
    pub async fn get_all_lands(&self) -> AppResult<plantpb::AllLandsReply> {
        let req = plantpb::AllLandsRequest { host_gid: 0 };
        let reply_bytes = self
            .network
            .send_request(&codec::ALL_LANDS, req.encode_to_vec())
            .await?;
        Ok(plantpb::AllLandsReply::decode(reply_bytes.as_slice())?)
    }

    /// Get lands for a specific host (friend's farm)
    pub async fn get_host_lands(&self, host_gid: i64) -> AppResult<plantpb::AllLandsReply> {
        let req = plantpb::AllLandsRequest { host_gid };
        let reply_bytes = self
            .network
            .send_request(&codec::ALL_LANDS, req.encode_to_vec())
            .await?;
        Ok(plantpb::AllLandsReply::decode(reply_bytes.as_slice())?)
    }

    /// Harvest mature crops
    pub async fn harvest(&self, land_ids: Vec<i64>, host_gid: i64) -> AppResult<plantpb::HarvestReply> {
        let req = plantpb::HarvestRequest {
            land_ids,
            host_gid,
            is_all: true,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::HARVEST, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.harvests += 1);
        Ok(plantpb::HarvestReply::decode(reply_bytes.as_slice())?)
    }

    /// Water dry lands
    pub async fn water(&self, land_ids: Vec<i64>, host_gid: i64) -> AppResult<plantpb::WaterLandReply> {
        let req = plantpb::WaterLandRequest { land_ids, host_gid };
        let reply_bytes = self
            .network
            .send_request(&codec::WATER_LAND, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.waters += 1);
        Ok(plantpb::WaterLandReply::decode(reply_bytes.as_slice())?)
    }

    /// Remove weeds
    pub async fn weed_out(&self, land_ids: Vec<i64>, host_gid: i64) -> AppResult<plantpb::WeedOutReply> {
        let req = plantpb::WeedOutRequest { land_ids, host_gid };
        let reply_bytes = self
            .network
            .send_request(&codec::WEED_OUT, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.weeds_removed += 1);
        Ok(plantpb::WeedOutReply::decode(reply_bytes.as_slice())?)
    }

    /// Remove insects
    pub async fn insecticide(&self, land_ids: Vec<i64>, host_gid: i64) -> AppResult<plantpb::InsecticideReply> {
        let req = plantpb::InsecticideRequest { land_ids, host_gid };
        let reply_bytes = self
            .network
            .send_request(&codec::INSECTICIDE, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.insects_removed += 1);
        Ok(plantpb::InsecticideReply::decode(reply_bytes.as_slice())?)
    }

    /// Plant seeds on lands
    pub async fn plant(&self, items: Vec<plantpb::PlantItem>) -> AppResult<plantpb::PlantReply> {
        let req = plantpb::PlantRequest {
            land_and_seed: Default::default(),
            items,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::PLANT, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.plants += 1);
        Ok(plantpb::PlantReply::decode(reply_bytes.as_slice())?)
    }

    /// Remove plants from lands
    pub async fn remove_plant(&self, land_ids: Vec<i64>) -> AppResult<plantpb::RemovePlantReply> {
        let req = plantpb::RemovePlantRequest { land_ids };
        let reply_bytes = self
            .network
            .send_request(&codec::REMOVE_PLANT, req.encode_to_vec())
            .await?;
        Ok(plantpb::RemovePlantReply::decode(reply_bytes.as_slice())?)
    }

    /// Apply fertilizer to lands
    pub async fn fertilize(
        &self,
        land_ids: Vec<i64>,
        fertilizer_id: i64,
    ) -> AppResult<plantpb::FertilizeReply> {
        let req = plantpb::FertilizeRequest {
            land_ids,
            fertilizer_id,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::FERTILIZE, req.encode_to_vec())
            .await?;
        Ok(plantpb::FertilizeReply::decode(reply_bytes.as_slice())?)
    }

    /// Put insects on friend's land (sabotage)
    pub async fn put_insects(
        &self,
        host_gid: i64,
        land_ids: Vec<i64>,
    ) -> AppResult<plantpb::PutInsectsReply> {
        let req = plantpb::PutInsectsRequest { host_gid, land_ids };
        let reply_bytes = self
            .network
            .send_request(&codec::PUT_INSECTS, req.encode_to_vec())
            .await?;
        Ok(plantpb::PutInsectsReply::decode(reply_bytes.as_slice())?)
    }

    /// Put weeds on friend's land (sabotage)
    pub async fn put_weeds(
        &self,
        host_gid: i64,
        land_ids: Vec<i64>,
    ) -> AppResult<plantpb::PutWeedsReply> {
        let req = plantpb::PutWeedsRequest { host_gid, land_ids };
        let reply_bytes = self
            .network
            .send_request(&codec::PUT_WEEDS, req.encode_to_vec())
            .await?;
        Ok(plantpb::PutWeedsReply::decode(reply_bytes.as_slice())?)
    }

    /// Upgrade a land plot
    pub async fn upgrade_land(&self, land_id: i64) -> AppResult<plantpb::UpgradeLandReply> {
        let req = plantpb::UpgradeLandRequest { land_id };
        let reply_bytes = self
            .network
            .send_request(&codec::UPGRADE_LAND, req.encode_to_vec())
            .await?;
        Ok(plantpb::UpgradeLandReply::decode(reply_bytes.as_slice())?)
    }

    /// Unlock a land plot
    pub async fn unlock_land(
        &self,
        land_id: i64,
        do_shared: bool,
    ) -> AppResult<plantpb::UnlockLandReply> {
        let req = plantpb::UnlockLandRequest { land_id, do_shared };
        let reply_bytes = self
            .network
            .send_request(&codec::UNLOCK_LAND, req.encode_to_vec())
            .await?;
        Ok(plantpb::UnlockLandReply::decode(reply_bytes.as_slice())?)
    }

    /// Check if an operation can be performed
    pub async fn check_can_operate(
        &self,
        host_gid: i64,
        operation_id: i64,
    ) -> AppResult<plantpb::CheckCanOperateReply> {
        let req = plantpb::CheckCanOperateRequest {
            host_gid,
            operation_id,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::CHECK_CAN_OPERATE, req.encode_to_vec())
            .await?;
        Ok(plantpb::CheckCanOperateReply::decode(reply_bytes.as_slice())?)
    }

    // ========== Automation helpers ==========

    /// Check own farm and perform automated actions
    pub async fn auto_check_farm(&self) -> AppResult<FarmCheckResult> {
        let config = self.state.automation_config.read().clone();
        let reply = self.get_all_lands().await?;

        let mut mature_ids = Vec::new();
        let mut dry_ids = Vec::new();
        let mut weed_ids = Vec::new();
        let mut insect_ids = Vec::new();
        let mut dead_ids = Vec::new();
        let mut empty_ids = Vec::new();
        let mut growing_after_harvest = Vec::new();

        let now = self.state.server_now_sec();

        for land in &reply.lands {
            if !land.unlocked {
                continue;
            }

            // Skip slave lands (occupied by 2x2 master land)
            if land.master_land_id > 0 && land.master_land_id != land.id {
                continue;
            }

            if let Some(plant) = &land.plant {
                let current_phase = get_current_phase(plant, now);

                match current_phase {
                    PlantPhase::Mature => mature_ids.push(land.id),
                    PlantPhase::Dead => dead_ids.push(land.id),
                    _ => {
                        if plant.dry_num > 0 {
                            dry_ids.push(land.id);
                        }
                        if !plant.weed_owners.is_empty() {
                            weed_ids.push(land.id);
                        }
                        if !plant.insect_owners.is_empty() {
                            insect_ids.push(land.id);
                        }
                    }
                }
            } else {
                empty_ids.push(land.id);
            }
        }

        // Harvest mature crops, then classify post-harvest land states
        if config.auto_harvest && !mature_ids.is_empty() {
            log::info!("Harvesting {} lands", mature_ids.len());
            match self.harvest(mature_ids.clone(), 0).await {
                Ok(harvest_reply) => {
                    for land in &harvest_reply.land {
                        if let Some(plant) = &land.plant {
                            let phase = get_current_phase(plant, now);
                            match phase {
                                PlantPhase::Dead => dead_ids.push(land.id),
                                PlantPhase::Mature | PlantPhase::Unknown => {
                                    empty_ids.push(land.id);
                                }
                                _ => {
                                    // Still growing = multi-season crop
                                    growing_after_harvest.push(land.id);
                                }
                            }
                        } else {
                            empty_ids.push(land.id);
                        }
                    }
                    let replied: std::collections::HashSet<i64> =
                        harvest_reply.land.iter().map(|l| l.id).collect();
                    for id in &mature_ids {
                        if !replied.contains(id) {
                            empty_ids.push(*id);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Harvest failed: {}", e);
                }
            }
        }

        // Water dry crops
        if config.auto_water && !dry_ids.is_empty() {
            log::info!("Watering {} lands", dry_ids.len());
            let _ = self.water(dry_ids, 0).await;
        }

        // Remove weeds
        if config.auto_weed && !weed_ids.is_empty() {
            log::info!("Removing weeds from {} lands", weed_ids.len());
            let _ = self.weed_out(weed_ids, 0).await;
        }

        // Remove insects
        if config.auto_insecticide && !insect_ids.is_empty() {
            log::info!("Removing insects from {} lands", insect_ids.len());
            let _ = self.insecticide(insect_ids, 0).await;
        }

        if config.auto_plant {
            Ok(FarmCheckResult { dead_ids, empty_ids, growing_after_harvest })
        } else {
            Ok(FarmCheckResult { dead_ids: Vec::new(), empty_ids: Vec::new(), growing_after_harvest })
        }
    }
}

/// Get the current plant phase based on server time
fn get_current_phase(plant: &plantpb::PlantInfo, now: i64) -> PlantPhase {
    plant
        .phases
        .iter()
        .rev()
        .find(|p| p.begin_time > 0 && p.begin_time <= now)
        .or_else(|| plant.phases.first())
        .map(|p| PlantPhase::from_i32(p.phase))
        .unwrap_or(PlantPhase::Unknown)
}

