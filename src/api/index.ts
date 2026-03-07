import { invoke } from "@tauri-apps/api/core";
import type {
  QrCodeResponse,
  LoginStatusResponse,
  MpLoginCodeResponse,
  UserState,
  StatusResponse,
  AutomationConfig,
  LogEntry,
} from "../types";

// Auth
export const requestQrCode = (preset?: string) =>
  invoke<QrCodeResponse>("request_qr_code", { preset });

export const checkQrStatus = (qrsig: string, preset?: string) =>
  invoke<LoginStatusResponse>("check_qr_status", { qrsig, preset });

export const requestMpLoginCode = () =>
  invoke<MpLoginCodeResponse>("request_mp_login_code");

export const connectAndLogin = (code: string) =>
  invoke<UserState>("connect_and_login", { code });

export const disconnect = () => invoke<void>("disconnect");

// Status
export const getStatus = () => invoke<StatusResponse>("get_status");

// Config
export const getAutomationConfig = () =>
  invoke<AutomationConfig>("get_automation_config");

export const setAutomationConfig = (config: AutomationConfig) =>
  invoke<void>("set_automation_config", { config });

// Farm
export const getAllLands = () => invoke<unknown>("get_all_lands");

export const harvest = (landIds: number[]) =>
  invoke<unknown>("harvest", { landIds });

export const waterLands = (landIds: number[]) =>
  invoke<void>("water_lands", { landIds });

export const weedOutLands = (landIds: number[]) =>
  invoke<void>("weed_out_lands", { landIds });

export const insecticideLands = (landIds: number[]) =>
  invoke<void>("insecticide_lands", { landIds });

export const removeDeadPlants = (landIds: number[]) =>
  invoke<void>("remove_dead_plants", { landIds });

export const autoPlantEmpty = (landIds: number[]) =>
  invoke<string>("auto_plant_empty", { landIds });

export const plantSeeds = (seedId: number, landIds: number[]) =>
  invoke<unknown>("plant_seeds", { seedId, landIds });

// Friends
export const getFriends = () => invoke<unknown>("get_friends");

export const visitAndActFriend = (gid: number) =>
  invoke<string>("visit_and_act_friend", { gid });

// Warehouse
export const getBag = () => invoke<unknown>("get_bag");

export const sellAllFruits = () => invoke<string>("sell_all_fruits");

export const sellItem = (itemId: number, count: number) =>
  invoke<string>("sell_item", { itemId, count });

export const useItem = (itemId: number, count: number) =>
  invoke<string>("use_item", { itemId, count });

// Tasks
export const getTasks = () => invoke<unknown>("get_tasks");

export const claimAllTasks = () => invoke<void>("claim_all_tasks");

// Shop
export const getShopInfo = (shopId: number) =>
  invoke<unknown>("get_shop_info", { shopId });

// Logs
export const getLogs = (since?: number) =>
  invoke<LogEntry[]>("get_logs", { since: since ?? null });

export const clearLogs = () => invoke<void>("clear_logs");

// System
export const getLoginCode = () =>
  invoke<string | null>("get_login_code");

export const restartCodeReceiver = () =>
  invoke<void>("restart_code_receiver");
