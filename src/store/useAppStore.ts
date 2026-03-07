import { create } from "zustand";
import { listen } from "@tauri-apps/api/event";
import type {
  StatusResponse,
  ConnectionStatus,
  FarmView,
  BagView,
  TasksData,
  FriendsData,
} from "../types";
import * as api from "../api";

interface AppState {
  // --- Status ---
  status: StatusResponse | null;
  connection: ConnectionStatus;

  // --- Page data ---
  farm: FarmView | null;
  bag: BagView | null;
  tasks: TasksData | null;
  friends: FriendsData | null;

  // --- Actions ---
  fetchStatus: () => Promise<void>;
  fetchFarm: () => Promise<void>;
  fetchBag: () => Promise<void>;
  fetchTasks: () => Promise<void>;
  fetchFriends: () => Promise<void>;
  setFarm: (updater: (prev: FarmView | null) => FarmView | null) => void;

  initListeners: () => () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  status: null,
  connection: "Disconnected",
  farm: null,
  bag: null,
  tasks: null,
  friends: null,

  fetchStatus: async () => {
    try {
      const s = await api.getStatus();
      set({ status: s, connection: s.connection });
    } catch {
      /* not connected */
    }
  },

  fetchFarm: async () => {
    try {
      set({ farm: (await api.getAllLands()) as FarmView });
    } catch {
      /* ignore */
    }
  },

  fetchBag: async () => {
    try {
      set({ bag: (await api.getBag()) as BagView });
    } catch {
      /* ignore */
    }
  },

  fetchTasks: async () => {
    try {
      set({ tasks: (await api.getTasks()) as TasksData });
    } catch {
      /* ignore */
    }
  },

  fetchFriends: async () => {
    try {
      set({ friends: (await api.getFriends()) as FriendsData });
    } catch {
      /* ignore */
    }
  },

  setFarm: (updater) => set({ farm: updater(get().farm) }),

  initListeners: () => {
    const cleanups: (() => void)[] = [];
    const state = get();

    // Fetch initial status, then all data if already logged in
    state.fetchStatus().then(() => {
      if (get().connection === "LoggedIn") {
        get().fetchFarm();
        get().fetchBag();
        get().fetchTasks();
        get().fetchFriends();
      }
    });

    // Status-changed listener
    listen<StatusResponse>("status-changed", (e) => {
      const prev = get().connection;
      const next = e.payload.connection;
      set({ status: e.payload, connection: next });

      // On login: fetch all data
      if (next === "LoggedIn" && prev !== "LoggedIn") {
        get().fetchFarm();
        get().fetchBag();
        get().fetchTasks();
        get().fetchFriends();
      }
      // On disconnect: clear all data
      if (next === "Disconnected" && prev !== "Disconnected") {
        set({ farm: null, bag: null, tasks: null, friends: null });
      }
    }).then((fn) => cleanups.push(fn));

    // Data-changed listener with per-scope debounce
    const timers: Record<string, ReturnType<typeof setTimeout>> = {};
    listen<string>("data-changed", (e) => {
      const scope = e.payload;
      clearTimeout(timers[scope]);
      timers[scope] = setTimeout(() => {
        const s = get();
        if (scope === "farm") s.fetchFarm();
        else if (scope === "inventory") s.fetchBag();
        else if (scope === "tasks") s.fetchTasks();
      }, 2000);
    }).then((fn) => cleanups.push(fn));

    return () => {
      for (const fn of cleanups) fn();
      for (const t of Object.values(timers)) clearTimeout(t);
    };
  },
}));
