use std::sync::Arc;

use prost::Message;

use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::mallpb;

pub struct MallService {
    network: Arc<NetworkManager>,
}

impl MallService {
    pub fn new(network: Arc<NetworkManager>) -> Self {
        Self { network }
    }

    /// Get mall goods list by slot type
    pub async fn get_mall_list(
        &self,
        slot_type: i32,
    ) -> AppResult<mallpb::GetMallListBySlotTypeResponse> {
        let req = mallpb::GetMallListBySlotTypeRequest { slot_type };
        let reply_bytes = self
            .network
            .send_request(&codec::GET_MALL_LIST, req.encode_to_vec())
            .await?;
        Ok(mallpb::GetMallListBySlotTypeResponse::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Purchase mall goods
    pub async fn purchase(
        &self,
        goods_id: i32,
        count: i32,
    ) -> AppResult<mallpb::PurchaseResponse> {
        let req = mallpb::PurchaseRequest { goods_id, count };
        let reply_bytes = self
            .network
            .send_request(&codec::PURCHASE, req.encode_to_vec())
            .await?;
        Ok(mallpb::PurchaseResponse::decode(reply_bytes.as_slice())?)
    }

    /// Get month card info
    pub async fn get_month_card_infos(&self) -> AppResult<mallpb::GetMonthCardInfosReply> {
        let req = mallpb::GetMonthCardInfosRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::GET_MONTH_CARD_INFOS, req.encode_to_vec())
            .await?;
        Ok(mallpb::GetMonthCardInfosReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Claim month card reward
    pub async fn claim_month_card_reward(
        &self,
        goods_id: i32,
    ) -> AppResult<mallpb::ClaimMonthCardRewardReply> {
        let req = mallpb::ClaimMonthCardRewardRequest { goods_id };
        let reply_bytes = self
            .network
            .send_request(&codec::CLAIM_MONTH_CARD_REWARD, req.encode_to_vec())
            .await?;
        Ok(mallpb::ClaimMonthCardRewardReply::decode(
            reply_bytes.as_slice(),
        )?)
    }

    /// Auto-claim all available month card rewards
    pub async fn auto_claim_month_card(&self) -> AppResult<()> {
        let reply = self.get_month_card_infos().await?;
        for info in &reply.infos {
            if info.can_claim {
                log::info!("Claiming month card reward: {}", info.goods_id);
                let _ = self.claim_month_card_reward(info.goods_id).await;
            }
        }
        Ok(())
    }

    /// Auto-claim free mall gifts
    pub async fn auto_claim_free_gifts(&self) -> AppResult<()> {
        for slot_type in 1..=5 {
            let list = match self.get_mall_list(slot_type).await {
                Ok(l) => l,
                Err(_) => continue,
            };

            for goods_bytes in &list.goods_list {
                if let Ok(goods) = mallpb::MallGoods::decode(goods_bytes.as_slice()) {
                    if goods.is_free {
                        log::info!("Claiming free mall gift: {}", goods.name);
                        let _ = self.purchase(goods.goods_id, 1).await;
                    }
                }
            }
        }
        Ok(())
    }
}
