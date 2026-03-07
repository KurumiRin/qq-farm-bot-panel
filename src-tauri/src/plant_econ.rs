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
    /// Total gold income from one harvest (fruit_count * fruit_price)
    /// For 2-season plants, this is per season
    pub fn gold_income(&self) -> i64 {
        self.fruit_count * self.fruit_price
    }

    /// Net profit = income - seed_cost
    /// For 2-season: income * 2 - seed_price
    pub fn net_profit(&self) -> i64 {
        self.net_profit_with_bonus(0)
    }

    /// Net profit with land yield bonus (percentage, e.g. 10 = +10%)
    pub fn net_profit_with_bonus(&self, yield_bonus: i64) -> i64 {
        let fruit = self.fruit_count * (100 + yield_bonus) / 100;
        let income = fruit * self.fruit_price;
        if self.seasons >= 2 {
            income * 2 - self.seed_price
        } else {
            income - self.seed_price
        }
    }

    /// Total exp from one full growth cycle
    /// For 2-season: exp * 2
    pub fn total_exp(&self) -> i64 {
        self.total_exp_with_bonus(0)
    }

    /// Total exp with land exp bonus (percentage, e.g. 10 = +10%)
    pub fn total_exp_with_bonus(&self, exp_bonus: i64) -> i64 {
        let base = self.exp * (100 + exp_bonus) / 100;
        if self.seasons >= 2 {
            base * 2
        } else {
            base
        }
    }
}

static PLANT_ECON_MAP: LazyLock<HashMap<i64, PlantEcon>> = LazyLock::new(|| {
    let data: &[(i64, PlantEcon)] = &[
        (20001, PlantEcon { exp: 784, fruit_count: 200, fruit_price: 56, seed_price: 1120, seasons: 1 }),
        (20002, PlantEcon { exp: 1, fruit_count: 5, fruit_price: 2, seed_price: 1, seasons: 1 }),
        (20003, PlantEcon { exp: 2, fruit_count: 10, fruit_price: 2, seed_price: 2, seasons: 1 }),
        (20004, PlantEcon { exp: 82, fruit_count: 40, fruit_price: 21, seed_price: 84, seasons: 1 }),
        (20005, PlantEcon { exp: 128, fruit_count: 60, fruit_price: 22, seed_price: 134, seasons: 1 }),
        (20006, PlantEcon { exp: 544, fruit_count: 200, fruit_price: 28, seed_price: 576, seasons: 1 }),
        (20007, PlantEcon { exp: 576, fruit_count: 200, fruit_price: 32, seed_price: 640, seasons: 1 }),
        (20008, PlantEcon { exp: 608, fruit_count: 200, fruit_price: 35, seed_price: 704, seasons: 1 }),
        (20009, PlantEcon { exp: 688, fruit_count: 200, fruit_price: 44, seed_price: 888, seasons: 1 }),
        (20010, PlantEcon { exp: 736, fruit_count: 200, fruit_price: 49, seed_price: 992, seasons: 1 }),
        (20011, PlantEcon { exp: 1176, fruit_count: 200, fruit_price: 84, seed_price: 1680, seasons: 1 }),
        (20013, PlantEcon { exp: 1016, fruit_count: 200, fruit_price: 86, seed_price: 1728, seasons: 1 }),
        (20014, PlantEcon { exp: 896, fruit_count: 200, fruit_price: 70, seed_price: 1400, seasons: 1 }),
        (20015, PlantEcon { exp: 2688, fruit_count: 200, fruit_price: 210, seed_price: 4200, seasons: 1 }),
        (20016, PlantEcon { exp: 867, fruit_count: 200, fruit_price: 117, seed_price: 4680, seasons: 2 }),
        (20018, PlantEcon { exp: 952, fruit_count: 200, fruit_price: 78, seed_price: 1560, seasons: 1 }),
        (20019, PlantEcon { exp: 2856, fruit_count: 200, fruit_price: 234, seed_price: 4680, seasons: 1 }),
        (20022, PlantEcon { exp: 2601, fruit_count: 200, fruit_price: 351, seed_price: 14040, seasons: 2 }),
        (20023, PlantEcon { exp: 1080, fruit_count: 200, fruit_price: 96, seed_price: 1920, seasons: 1 }),
        (20026, PlantEcon { exp: 3240, fruit_count: 200, fruit_price: 288, seed_price: 5760, seasons: 1 }),
        (20027, PlantEcon { exp: 858, fruit_count: 200, fruit_price: 79, seed_price: 3168, seasons: 2 }),
        (20029, PlantEcon { exp: 456, fruit_count: 200, fruit_price: 43, seed_price: 1746, seasons: 2 }),
        (20031, PlantEcon { exp: 2736, fruit_count: 200, fruit_price: 261, seed_price: 10476, seasons: 2 }),
        (20033, PlantEcon { exp: 2880, fruit_count: 200, fruit_price: 287, seed_price: 11484, seasons: 2 }),
        (20034, PlantEcon { exp: 1020, fruit_count: 200, fruit_price: 104, seed_price: 4176, seasons: 2 }),
        (20035, PlantEcon { exp: 3060, fruit_count: 200, fruit_price: 313, seed_price: 12528, seasons: 2 }),
        (20036, PlantEcon { exp: 1287, fruit_count: 200, fruit_price: 118, seed_price: 4752, seasons: 2 }),
        (20037, PlantEcon { exp: 912, fruit_count: 200, fruit_price: 52, seed_price: 1056, seasons: 1 }),
        (20038, PlantEcon { exp: 1074, fruit_count: 200, fruit_price: 114, seed_price: 4560, seasons: 2 }),
        (20039, PlantEcon { exp: 567, fruit_count: 200, fruit_price: 61, seed_price: 2472, seasons: 2 }),
        (20041, PlantEcon { exp: 1824, fruit_count: 200, fruit_price: 105, seed_price: 2112, seasons: 1 }),
        (20042, PlantEcon { exp: 3402, fruit_count: 200, fruit_price: 370, seed_price: 14832, seasons: 2 }),
        (20043, PlantEcon { exp: 2574, fruit_count: 200, fruit_price: 237, seed_price: 9504, seasons: 2 }),
        (20044, PlantEcon { exp: 1524, fruit_count: 200, fruit_price: 129, seed_price: 2592, seasons: 1 }),
        (20045, PlantEcon { exp: 480, fruit_count: 200, fruit_price: 47, seed_price: 1914, seasons: 2 }),
        (20047, PlantEcon { exp: 1428, fruit_count: 200, fruit_price: 117, seed_price: 2340, seasons: 1 }),
        (20048, PlantEcon { exp: 1194, fruit_count: 200, fruit_price: 134, seed_price: 5364, seasons: 2 }),
        (20049, PlantEcon { exp: 912, fruit_count: 200, fruit_price: 87, seed_price: 3492, seasons: 2 }),
        (20050, PlantEcon { exp: 429, fruit_count: 200, fruit_price: 39, seed_price: 1584, seasons: 2 }),
        (20051, PlantEcon { exp: 816, fruit_count: 200, fruit_price: 43, seed_price: 864, seasons: 1 }),
        (20052, PlantEcon { exp: 1368, fruit_count: 200, fruit_price: 130, seed_price: 5238, seasons: 2 }),
        (20053, PlantEcon { exp: 1611, fruit_count: 200, fruit_price: 171, seed_price: 6840, seasons: 2 }),
        (20054, PlantEcon { exp: 960, fruit_count: 200, fruit_price: 95, seed_price: 3828, seasons: 2 }),
        (20055, PlantEcon { exp: 510, fruit_count: 200, fruit_price: 52, seed_price: 2088, seasons: 2 }),
        (20056, PlantEcon { exp: 1134, fruit_count: 200, fruit_price: 123, seed_price: 4944, seasons: 2 }),
        (20057, PlantEcon { exp: 597, fruit_count: 200, fruit_price: 67, seed_price: 2682, seasons: 2 }),
        (20058, PlantEcon { exp: 1791, fruit_count: 200, fruit_price: 201, seed_price: 8046, seasons: 2 }),
        (20059, PlantEcon { exp: 5, fruit_count: 20, fruit_price: 2, seed_price: 5, seasons: 1 }),
        (20060, PlantEcon { exp: 41, fruit_count: 30, fruit_price: 14, seed_price: 42, seasons: 1 }),
        (20061, PlantEcon { exp: 62, fruit_count: 40, fruit_price: 15, seed_price: 63, seasons: 1 }),
        (20062, PlantEcon { exp: 2352, fruit_count: 200, fruit_price: 168, seed_price: 3360, seasons: 1 }),
        (20063, PlantEcon { exp: 1980, fruit_count: 200, fruit_price: 234, seed_price: 9360, seasons: 2 }),
        (20064, PlantEcon { exp: 20, fruit_count: 30, fruit_price: 7, seed_price: 21, seasons: 1 }),
        (20065, PlantEcon { exp: 10, fruit_count: 20, fruit_price: 5, seed_price: 10, seasons: 1 }),
        (20066, PlantEcon { exp: 106, fruit_count: 60, fruit_price: 18, seed_price: 111, seasons: 1 }),
        (20067, PlantEcon { exp: 537, fruit_count: 200, fruit_price: 57, seed_price: 2280, seasons: 2 }),
        (20068, PlantEcon { exp: 693, fruit_count: 200, fruit_price: 84, seed_price: 3360, seasons: 2 }),
        (20070, PlantEcon { exp: 1344, fruit_count: 200, fruit_price: 105, seed_price: 2100, seasons: 1 }),
        (20071, PlantEcon { exp: 160, fruit_count: 80, fruit_price: 20, seed_price: 167, seasons: 1 }),
        (20072, PlantEcon { exp: 3048, fruit_count: 200, fruit_price: 259, seed_price: 5184, seasons: 1 }),
        (20073, PlantEcon { exp: 392, fruit_count: 200, fruit_price: 28, seed_price: 560, seasons: 1 }),
        (20074, PlantEcon { exp: 4158, fruit_count: 200, fruit_price: 504, seed_price: 20160, seasons: 2 }),
        (20075, PlantEcon { exp: 1701, fruit_count: 200, fruit_price: 185, seed_price: 7416, seasons: 2 }),
        (20076, PlantEcon { exp: 3762, fruit_count: 200, fruit_price: 434, seed_price: 17388, seasons: 2 }),
        (20077, PlantEcon { exp: 1254, fruit_count: 200, fruit_price: 144, seed_price: 5796, seasons: 2 }),
        (20078, PlantEcon { exp: 2079, fruit_count: 200, fruit_price: 252, seed_price: 10080, seasons: 2 }),
        (20079, PlantEcon { exp: 3582, fruit_count: 200, fruit_price: 402, seed_price: 16092, seasons: 2 }),
        (20080, PlantEcon { exp: 3222, fruit_count: 200, fruit_price: 342, seed_price: 13680, seasons: 2 }),
        (20083, PlantEcon { exp: 726, fruit_count: 200, fruit_price: 90, seed_price: 3600, seasons: 2 }),
        (20084, PlantEcon { exp: 2178, fruit_count: 200, fruit_price: 270, seed_price: 10800, seasons: 2 }),
        (20085, PlantEcon { exp: 762, fruit_count: 200, fruit_price: 96, seed_price: 3858, seasons: 2 }),
        (20086, PlantEcon { exp: 2286, fruit_count: 200, fruit_price: 289, seed_price: 11574, seasons: 2 }),
        (20087, PlantEcon { exp: 795, fruit_count: 200, fruit_price: 103, seed_price: 4122, seasons: 2 }),
        (20088, PlantEcon { exp: 2385, fruit_count: 200, fruit_price: 309, seed_price: 12366, seasons: 2 }),
        (20089, PlantEcon { exp: 831, fruit_count: 200, fruit_price: 109, seed_price: 4392, seasons: 2 }),
        (20090, PlantEcon { exp: 2493, fruit_count: 200, fruit_price: 329, seed_price: 13176, seasons: 2 }),
        (20091, PlantEcon { exp: 2208, fruit_count: 200, fruit_price: 148, seed_price: 2976, seasons: 1 }),
        (20095, PlantEcon { exp: 1620, fruit_count: 200, fruit_price: 144, seed_price: 2880, seasons: 1 }),
        (20096, PlantEcon { exp: 192, fruit_count: 80, fruit_price: 25, seed_price: 201, seasons: 1 }),
        (20097, PlantEcon { exp: 1032, fruit_count: 200, fruit_price: 66, seed_price: 1332, seasons: 1 }),
        (20098, PlantEcon { exp: 864, fruit_count: 200, fruit_price: 48, seed_price: 960, seasons: 1 }),
        (20099, PlantEcon { exp: 272, fruit_count: 200, fruit_price: 14, seed_price: 288, seasons: 1 }),
        (20100, PlantEcon { exp: 476, fruit_count: 200, fruit_price: 39, seed_price: 780, seasons: 1 }),
        (20103, PlantEcon { exp: 368, fruit_count: 200, fruit_price: 24, seed_price: 496, seasons: 1 }),
        (20104, PlantEcon { exp: 420, fruit_count: 200, fruit_price: 31, seed_price: 624, seasons: 1 }),
        (20105, PlantEcon { exp: 304, fruit_count: 200, fruit_price: 17, seed_price: 352, seasons: 1 }),
        (20110, PlantEcon { exp: 648, fruit_count: 200, fruit_price: 39, seed_price: 792, seasons: 1 }),
        (20116, PlantEcon { exp: 660, fruit_count: 200, fruit_price: 78, seed_price: 3120, seasons: 2 }),
        (20120, PlantEcon { exp: 1632, fruit_count: 200, fruit_price: 86, seed_price: 1728, seasons: 1 }),
        (20126, PlantEcon { exp: 1320, fruit_count: 200, fruit_price: 156, seed_price: 6240, seasons: 2 }),
        (20128, PlantEcon { exp: 508, fruit_count: 200, fruit_price: 43, seed_price: 864, seasons: 1 }),
        (20135, PlantEcon { exp: 840, fruit_count: 200, fruit_price: 62, seed_price: 1248, seasons: 1 }),
        (20141, PlantEcon { exp: 1260, fruit_count: 200, fruit_price: 93, seed_price: 1872, seasons: 1 }),
        (20142, PlantEcon { exp: 2520, fruit_count: 200, fruit_price: 187, seed_price: 3744, seasons: 1 }),
        (20143, PlantEcon { exp: 972, fruit_count: 200, fruit_price: 59, seed_price: 1188, seasons: 1 }),
        (20145, PlantEcon { exp: 448, fruit_count: 200, fruit_price: 35, seed_price: 700, seasons: 1 }),
        (20147, PlantEcon { exp: 1944, fruit_count: 200, fruit_price: 118, seed_price: 2376, seasons: 1 }),
        (20161, PlantEcon { exp: 324, fruit_count: 200, fruit_price: 19, seed_price: 396, seasons: 1 }),
        (20162, PlantEcon { exp: 344, fruit_count: 200, fruit_price: 22, seed_price: 444, seasons: 1 }),
        (20201, PlantEcon { exp: 4770, fruit_count: 200, fruit_price: 618, seed_price: 24732, seasons: 2 }),
        (20202, PlantEcon { exp: 1662, fruit_count: 200, fruit_price: 219, seed_price: 8784, seasons: 2 }),
        (20204, PlantEcon { exp: 1734, fruit_count: 200, fruit_price: 234, seed_price: 9360, seasons: 2 }),
        (20218, PlantEcon { exp: 627, fruit_count: 200, fruit_price: 72, seed_price: 2898, seasons: 2 }),
        (20220, PlantEcon { exp: 1881, fruit_count: 200, fruit_price: 217, seed_price: 8694, seasons: 2 }),
        (20221, PlantEcon { exp: 3960, fruit_count: 200, fruit_price: 468, seed_price: 18720, seasons: 2 }),
        (20222, PlantEcon { exp: 1386, fruit_count: 200, fruit_price: 168, seed_price: 6720, seasons: 2 }),
        (20225, PlantEcon { exp: 1452, fruit_count: 200, fruit_price: 180, seed_price: 7200, seasons: 2 }),
        (20226, PlantEcon { exp: 4356, fruit_count: 200, fruit_price: 540, seed_price: 21600, seasons: 2 }),
        (20227, PlantEcon { exp: 1524, fruit_count: 200, fruit_price: 192, seed_price: 7716, seasons: 2 }),
        (20228, PlantEcon { exp: 4572, fruit_count: 200, fruit_price: 578, seed_price: 23148, seasons: 2 }),
        (20229, PlantEcon { exp: 4986, fruit_count: 200, fruit_price: 658, seed_price: 26352, seasons: 2 }),
        (20235, PlantEcon { exp: 1590, fruit_count: 200, fruit_price: 206, seed_price: 8244, seasons: 2 }),
        (20242, PlantEcon { exp: 5202, fruit_count: 200, fruit_price: 702, seed_price: 28080, seasons: 2 }),
        (20259, PlantEcon { exp: 288, fruit_count: 200, fruit_price: 16, seed_price: 320, seasons: 1 }),
        (20305, PlantEcon { exp: 1728, fruit_count: 200, fruit_price: 96, seed_price: 1920, seasons: 1 }),
        (20306, PlantEcon { exp: 2064, fruit_count: 200, fruit_price: 133, seed_price: 2664, seasons: 1 }),
        (20308, PlantEcon { exp: 1104, fruit_count: 200, fruit_price: 74, seed_price: 1488, seasons: 1 }),
        (20396, PlantEcon { exp: 540, fruit_count: 200, fruit_price: 48, seed_price: 960, seasons: 1 }),
        (20413, PlantEcon { exp: 1530, fruit_count: 200, fruit_price: 156, seed_price: 6264, seasons: 2 }),
        (20442, PlantEcon { exp: 1440, fruit_count: 200, fruit_price: 143, seed_price: 5742, seasons: 2 }),
        (21542, PlantEcon { exp: 688, fruit_count: 20, fruit_price: 1, seed_price: 0, seasons: 1 }),
        (29998, PlantEcon { exp: 1, fruit_count: 50, fruit_price: 702, seed_price: 28080, seasons: 1 }),
    ];
    data.iter().cloned().collect()
});

pub fn get_plant_econ(seed_id: i64) -> Option<&'static PlantEcon> {
    PLANT_ECON_MAP.get(&seed_id)
}
