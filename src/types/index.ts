export interface UserState {
  gid: number;
  name: string;
  level: number;
  gold: number;
  exp: number;
  coupon: number;
  avatar_url: string;
}

export type ConnectionStatus =
  | "Disconnected"
  | "Connecting"
  | "Connected"
  | "LoggingIn"
  | "LoggedIn"
  | "Reconnecting"
  | { Error: string };

export interface Stats {
  harvests: number;
  plants: number;
  waters: number;
  weeds_removed: number;
  insects_removed: number;
  steals: number;
  help_waters: number;
  help_weeds: number;
  help_insects: number;
  gold_earned: number;
  exp_earned: number;
  items_sold: number;
  tasks_claimed: number;
  emails_claimed: number;
}

export interface StatusResponse {
  user: UserState;
  connection: ConnectionStatus;
  stats: Stats;
}

export interface QrCodeResponse {
  qrsig: string;
  qrcode: string;
}

export interface MpLoginCodeResponse {
  code: string;
  qrcode: string;
}

export interface LoginStatusResponse {
  ret: string;
  msg: string;
  nickname: string;
}

export interface LogEntry {
  timestamp: number;
  level: string;
  message: string;
}

export type PlantingStrategy = "preferred" | "level" | "max_exp" | "max_profit";
export type FertilizerStrategy = "none" | "normal" | "organic" | "both";

export interface IntervalConfig {
  farm_min: number;
  farm_max: number;
  friend_min: number;
  friend_max: number;
}

export interface QuietHoursConfig {
  enabled: boolean;
  start: string;
  end: string;
}

// --- Farm ---

export interface LandView {
  id: number;
  unlocked: boolean;
  level: number;
  max_level: number;
  status: "locked" | "empty" | "growing" | "mature" | "dead";
  seed_id: number;
  seed_name: string;
  phase: number;
  phase_name: string;
  mature_in_sec: number;
  total_grow_sec: number;
  fruit_num: number;
  need_water: boolean;
  need_weed: boolean;
  need_insect: boolean;
  est_gold: number;
  est_exp: number;
  seasons: number;
}

export interface FarmSummary {
  total: number;
  unlocked: number;
  mature: number;
  growing: number;
  empty: number;
  dead: number;
  need_water: number;
  need_weed: number;
  need_insect: number;
}

export interface FarmView {
  lands: LandView[];
  summary: FarmSummary;
}

// --- Inventory ---

export interface BagItemView {
  id: number;
  count: number;
  name: string;
  category: string;
  unit_price: number;
}

export interface CurrencyView {
  id: number;
  count: number;
  name: string;
}

export interface BagView {
  items: BagItemView[];
  currencies: CurrencyView[];
  seed_count: number;
  fruit_count: number;
  fertilizer_count: number;
  other_count: number;
  normal_fert_secs: number;
  organic_fert_secs: number;
}

// --- Friends ---

export interface FriendView {
  gid: number;
  name: string;
  level: number;
  avatar_url: string;
  steal_count: number;
  dry_count: number;
  weed_count: number;
  insect_count: number;
}

export interface FriendsData {
  friends: FriendView[];
  application_count: number;
}

// --- Tasks ---

export interface RewardView {
  id: number;
  count: number;
  name: string;
}

export interface TaskView {
  id: number;
  desc: string;
  progress: number;
  total_progress: number;
  is_claimed: boolean;
  is_unlocked: boolean;
  task_type: number;
  share_multiple: number;
  rewards: RewardView[];
}

export interface ActiveRewardView {
  point_id: number;
  need_progress: number;
  status: number;
  rewards: RewardView[];
}

export interface ActiveView {
  active_type: number;
  progress: number;
  rewards: ActiveRewardView[];
}

export interface TasksData {
  growth_tasks: TaskView[];
  daily_tasks: TaskView[];
  tasks: TaskView[];
  actives: ActiveView[];
}

// --- Config ---

export interface AutomationConfig {
  // Strategy
  planting_strategy: PlantingStrategy;
  preferred_seed_id: number | null;
  intervals: IntervalConfig;
  friend_quiet_hours: QuietHoursConfig;

  // Farm
  auto_harvest: boolean;
  auto_plant: boolean;
  auto_farm_manage: boolean;
  auto_water: boolean;
  auto_weed: boolean;
  auto_insecticide: boolean;
  fertilizer_strategy: FertilizerStrategy;
  auto_land_upgrade: boolean;
  auto_farm_push: boolean;

  // Sell & Claim
  auto_sell: boolean;
  auto_claim_tasks: boolean;
  auto_claim_emails: boolean;
  auto_free_gifts: boolean;
  auto_share_reward: boolean;
  auto_vip_gift: boolean;
  auto_month_card: boolean;
  auto_open_server_gift: boolean;
  auto_fertilizer_gift: boolean;
  auto_fertilizer_buy: boolean;

  // Social
  auto_steal: boolean;
  auto_help_water: boolean;
  auto_help_weed: boolean;
  auto_help_insecticide: boolean;
  auto_friend_bad: boolean;
  auto_friend_help_exp_limit: boolean;

  friend_blacklist: number[];
}
