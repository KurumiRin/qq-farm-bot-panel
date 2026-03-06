use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use prost::Message;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_tungstenite::tungstenite;

use crate::config::{self, item_ids};
use crate::error::{AppError, AppResult};
use crate::proto::{gatepb, userpb, plantpb, friendpb};
use crate::state::{AppState, ConnectionStatus};

use super::codec::{self, ServiceRoute};

type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    tungstenite::Message,
>;

type PendingCallback = oneshot::Sender<Result<(Vec<u8>, gatepb::Meta), AppError>>;

/// Event types emitted by the network layer
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    Kickout { reason: String },
    LandsChanged { lands: Vec<plantpb::LandInfo> },
    BasicNotify { level: i64, gold: i64, exp: i64 },
    FriendApplicationReceived,
    FriendAdded { names: Vec<String> },
    TaskInfoNotify,
    GoodsUnlockNotify,
}

pub struct NetworkManager {
    state: Arc<AppState>,
    client_seq: AtomicI64,
    server_seq: AtomicI64,
    pending: Arc<DashMap<i64, PendingCallback>>,
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    connected: AtomicBool,
    event_tx: mpsc::UnboundedSender<NetworkEvent>,
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<NetworkEvent>>>,
}

impl NetworkManager {
    pub fn new(state: Arc<AppState>) -> Arc<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Arc::new(Self {
            state,
            client_seq: AtomicI64::new(1),
            server_seq: AtomicI64::new(0),
            pending: Arc::new(DashMap::new()),
            ws_sink: Arc::new(Mutex::new(None)),
            connected: AtomicBool::new(false),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
        })
    }

    pub fn event_sender(&self) -> mpsc::UnboundedSender<NetworkEvent> {
        self.event_tx.clone()
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// Connect to the game server and start processing messages
    pub async fn connect(self: &Arc<Self>, code: &str) -> AppResult<()> {
        self.state.set_connection_status(ConnectionStatus::Connecting);

        let url = format!(
            "{}?platform={}&os={}&ver={}&code={}&openID=",
            config::SERVER_URL,
            config::PLATFORM,
            config::OS,
            config::CLIENT_VERSION,
            code,
        );

        let request = tungstenite::http::Request::builder()
            .uri(&url)
            .header("User-Agent", config::USER_AGENT)
            .header("Origin", config::ORIGIN)
            .header("Sec-WebSocket-Key", tungstenite::handshake::client::generate_key())
            .header("Sec-WebSocket-Version", "13")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Host", "gate-obt.nqf.qq.com")
            .body(())
            .map_err(|e| AppError::WebSocket(e.to_string()))?;

        let (ws_stream, _response) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| AppError::WebSocket(e.to_string()))?;

        let (sink, stream) = ws_stream.split();
        *self.ws_sink.lock().await = Some(sink);
        self.connected.store(true, Ordering::Relaxed);

        self.state.set_connection_status(ConnectionStatus::Connected);
        log::info!("[WS] Connected to server");

        // Start message processing loop
        let self_clone = Arc::clone(self);
        tokio::spawn(async move {
            self_clone.read_loop(stream).await;
        });

        Ok(())
    }

    /// Send login request after connection
    pub async fn send_login(self: &Arc<Self>) -> AppResult<()> {
        self.state.set_connection_status(ConnectionStatus::LoggingIn);

        let req = userpb::LoginRequest {
            sharer_id: 0,
            sharer_open_id: String::new(),
            device_info: Some(userpb::DeviceInfo {
                client_version: config::CLIENT_VERSION.to_string(),
                sys_software: "iOS 26.2.1".to_string(),
                network: "wifi".to_string(),
                memory: 7672,
                device_id: "iPhone X<iPhone18,3>".to_string(),
                ..Default::default()
            }),
            share_cfg_id: 0,
            scene_id: "1256".to_string(),
            report_data: Some(userpb::ReportData {
                minigame_channel: "other".to_string(),
                minigame_platid: 2,
                ..Default::default()
            }),
        };

        let reply_bytes = self.send_request(&codec::LOGIN, req.encode_to_vec()).await?;
        let reply = userpb::LoginReply::decode(reply_bytes.as_slice())?;

        if let Some(basic) = &reply.basic {
            log::info!(
                "Login reply basic: name={} lv={} gid={} gold={} exp={}",
                basic.name, basic.level, basic.gid, basic.gold, basic.exp
            );

            self.state.update_user_from_login(
                basic.gid,
                basic.name.clone(),
                basic.level,
                basic.gold,
                basic.exp,
                basic.avatar_url.clone(),
            );

            if reply.time_now_millis > 0 {
                self.state.sync_server_time(reply.time_now_millis);
            }

            self.state.set_connection_status(ConnectionStatus::LoggedIn);
        }

        Ok(())
    }

    /// Send a protobuf request and wait for response
    pub async fn send_request(
        &self,
        route: &ServiceRoute,
        body: Vec<u8>,
    ) -> AppResult<Vec<u8>> {
        if !self.connected.load(Ordering::Relaxed) {
            return Err(AppError::NotConnected);
        }

        let seq = self.client_seq.fetch_add(1, Ordering::Relaxed);
        let server_seq = self.server_seq.load(Ordering::Relaxed);

        let frame = codec::encode_gate_message(route, body, seq, server_seq);

        let (tx, rx) = oneshot::channel();
        self.pending.insert(seq, tx);

        // Send the frame
        {
            let mut sink_guard = self.ws_sink.lock().await;
            if let Some(sink) = sink_guard.as_mut() {
                sink.send(tungstenite::Message::Binary(frame.into()))
                    .await
                    .map_err(|e| {
                        self.pending.remove(&seq);
                        AppError::WebSocket(e.to_string())
                    })?;
            } else {
                self.pending.remove(&seq);
                return Err(AppError::NotConnected);
            }
        }

        // Wait for response with timeout
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(config::REQUEST_TIMEOUT_MS),
            rx,
        )
        .await
        .map_err(|_| {
            self.pending.remove(&seq);
            AppError::Timeout(format!("{}.{}", route.service, route.method))
        })?
        .map_err(|_| AppError::Other("Response channel closed".into()))?;

        match result {
            Ok((body, _meta)) => Ok(body),
            Err(e) => Err(e),
        }
    }

    /// Send heartbeat
    pub async fn send_heartbeat(&self) -> AppResult<()> {
        let gid = self.state.user_gid();
        let req = userpb::HeartbeatRequest {
            gid,
            client_version: config::CLIENT_VERSION.to_string(),
        };

        let reply_bytes = self
            .send_request(&codec::HEARTBEAT, req.encode_to_vec())
            .await?;
        let reply = userpb::HeartbeatReply::decode(reply_bytes.as_slice())?;

        if reply.server_time > 0 {
            self.state.sync_server_time(reply.server_time);
        }

        Ok(())
    }

    /// Start heartbeat loop
    pub fn start_heartbeat(self: &Arc<Self>) {
        let self_clone = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_millis(config::HEARTBEAT_INTERVAL_MS));
            loop {
                interval.tick().await;
                if !self_clone.is_connected() {
                    break;
                }
                if let Err(e) = self_clone.send_heartbeat().await {
                    log::warn!("Heartbeat failed: {}", e);
                }
            }
        });
    }

    /// Disconnect and clean up
    pub async fn disconnect(&self) {
        self.connected.store(false, Ordering::Relaxed);

        // Close WebSocket
        if let Some(mut sink) = self.ws_sink.lock().await.take() {
            let _ = sink.close().await;
        }

        // Cancel all pending requests (dropping senders closes channels)
        self.pending.clear();

        self.state.set_connection_status(ConnectionStatus::Disconnected);
        log::info!("[WS] Disconnected");
    }

    /// WebSocket read loop - processes incoming messages
    async fn read_loop(
        self: Arc<Self>,
        mut stream: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    ) {
        while let Some(msg_result) = stream.next().await {
            match msg_result {
                Ok(tungstenite::Message::Binary(data)) => {
                    self.handle_message(&data);
                }
                Ok(tungstenite::Message::Close(_)) => {
                    log::info!("[WS] Connection closed by server");
                    break;
                }
                Err(e) => {
                    log::error!("[WS] Read error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        self.connected.store(false, Ordering::Relaxed);
        self.state.set_connection_status(ConnectionStatus::Disconnected);
    }

    /// Handle an incoming WebSocket message
    fn handle_message(&self, data: &[u8]) {
        let msg = match codec::decode_gate_message(data) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to decode gate message: {}", e);
                return;
            }
        };

        let meta = match &msg.meta {
            Some(m) => m,
            None => return,
        };

        // Update server_seq
        if meta.server_seq > 0 {
            self.server_seq
                .fetch_max(meta.server_seq, Ordering::Relaxed);
        }

        match meta.message_type {
            // Notify (3)
            3 => self.handle_notify(&msg),
            // Response (2)
            2 => {
                let error_code = meta.error_code;
                let client_seq = meta.client_seq;

                if let Some((_, tx)) = self.pending.remove(&client_seq) {
                    if error_code != 0 {
                        let _ = tx.send(Err(AppError::Server {
                            code: error_code,
                            message: meta.error_message.clone(),
                        }));
                    } else {
                        let _ = tx.send(Ok((msg.body, meta.clone())));
                    }
                } else if error_code != 0 {
                    log::warn!(
                        "Unmatched error response: {}.{} code={}",
                        meta.service_name,
                        meta.method_name,
                        error_code
                    );
                }
            }
            _ => {}
        }
    }

    /// Handle server push notifications
    fn handle_notify(&self, msg: &gatepb::Message) {
        if msg.body.is_empty() {
            return;
        }

        let event = match gatepb::EventMessage::decode(msg.body.as_slice()) {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Failed to decode EventMessage: {}", e);
                return;
            }
        };

        let msg_type = &event.message_type;

        if msg_type.contains("Kickout") {
            if let Ok(notify) = crate::proto::gatepb::KickoutNotify::decode(event.body.as_slice()) {
                let reason = if notify.reason_message.is_empty() {
                    "未知".to_string()
                } else {
                    notify.reason_message
                };
                log::warn!("Kicked out: {}", reason);
                let _ = self.event_tx.send(NetworkEvent::Kickout { reason });
            }
        } else if msg_type.contains("LandsNotify") {
            if let Ok(notify) = plantpb::LandsNotify::decode(event.body.as_slice()) {
                let host_gid = notify.host_gid;
                let my_gid = self.state.user_gid();
                if !notify.lands.is_empty() && (host_gid == my_gid || host_gid == 0) {
                    let _ = self.event_tx.send(NetworkEvent::LandsChanged {
                        lands: notify.lands,
                    });
                }
            }
        } else if msg_type.contains("ItemNotify") {
            if let Ok(notify) = crate::proto::itempb::ItemNotify::decode(event.body.as_slice()) {
                let mut changed = false;
                for item_chg in &notify.items {
                    if let Some(item) = &item_chg.item {
                        let id = item.id;
                        let count = item.count;
                        let delta = item_chg.delta;
                        log::debug!("[ItemNotify] id={} count={} delta={}", id, count, delta);
                        let mut user = self.state.user.write();
                        match id {
                            item_ids::EXP_ITEM => {
                                if count > 0 { user.exp = count; changed = true; }
                                else if delta != 0 { user.exp = (user.exp + delta).max(0); changed = true; }
                            }
                            item_ids::GOLD | item_ids::GOLD_ALT => {
                                if count > 0 { user.gold = count; changed = true; }
                                else if delta != 0 { user.gold = (user.gold + delta).max(0); changed = true; }
                            }
                            item_ids::COUPON => {
                                if count > 0 { user.coupon = count; changed = true; }
                                else if delta != 0 { user.coupon = (user.coupon + delta).max(0); changed = true; }
                            }
                            _ => {}
                        }
                    }
                }
                if changed {
                    self.state.emit_status();
                }
            }
        } else if msg_type.contains("BasicNotify") {
            if let Ok(notify) = userpb::BasicNotify::decode(event.body.as_slice()) {
                if let Some(basic) = &notify.basic {
                    log::debug!("[BasicNotify] level={} gold={} exp={}", basic.level, basic.gold, basic.exp);
                    {
                        let mut user = self.state.user.write();
                        if basic.level > 0 { user.level = basic.level; }
                        if basic.gold > 0 { user.gold = basic.gold; }
                        if basic.exp > 0 { user.exp = basic.exp; }
                        let _ = self.event_tx.send(NetworkEvent::BasicNotify {
                            level: user.level,
                            gold: user.gold,
                            exp: user.exp,
                        });
                    }
                    self.state.emit_status();
                }
            }
        } else if msg_type.contains("FriendApplicationReceived") {
            let _ = self.event_tx.send(NetworkEvent::FriendApplicationReceived);
        } else if msg_type.contains("FriendAdded") {
            if let Ok(notify) = friendpb::FriendAddedNotify::decode(event.body.as_slice()) {
                let names: Vec<String> = notify
                    .friends
                    .iter()
                    .map(|f| {
                        if !f.name.is_empty() { f.name.clone() }
                        else { format!("GID:{}", f.gid) }
                    })
                    .collect();
                log::info!("New friends: {}", names.join(", "));
                let _ = self.event_tx.send(NetworkEvent::FriendAdded { names });
            }
        } else if msg_type.contains("GoodsUnlock") {
            let _ = self.event_tx.send(NetworkEvent::GoodsUnlockNotify);
        } else if msg_type.contains("TaskInfo") {
            let _ = self.event_tx.send(NetworkEvent::TaskInfoNotify);
        }
    }
}
