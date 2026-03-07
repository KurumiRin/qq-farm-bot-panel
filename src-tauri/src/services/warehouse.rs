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
    /// Returns total number of fruit items sold
    pub async fn auto_sell_fruits(&self) -> AppResult<i64> {
        let bag = self.get_bag().await?;
        let items = match bag.item_bag {
            Some(bag) => bag.items,
            None => return Ok(0),
        };

        // Fruit IDs: 40000-49999 range
        let fruits: Vec<corepb::Item> = items
            .into_iter()
            .filter(|item| item.id >= 40000 && item.id < 50000 && item.count > 0)
            .collect();

        if fruits.is_empty() {
            return Ok(0);
        }

        let total_count: i64 = fruits.iter().map(|i| i.count).sum();

        // Sell in batches of 15, preserving uid for each item
        for chunk in fruits.chunks(15) {
            let batch: Vec<corepb::Item> = chunk
                .iter()
                .map(|item| {
                    let mut sell_item = corepb::Item {
                        id: item.id,
                        count: item.count,
                        ..Default::default()
                    };
                    if item.uid > 0 {
                        sell_item.uid = item.uid;
                    }
                    sell_item
                })
                .collect();

            let batch_len = batch.len();
            log::info!("Selling batch: {:?}", batch.iter().map(|i| (i.id, i.count, i.uid)).collect::<Vec<_>>());
            match self.sell_items(batch).await {
                Ok(reply) => {
                    log::info!("Sold batch of {} fruit types, got {} items back",
                        batch_len, reply.get_items.len());
                }
                Err(e) => {
                    log::warn!("Batch sell failed, trying one by one: {}", e);
                    // Fallback: sell one by one, skip failures
                    for item in chunk {
                        let mut single_item = corepb::Item {
                            id: item.id,
                            count: item.count,
                            ..Default::default()
                        };
                        if item.uid > 0 {
                            single_item.uid = item.uid;
                        }
                        if let Err(e) = self.sell_items(vec![single_item]).await {
                            log::warn!("Skip unsellable item id={} uid={}: {}", item.id, item.uid, e);
                        }
                    }
                }
            }
            if fruits.len() > 15 {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }
        }

        Ok(total_count)
    }
}
