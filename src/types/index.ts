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

export interface AutomationConfig {
  auto_harvest: boolean;
  auto_plant: boolean;
  auto_water: boolean;
  auto_weed: boolean;
  auto_insecticide: boolean;
  auto_fertilize: boolean;
  auto_sell: boolean;
  auto_steal: boolean;
  auto_help_water: boolean;
  auto_help_weed: boolean;
  auto_help_insecticide: boolean;
  auto_claim_tasks: boolean;
  auto_claim_emails: boolean;
  preferred_seed_id: number | null;
  friend_blacklist: number[];
}
