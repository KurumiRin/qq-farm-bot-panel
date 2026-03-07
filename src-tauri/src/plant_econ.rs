use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct PlantEcon {
    pub exp: i64,
    pub fruit_count: i64,
    pub fruit_price: i64,
    pub seed_price: i64,
    pub seasons: i64,
}

impl PlantEcon {
    pub fn gold_income(&self) -> i64 {
        self.fruit_count * self.fruit_price
    }

    pub fn net_profit(&self) -> i64 {
        self.net_profit_with_bonus(0)
    }

    pub fn net_profit_with_bonus(&self, yield_bonus: i64) -> i64 {
        let fruit = self.fruit_count * (100 + yield_bonus) / 100;
        let income = fruit * self.fruit_price;
        if self.seasons >= 2 {
            income * 2 - self.seed_price
        } else {
            income - self.seed_price
        }
    }

    pub fn total_exp(&self) -> i64 {
        self.total_exp_with_bonus(0)
    }

    pub fn total_exp_with_bonus(&self, exp_bonus: i64) -> i64 {
        let base = self.exp * (100 + exp_bonus) / 100;
        if self.seasons >= 2 {
            base * 2
        } else {
            base
        }
    }
}

#[derive(serde::Deserialize)]
struct SeedJson {
    id: i64,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    level: i64,
    price: i64,
    exp: i64,
    fruit_count: i64,
    fruit_price: i64,
    seasons: i64,
}

static PLANT_ECON_MAP: LazyLock<HashMap<i64, PlantEcon>> = LazyLock::new(|| {
    let json_str = include_str!("../../shared-data/seeds.json");
    let seeds: Vec<SeedJson> = serde_json::from_str(json_str).unwrap_or_default();
    seeds
        .into_iter()
        .map(|s| {
            (
                s.id,
                PlantEcon {
                    exp: s.exp,
                    fruit_count: s.fruit_count,
                    fruit_price: s.fruit_price,
                    seed_price: s.price,
                    seasons: s.seasons,
                },
            )
        })
        .collect()
});

pub fn get_plant_econ(seed_id: i64) -> Option<&'static PlantEcon> {
    PLANT_ECON_MAP.get(&seed_id)
}
