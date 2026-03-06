use serde::{Deserialize, Serialize};

pub const SERVER_URL: &str = "wss://gate-obt.nqf.qq.com/prod/ws";
pub const CLIENT_VERSION: &str = "1.6.0.14_20251224";
pub const PLATFORM: &str = "qq";
pub const OS: &str = "iOS";
pub const HEARTBEAT_INTERVAL_MS: u64 = 25_000;
pub const FARM_CHECK_INTERVAL_MS: u64 = 2_000;
pub const FRIEND_CHECK_INTERVAL_MS: u64 = 10_000;
pub const REQUEST_TIMEOUT_MS: u64 = 10_000;
pub const RECONNECT_DELAY_MS: u64 = 5_000;
pub const HEARTBEAT_MISS_THRESHOLD_MS: u64 = 60_000;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 MicroMessenger/7.0.20.1781(0x6700143B) NetType/WIFI MiniProgramEnv/Windows WindowsWechat/WMPF WindowsWechat(0x63090a13)";
pub const ORIGIN: &str = "https://gate-obt.nqf.qq.com";

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlantPhase {
    Unknown = 0,
    Seed = 1,
    Germination = 2,
    SmallLeaves = 3,
    LargeLeaves = 4,
    Blooming = 5,
    Mature = 6,
    Dead = 7,
}

impl PlantPhase {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Seed,
            2 => Self::Germination,
            3 => Self::SmallLeaves,
            4 => Self::LargeLeaves,
            5 => Self::Blooming,
            6 => Self::Mature,
            7 => Self::Dead,
            _ => Self::Unknown,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Unknown => "未知",
            Self::Seed => "种子",
            Self::Germination => "发芽",
            Self::SmallLeaves => "小叶",
            Self::LargeLeaves => "大叶",
            Self::Blooming => "开花",
            Self::Mature => "成熟",
            Self::Dead => "枯死",
        }
    }
}

/// Operation type IDs used in OperationLimit
#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    HelpWater = 10001,
    HelpInsecticide = 10002,
    HelpWeedOut = 10003,
    Steal = 10004,
    PutInsects = 10005,
    PutWeeds = 10006,
}

/// Item IDs
pub mod item_ids {
    pub const GOLD: i64 = 1;
    pub const GOLD_ALT: i64 = 1001;
    pub const COUPON: i64 = 1002;
    pub const NORMAL_FERTILIZER: i64 = 1011;
    pub const ORGANIC_FERTILIZER: i64 = 1012;
    pub const EXP_ITEM: i64 = 1101;
}

/// Automation settings per account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    pub auto_harvest: bool,
    pub auto_plant: bool,
    pub auto_water: bool,
    pub auto_weed: bool,
    pub auto_insecticide: bool,
    pub auto_fertilize: bool,
    pub auto_sell: bool,
    pub auto_steal: bool,
    pub auto_help_water: bool,
    pub auto_help_weed: bool,
    pub auto_help_insecticide: bool,
    pub auto_claim_tasks: bool,
    pub auto_claim_emails: bool,
    pub preferred_seed_id: Option<i64>,
    pub friend_blacklist: Vec<i64>,
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            auto_harvest: true,
            auto_plant: true,
            auto_water: true,
            auto_weed: true,
            auto_insecticide: true,
            auto_fertilize: false,
            auto_sell: true,
            auto_steal: true,
            auto_help_water: true,
            auto_help_weed: true,
            auto_help_insecticide: true,
            auto_claim_tasks: true,
            auto_claim_emails: true,
            preferred_seed_id: None,
            friend_blacklist: Vec::new(),
        }
    }
}
