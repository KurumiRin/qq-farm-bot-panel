use std::sync::Arc;

use prost::Message;

use crate::error::AppResult;
use crate::network::codec;
use crate::network::NetworkManager;
use crate::proto::{corepb, itempb};
use crate::state::AppState;

pub struct WarehouseService {
    network: Arc<NetworkManager>,
    state: Arc<AppState>,
}

impl WarehouseService {
    pub fn new(network: Arc<NetworkManager>, state: Arc<AppState>) -> Self {
        Self { network, state }
    }

    /// Get backpack contents
    pub async fn get_bag(&self) -> AppResult<itempb::BagReply> {
        let req = itempb::BagRequest {};
        let reply_bytes = self
            .network
            .send_request(&codec::BAG, req.encode_to_vec())
            .await?;
        Ok(itempb::BagReply::decode(reply_bytes.as_slice())?)
    }

    /// Sell items (batch, max 15 per request)
    pub async fn sell_items(&self, items: Vec<corepb::Item>) -> AppResult<itempb::SellReply> {
        let req = itempb::SellRequest { items };
        let reply_bytes = self
            .network
            .send_request(&codec::SELL, req.encode_to_vec())
            .await?;
        self.state.record_stat(|s| s.items_sold += 1);
        Ok(itempb::SellReply::decode(reply_bytes.as_slice())?)
    }

    /// Use an item
    pub async fn use_item(
        &self,
        item_id: i64,
        count: i64,
        land_ids: Vec<i64>,
    ) -> AppResult<itempb::UseReply> {
        let req = itempb::UseRequest {
            item_id,
            count,
            land_ids,
        };
        let reply_bytes = self
            .network
            .send_request(&codec::USE_ITEM, req.encode_to_vec())
            .await?;
        Ok(itempb::UseReply::decode(reply_bytes.as_slice())?)
    }

    /// Batch use items
    pub async fn batch_use(&self, items: Vec<corepb::Item>) -> AppResult<itempb::BatchUseReply> {
        let req = itempb::BatchUseRequest { items };
        let reply_bytes = self
            .network
            .send_request(&codec::BATCH_USE, req.encode_to_vec())
            .await?;
        Ok(itempb::BatchUseReply::decode(reply_bytes.as_slice())?)
    }

    /// Auto-sell all fruits in bag
    pub async fn auto_sell_fruits(&self) -> AppResult<()> {
        let bag = self.get_bag().await?;
        let items = match bag.item_bag {
            Some(bag) => bag.items,
            None => return Ok(()),
        };

        // Fruit IDs: 40000+ range (type 6)
        let fruits: Vec<corepb::Item> = items
            .into_iter()
            .filter(|item| item.id >= 40000 && item.count > 0)
            .collect();

        if fruits.is_empty() {
            return Ok(());
        }

        // Sell in batches of 15
        for chunk in fruits.chunks(15) {
            let batch: Vec<corepb::Item> = chunk
                .iter()
                .map(|item| corepb::Item {
                    id: item.id,
                    count: item.count,
                    ..Default::default()
                })
                .collect();

            log::info!("Selling {} items", batch.len());
            let _ = self.sell_items(batch).await;
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        Ok(())
    }
}
