use std::collections::HashMap;
use std::sync::LazyLock;

static ITEM_PRICES: LazyLock<HashMap<i64, i64>> = LazyLock::new(|| {
    let json_str = include_str!("../../shared-data/item_prices.json");
    let map: HashMap<String, i64> = serde_json::from_str(json_str).unwrap_or_default();
    map.into_iter()
        .filter_map(|(k, v)| k.parse::<i64>().ok().map(|id| (id, v)))
        .collect()
});

pub fn get_item_price(id: i64) -> i64 {
    ITEM_PRICES.get(&id).copied().unwrap_or(0)
}
