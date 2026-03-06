use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use crate::config::AutomationConfig;

const MAX_LOG_ENTRIES: usize = 500;

/// Global app handle for emitting events from anywhere
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub fn set_app_handle(handle: AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

fn emit<S: Serialize + Clone>(event: &str, payload: S) {
    if let Some(handle) = APP_HANDLE.get() {
        let _ = handle.emit(event, payload);
    }
}

/// Notify frontend that data pages should refresh
pub fn emit_data_changed(scope: &str) {
    emit("data-changed", scope.to_string());
}

/// Per-user state after login
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    pub gid: i64,
    pub name: String,
    pub level: i64,
    pub gold: i64,
    pub exp: i64,
    pub coupon: i64,
    pub avatar_url: String,
}

impl Default for UserState {
    fn default() -> Self {
        Self {
            gid: 0,
            name: String::new(),
            level: 0,
            gold: 0,
            exp: 0,
            coupon: 0,
            avatar_url: String::new(),
        }
    }
}

/// Server time synchronization
#[derive(Debug)]
pub struct TimeSync {
    server_time_ms: i64,
    local_time_ms: i64,
}

impl TimeSync {
    pub fn new() -> Self {
        Self {
            server_time_ms: 0,
            local_time_ms: 0,
        }
    }

    pub fn sync(&mut self, server_time_ms: i64) {
        self.server_time_ms = server_time_ms;
        self.local_time_ms = now_ms();
    }

    pub fn server_now_ms(&self) -> i64 {
        if self.server_time_ms == 0 {
            return now_ms();
        }
        let elapsed = now_ms() - self.local_time_ms;
        self.server_time_ms + elapsed
    }

    pub fn server_now_sec(&self) -> i64 {
        self.server_now_ms() / 1000
    }
}

/// Operation statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub harvests: u64,
    pub plants: u64,
    pub waters: u64,
    pub weeds_removed: u64,
    pub insects_removed: u64,
    pub steals: u64,
    pub help_waters: u64,
    pub help_weeds: u64,
    pub help_insects: u64,
    pub gold_earned: i64,
    pub exp_earned: i64,
    pub items_sold: u64,
    pub tasks_claimed: u64,
    pub emails_claimed: u64,
}

/// Log entry for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: String,
    pub message: String,
}

/// Connection status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    LoggingIn,
    LoggedIn,
    Reconnecting,
    Error(String),
}

/// Status snapshot sent to frontend via events
#[derive(Clone, Serialize)]
pub struct StatusPayload {
    pub user: UserState,
    pub connection: ConnectionStatus,
    pub stats: Stats,
}

/// Global application state shared across all components
#[derive(Debug)]
pub struct AppState {
    pub user: RwLock<UserState>,
    pub time_sync: RwLock<TimeSync>,
    pub stats: RwLock<Stats>,
    pub connection_status: RwLock<ConnectionStatus>,
    pub automation_config: RwLock<AutomationConfig>,
    pub login_code: RwLock<Option<String>>,
    pub logs: RwLock<VecDeque<LogEntry>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            user: RwLock::new(UserState::default()),
            time_sync: RwLock::new(TimeSync::new()),
            stats: RwLock::new(Stats::default()),
            connection_status: RwLock::new(ConnectionStatus::Disconnected),
            automation_config: RwLock::new(AutomationConfig::default()),
            login_code: RwLock::new(None),
            logs: RwLock::new(VecDeque::new()),
        })
    }

    /// Emit current status snapshot to frontend
    pub fn emit_status(&self) {
        emit(
            "status-changed",
            StatusPayload {
                user: self.user.read().clone(),
                connection: self.connection_status.read().clone(),
                stats: self.stats.read().clone(),
            },
        );
    }

    pub fn update_user_from_login(
        &self,
        gid: i64,
        name: String,
        level: i64,
        gold: i64,
        exp: i64,
        avatar_url: String,
    ) {
        let mut user = self.user.write();
        user.gid = gid;
        user.name = name;
        user.level = level;
        user.gold = gold;
        user.exp = exp;
        user.avatar_url = avatar_url;
        drop(user);
        self.emit_status();
    }

    pub fn sync_server_time(&self, server_time_ms: i64) {
        self.time_sync.write().sync(server_time_ms);
    }

    pub fn server_now_sec(&self) -> i64 {
        self.time_sync.read().server_now_sec()
    }

    pub fn record_stat(&self, f: impl FnOnce(&mut Stats)) {
        f(&mut self.stats.write());
        self.emit_status();
    }

    pub fn set_connection_status(&self, status: ConnectionStatus) {
        *self.connection_status.write() = status;
        self.emit_status();
    }

    pub fn is_logged_in(&self) -> bool {
        *self.connection_status.read() == ConnectionStatus::LoggedIn
    }

    pub fn user_gid(&self) -> i64 {
        self.user.read().gid
    }

    pub fn push_log(&self, level: &str, message: impl Into<String>) {
        let entry = LogEntry {
            timestamp: now_ms(),
            level: level.to_string(),
            message: message.into(),
        };
        emit("log-added", entry.clone());
        let mut logs = self.logs.write();
        if logs.len() >= MAX_LOG_ENTRIES {
            logs.pop_front();
        }
        logs.push_back(entry);
    }

    pub fn get_logs(&self, since: Option<i64>) -> Vec<LogEntry> {
        let logs = self.logs.read();
        match since {
            Some(ts) => logs.iter().filter(|l| l.timestamp > ts).cloned().collect(),
            None => logs.iter().cloned().collect(),
        }
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
