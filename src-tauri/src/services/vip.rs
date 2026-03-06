use std::sync::Arc;

use prost::Message;

use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::{illustratedpb, qqvippb, redpacketpb, sharepb};

pub struct VipService {
    network: Arc<NetworkManager>,
}

impl VipService {
    pub fn new(network: Arc<NetworkManager>) -> Self {
        Self { network }
    }

    // ========== QQ VIP ==========

    /// Get QQ VIP daily gift status
    pub async fn get_daily_gift_status(&self) -> AppResult<qqvippb::GetDailyGiftStatusReply> {
        let req = qqvippb::GetDailyGiftStatusRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::GET_DAILY_GIFT_STATUS, req.encode_to_vec())
            .await?;
        Ok(qqvippb::GetDailyGiftStatusReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Claim QQ VIP daily gift
    pub async fn claim_daily_gift(&self) -> AppResult<qqvippb::ClaimDailyGiftReply> {
        let req = qqvippb::ClaimDailyGiftRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::CLAIM_DAILY_GIFT, req.encode_to_vec())
            .await?;
        Ok(qqvippb::ClaimDailyGiftReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    // ========== Red Packet ==========

    /// Get today's red packet status
    pub async fn get_red_packet_status(
        &self,
    ) -> AppResult<redpacketpb::GetTodayClaimStatusReply> {
        let req = redpacketpb::GetTodayClaimStatusRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::GET_TODAY_CLAIM_STATUS, req.encode_to_vec())
            .await?;
        Ok(redpacketpb::GetTodayClaimStatusReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Claim a red packet
    pub async fn claim_red_packet(
        &self,
        id: i32,
    ) -> AppResult<redpacketpb::ClaimRedPacketReply> {
        let req = redpacketpb::ClaimRedPacketRequest { id };
        let reply_bytes = self
            .network
            .send_request(&codec::CLAIM_RED_PACKET, req.encode_to_vec())
            .await?;
        Ok(redpacketpb::ClaimRedPacketReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    // ========== Illustrated/Compendium ==========

    /// Get illustrated list
    pub async fn get_illustrated_list(
        &self,
    ) -> AppResult<illustratedpb::GetIllustratedListV2Reply> {
        let req = illustratedpb::GetIllustratedListV2Request {
            refresh: false,
            full: true,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::GET_ILLUSTRATED_LIST, req.encode_to_vec())
            .await?;
        Ok(illustratedpb::GetIllustratedListV2Reply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Claim all illustrated rewards
    pub async fn claim_all_illustrated_rewards(
        &self,
    ) -> AppResult<illustratedpb::ClaimAllRewardsV2Reply> {
        let req = illustratedpb::ClaimAllRewardsV2Request {
            only_claimable: true,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::CLAIM_ALL_REWARDS_V2, req.encode_to_vec())
            .await?;
        Ok(illustratedpb::ClaimAllRewardsV2Reply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    // ========== Share ==========

    /// Check if can share
    pub async fn check_can_share(&self) -> AppResult<sharepb::CheckCanShareReply> {
        let req = sharepb::CheckCanShareRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::CHECK_CAN_SHARE, req.encode_to_vec())
            .await?;
        Ok(sharepb::CheckCanShareReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Report share
    pub async fn report_share(&self) -> AppResult<sharepb::ReportShareReply> {
        let req = sharepb::ReportShareRequest { shared: true };
        let reply_bytes = self
            .network
            .send_request(&codec::REPORT_SHARE, req.encode_to_vec())
            .await?;
        Ok(sharepb::ReportShareReply::decode(reply_bytes.as_slice())?)
    }

    /// Claim share reward
    pub async fn claim_share_reward(&self) -> AppResult<sharepb::ClaimShareRewardReply> {
        let req = sharepb::ClaimShareRewardRequest { claimed: true };
        let reply_bytes = self
            .network
            .send_request(&codec::CLAIM_SHARE_REWARD, req.encode_to_vec())
            .await?;
        Ok(sharepb::ClaimShareRewardReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    // ========== Auto-claim all dailies ==========

    /// Auto-claim all daily rewards (VIP, red packets, illustrated, share)
    pub async fn auto_claim_dailies(&self) -> AppResult<()> {
        // QQ VIP daily gift
        match self.get_daily_gift_status().await {
            Ok(status) => {
                if status.can_claim && status.has_gift {
                    log::info!("Claiming QQ VIP daily gift");
                    let _ = self.claim_daily_gift().await;
                }
            }
            Err(e) => log::debug!("VIP gift check failed: {}", e),
        }

        // Red packets
        match self.get_red_packet_status().await {
            Ok(status) => {
                for info in &status.infos {
                    if info.can_claim {
                        log::info!("Claiming red packet {}", info.id);
                        let _ = self.claim_red_packet(info.id).await;
                    }
                }
            }
            Err(e) => log::debug!("Red packet check failed: {}", e),
        }

        // Illustrated rewards
        match self.get_illustrated_list().await {
            Ok(list) => {
                let has_rewards = list.items.iter().any(|item| item.has_reward);
                if has_rewards {
                    log::info!("Claiming illustrated rewards");
                    let _ = self.claim_all_illustrated_rewards().await;
                }
            }
            Err(e) => log::debug!("Illustrated check failed: {}", e),
        }

        // Share reward
        match self.check_can_share().await {
            Ok(reply) => {
                if reply.can_share {
                    let _ = self.report_share().await;
                    let _ = self.claim_share_reward().await;
                }
            }
            Err(e) => log::debug!("Share check failed: {}", e),
        }

        Ok(())
    }
}
