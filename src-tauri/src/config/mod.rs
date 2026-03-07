use serde::{Deserialize, Serialize};

pub const SERVER_URL: &str = "wss://gate-obt.nqf.qq.com/prod/ws";
pub const CLIENT_VERSION: &str = "1.6.0.14_20251224";
pub const PLATFORM: &str = "qq";
pub const OS: &str = "iOS";
pub const HEARTBEAT_INTERVAL_MS: u64 = 25_000;
// These are only fallbacks; runtime reads from AutomationConfig.intervals
pub const FARM_CHECK_INTERVAL_MS: u64 = 30_000;
pub const FRIEND_CHECK_INTERVAL_MS: u64 = 120_000;
pub const REQUEST_TIMEOUT_MS: u64 = 10_000;
pub const RECONNECT_DELAY_MS: u64 = 5_000;
pub const HEARTBEAT_MISS_THRESHOLD_MS: u64 = 60_000;
/// Force reconnect after this many consecutive heartbeat failures
pub const HEARTBEAT_MAX_MISS: u32 = 2;

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

/// Planting strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlantingStrategy {
    #[default]
    Preferred,
    Level,
    MaxExp,
    MaxProfit,
}

/// Fertilizer strategy
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FertilizerStrategy {
    #[default]
    None,
    Normal,
    Organic,
    Both,
}

/// Interval settings (in seconds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntervalConfig {
    pub farm_min: u64,
    pub farm_max: u64,
    pub friend_min: u64,
    pub friend_max: u64,
}

impl Default for IntervalConfig {
    fn default() -> Self {
        Self {
            farm_min: 30,
            farm_max: 60,
            friend_min: 120,
            friend_max: 300,
        }
    }
}

/// Friend quiet hours
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHoursConfig {
    pub enabled: bool,
    pub start: String,
    pub end: String,
}

impl Default for QuietHoursConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start: "23:00".to_string(),
            end: "07:00".to_string(),
        }
    }
}

/// Automation settings per account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationConfig {
    // -- Strategy --
    #[serde(default)]
    pub planting_strategy: PlantingStrategy,
    pub preferred_seed_id: Option<i64>,
    #[serde(default)]
    pub intervals: IntervalConfig,
    #[serde(default)]
    pub friend_quiet_hours: QuietHoursConfig,

    // -- Farm --
    pub auto_harvest: bool,
    pub auto_plant: bool,
    #[serde(default = "default_true")]
    pub auto_farm_manage: bool,
    pub auto_water: bool,
    pub auto_weed: bool,
    pub auto_insecticide: bool,
    #[serde(default)]
    pub fertilizer_strategy: FertilizerStrategy,
    #[serde(default)]
    pub auto_land_upgrade: bool,
    #[serde(default = "default_true")]
    pub auto_farm_push: bool,

    // -- Sell & Claim --
    pub auto_sell: bool,
    pub auto_claim_tasks: bool,
    pub auto_claim_emails: bool,
    #[serde(default)]
    pub auto_free_gifts: bool,
    #[serde(default)]
    pub auto_share_reward: bool,
    #[serde(default)]
    pub auto_vip_gift: bool,
    #[serde(default)]
    pub auto_month_card: bool,
    #[serde(default)]
    pub auto_open_server_gift: bool,
    #[serde(default)]
    pub auto_fertilizer_gift: bool,
    #[serde(default)]
    pub auto_fertilizer_buy: bool,

    // -- Social --
    pub auto_steal: bool,
    pub auto_help_water: bool,
    pub auto_help_weed: bool,
    pub auto_help_insecticide: bool,
    #[serde(default)]
    pub auto_friend_bad: bool,
    #[serde(default)]
    pub auto_friend_help_exp_limit: bool,

    pub friend_blacklist: Vec<i64>,

    // Legacy field - kept for deserialization compat, ignored
    #[serde(default, skip_serializing)]
    pub auto_fertilize: Option<bool>,
}

fn default_true() -> bool {
    true
}

impl Default for AutomationConfig {
    fn default() -> Self {
        Self {
            planting_strategy: PlantingStrategy::default(),
            preferred_seed_id: None,
            intervals: IntervalConfig::default(),
            friend_quiet_hours: QuietHoursConfig::default(),

            auto_harvest: true,
            auto_plant: true,
            auto_farm_manage: true,
            auto_water: true,
            auto_weed: true,
            auto_insecticide: true,
            fertilizer_strategy: FertilizerStrategy::default(),
            auto_land_upgrade: true,
            auto_farm_push: true,

            auto_sell: false,
            auto_claim_tasks: true,
            auto_claim_emails: true,
            auto_free_gifts: true,
            auto_share_reward: true,
            auto_vip_gift: true,
            auto_month_card: true,
            auto_open_server_gift: true,
            auto_fertilizer_gift: false,
            auto_fertilizer_buy: false,

            auto_steal: true,
            auto_help_water: true,
            auto_help_weed: true,
            auto_help_insecticide: true,
            auto_friend_bad: false,
            auto_friend_help_exp_limit: true,

            friend_blacklist: Vec::new(),
            auto_fertilize: None,
        }
    }
}
