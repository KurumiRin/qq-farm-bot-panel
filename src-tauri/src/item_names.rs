use std::collections::HashMap;
use std::sync::LazyLock;

static ITEM_NAMES: LazyLock<HashMap<i64, String>> = LazyLock::new(|| {
    let json_str = include_str!("../../shared-data/item_names.json");
    let map: HashMap<String, String> = serde_json::from_str(json_str).unwrap_or_default();
    map.into_iter()
        .filter_map(|(k, v)| k.parse::<i64>().ok().map(|id| (id, v)))
        .collect()
});

pub fn get_item_name(id: i64) -> Option<&'static str> {
    ITEM_NAMES.get(&id).map(|s| s.as_str())
}
