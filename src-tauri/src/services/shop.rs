use std::sync::Arc;

use prost::Message;

use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::shoppb;

pub struct ShopService {
    network: Arc<NetworkManager>,
}

impl ShopService {
    pub fn new(network: Arc<NetworkManager>) -> Self {
        Self { network }
    }

    /// Get all shop profiles
    pub async fn get_shop_profiles(&self) -> AppResult<shoppb::ShopProfilesReply> {
        let req = shoppb::ShopProfilesRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::SHOP_PROFILES, req.encode_to_vec())
            .await?;
        Ok(shoppb::ShopProfilesReply::decode(reply_bytes.as_slice())?)
    }

    /// Get goods in a specific shop
    pub async fn get_shop_info(&self, shop_id: i64) -> AppResult<shoppb::ShopInfoReply> {
        let req = shoppb::ShopInfoRequest { shop_id };
        let reply_bytes = self
            .network
            .send_request(&codec::SHOP_INFO, req.encode_to_vec())
            .await?;
        Ok(shoppb::ShopInfoReply::decode(reply_bytes.as_slice())?)
    }

    /// Buy goods from shop
    pub async fn buy_goods(
        &self,
        goods_id: i64,
        num: i64,
        price: i64,
    ) -> AppResult<shoppb::BuyGoodsReply> {
        let req = shoppb::BuyGoodsRequest {
            goods_id,
            num,
            price,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::BUY_GOODS, req.encode_to_vec())
            .await?;
        Ok(shoppb::BuyGoodsReply::decode(reply_bytes.as_slice())?)
    }
}
