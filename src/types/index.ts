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
