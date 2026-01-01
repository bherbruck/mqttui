//! Main application module for MQTT UI

mod mqtt_worker;
mod types;
mod views;

use std::collections::HashMap;
use std::panic;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use iced::widget::{column, pane_grid};
use iced::{time, Element, Length, Subscription, Task, Theme};

use crate::config::{AppConfig, MqttProtocol, Subscription as MqttSubscription};
use crate::mqtt::{ConnectionStatus, MqttMessage, TopicTree};
use crate::theme;

pub use types::{ConnectionState, MqttCommand, MqttEvent, Pane, View};

#[derive(Debug, Clone)]
#[allow(dead_code, clippy::enum_variant_names)]
pub enum Message {
    // Navigation
    GoHome,
    NewConnection,
    EditConnection(String),
    OpenConnection(String),
    CloseTab(String),
    SelectTab(String),

    // Connection form
    FormNameChanged(String),
    FormHostChanged(String),
    FormPortChanged(String),
    FormClientIdChanged(String),
    FormUsernameChanged(String),
    FormPasswordChanged(String),
    FormProtocolChanged(MqttProtocol),
    FormAddSubscription,
    FormRemoveSubscription(usize),
    FormSubscriptionTopicChanged(usize, String),
    FormSubscriptionQosChanged(usize, u8),
    FormSaveConnection,
    FormConnectAndSave,
    FormCancel,

    // Connection actions
    Connect(String),
    Disconnect(String),
    DeleteConnection(String),

    // MQTT events
    MqttConnected(String),
    MqttDisconnected(String),
    MqttMessage(String, MqttMessage),
    MqttError(String, String),

    // Topic tree
    SelectTopic(String, String),
    ExpandTopic(String, String),
    CollapseTopic(String, String),
    ClearTopics(String),

    // Publish
    PublishTopicChanged(String),
    PublishPayloadChanged(String),
    PublishQosChanged(u8),
    PublishRetainChanged(bool),
    SendMessage,

    // Pane resizing
    PaneResized(pane_grid::ResizeEvent),

    // Tick for polling
    Tick,
}

pub struct MqttUi {
    pub config: AppConfig,
    pub view: View,

    // Connection form state
    pub form_name: String,
    pub form_host: String,
    pub form_port: String,
    pub form_client_id: String,
    pub form_username: String,
    pub form_password: String,
    pub form_protocol: MqttProtocol,
    pub form_subscriptions: Vec<(String, u8)>,

    // Active connections
    pub connections: HashMap<String, ConnectionState>,
    pub topic_trees: HashMap<String, TopicTree>,

    // Open tabs
    pub open_tabs: Vec<String>,
    pub active_tab: Option<String>,

    // Selected topics and messages
    pub selected_topics: HashMap<String, Option<String>>,
    pub selected_messages: HashMap<String, Option<MqttMessage>>,

    // Publish panel state
    pub publish_topic: String,
    pub publish_payload: String,

    // Pane layout
    pub panes: pane_grid::State<Pane>,
    pub publish_qos: u8,
    pub publish_retain: bool,

    // UI throttling - cache tree nodes to avoid rebuilding every frame
    pub cached_tree_nodes: HashMap<String, Vec<types::TreeNodeInfo>>,
    pub tree_cache_dirty: HashMap<String, bool>,
    tick_counter: u32,
}

impl MqttUi {
    pub fn new() -> (Self, Task<Message>) {
        let config = AppConfig::load().unwrap_or_default();
        let open_tabs = config.last_opened_tabs.clone();
        let panes = types::create_pane_layout();

        (
            Self {
                config,
                view: View::Home,
                form_name: String::new(),
                form_host: String::from("localhost"),
                form_port: String::from("1883"),
                form_client_id: String::new(),
                form_username: String::new(),
                form_password: String::new(),
                form_protocol: MqttProtocol::default(),
                form_subscriptions: vec![("#".to_string(), 0)],
                connections: HashMap::new(),
                topic_trees: HashMap::new(),
                open_tabs,
                active_tab: None,
                selected_topics: HashMap::new(),
                selected_messages: HashMap::new(),
                publish_topic: String::new(),
                publish_payload: String::new(),
                panes,
                publish_qos: 0,
                publish_retain: false,
                cached_tree_nodes: HashMap::new(),
                tree_cache_dirty: HashMap::new(),
                tick_counter: 0,
            },
            Task::none(),
        )
    }

    pub fn theme(&self) -> Theme {
        theme::mqtt_theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Just poll for MQTT messages
        time::every(Duration::from_millis(50)).map(|_| Message::Tick)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GoHome => {
                self.view = View::Home;
                self.active_tab = None;
            }

            Message::NewConnection => {
                self.reset_form();
                self.view = View::ConnectionForm { editing_id: None };
            }

            Message::EditConnection(id) => {
                if let Some(config) = self.config.get_connection(&id).cloned() {
                    self.form_name = config.name;
                    self.form_host = config.host;
                    self.form_port = config.port.to_string();
                    self.form_client_id = config.client_id.clone().unwrap_or_default();
                    self.form_username = config.username.clone().unwrap_or_default();
                    self.form_password = config.password.clone().unwrap_or_default();
                    self.form_protocol = config.protocol;
                    self.form_subscriptions = if config.subscriptions.is_empty() {
                        vec![("#".to_string(), 0)]
                    } else {
                        config
                            .subscriptions
                            .iter()
                            .map(|s| (s.topic.clone(), s.qos))
                            .collect()
                    };
                    self.view = View::ConnectionForm {
                        editing_id: Some(id),
                    };
                }
            }

            Message::OpenConnection(id) => {
                self.open_connection(&id);
            }

            Message::CloseTab(id) => {
                self.close_tab(&id);
            }

            Message::SelectTab(id) => {
                self.active_tab = Some(id.clone());
                self.view = View::Connection(id);
            }

            // Form handlers
            Message::FormNameChanged(v) => self.form_name = v,
            Message::FormHostChanged(v) => self.form_host = v,
            Message::FormPortChanged(v) => self.form_port = v,
            Message::FormClientIdChanged(v) => self.form_client_id = v,
            Message::FormUsernameChanged(v) => self.form_username = v,
            Message::FormPasswordChanged(v) => self.form_password = v,
            Message::FormProtocolChanged(v) => self.form_protocol = v,

            Message::FormAddSubscription => {
                self.form_subscriptions.push(("#".to_string(), 0));
            }
            Message::FormRemoveSubscription(idx) => {
                if self.form_subscriptions.len() > 1 {
                    self.form_subscriptions.remove(idx);
                }
            }
            Message::FormSubscriptionTopicChanged(idx, topic) => {
                if let Some(sub) = self.form_subscriptions.get_mut(idx) {
                    sub.0 = topic;
                }
            }
            Message::FormSubscriptionQosChanged(idx, qos) => {
                if let Some(sub) = self.form_subscriptions.get_mut(idx) {
                    sub.1 = qos;
                }
            }

            Message::FormSaveConnection => {
                self.save_form_connection(false);
                self.view = View::Home;
            }

            Message::FormConnectAndSave => {
                let id = self.save_form_connection(true);
                if let Some(id) = id {
                    self.open_connection(&id);
                    self.start_connection(&id);
                }
            }

            Message::FormCancel => {
                self.view = View::Home;
            }

            Message::Connect(id) => {
                self.start_connection(&id);
            }

            Message::Disconnect(id) => {
                self.stop_connection(&id);
            }

            Message::DeleteConnection(id) => {
                self.config.remove_connection(&id);
                self.connections.remove(&id);
                self.close_tab(&id);
                self.save_config();
            }

            Message::MqttConnected(id) => {
                if let Some(conn) = self.connections.get_mut(&id) {
                    conn.status = ConnectionStatus::Connected;
                }
            }

            Message::MqttDisconnected(id) => {
                if let Some(conn) = self.connections.get_mut(&id) {
                    conn.status = ConnectionStatus::Disconnected;
                }
            }

            Message::MqttMessage(id, msg) => {
                // Messages are stored in the topic tree (not conn.messages)
                let tree = self.topic_trees.entry(id.clone()).or_default();
                tree.insert(msg);
                self.tree_cache_dirty.insert(id, true);
            }

            Message::MqttError(id, err) => {
                if let Some(conn) = self.connections.get_mut(&id) {
                    conn.status = ConnectionStatus::Error(err);
                }
            }

            Message::SelectTopic(conn_id, topic) => {
                self.selected_topics
                    .insert(conn_id.clone(), Some(topic.clone()));
                // Find the latest message for this topic
                if let Some(tree) = self.topic_trees.get(&conn_id) {
                    if let Some(msg) = tree.get_latest_message(&topic) {
                        self.selected_messages.insert(conn_id, Some(msg.clone()));
                    }
                }
            }

            Message::ExpandTopic(conn_id, topic) => {
                if let Some(tree) = self.topic_trees.get_mut(&conn_id) {
                    tree.expand(&topic);
                    self.tree_cache_dirty.insert(conn_id, true);
                }
            }

            Message::CollapseTopic(conn_id, topic) => {
                if let Some(tree) = self.topic_trees.get_mut(&conn_id) {
                    tree.collapse(&topic);
                    self.tree_cache_dirty.insert(conn_id, true);
                }
            }

            Message::ClearTopics(conn_id) => {
                self.topic_trees.insert(conn_id.clone(), TopicTree::new());
                self.cached_tree_nodes.remove(&conn_id);
                self.selected_topics.remove(&conn_id);
                self.selected_messages.remove(&conn_id);
            }

            Message::PublishTopicChanged(v) => self.publish_topic = v,
            Message::PublishPayloadChanged(v) => self.publish_payload = v,
            Message::PublishQosChanged(v) => self.publish_qos = v,
            Message::PublishRetainChanged(v) => self.publish_retain = v,

            Message::SendMessage => {
                if let Some(ref id) = self.active_tab {
                    if let Some(conn) = self.connections.get(id) {
                        if let Some(tx) = &conn.command_tx {
                            let _ = tx.send(MqttCommand::Publish(
                                self.publish_topic.clone(),
                                self.publish_payload.as_bytes().to_vec(),
                                self.publish_qos,
                                self.publish_retain,
                            ));
                        }
                    }
                }
            }

            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }

            Message::Tick => {
                self.tick_counter = self.tick_counter.wrapping_add(1);
                self.poll_connections();

                // Rebuild tree caches every 10 ticks (500ms) if dirty
                if self.tick_counter % 10 == 0 {
                    self.rebuild_dirty_caches();
                }
            }
        }

        Task::none()
    }

    fn rebuild_dirty_caches(&mut self) {
        use crate::app::views::topic_tree::collect_tree_nodes_static;

        let dirty_ids: Vec<String> = self.tree_cache_dirty
            .iter()
            .filter(|(_, dirty)| **dirty)
            .map(|(id, _)| id.clone())
            .collect();

        for id in dirty_ids {
            if let Some(tree) = self.topic_trees.get(&id) {
                let nodes = collect_tree_nodes_static(&tree.root, 0);
                self.cached_tree_nodes.insert(id.clone(), nodes);
                self.tree_cache_dirty.insert(id, false);
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = match &self.view {
            View::Home => self.view_home(),
            View::ConnectionForm { editing_id } => self.view_connection_form(editing_id.as_deref()),
            View::Connection(id) => self.view_connection(id),
        };

        let tabs = self.view_tabs();

        column![tabs, content]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // Helper methods
    fn reset_form(&mut self) {
        self.form_name = String::new();
        self.form_host = String::from("localhost");
        self.form_port = String::from("1883");
        self.form_client_id = String::new();
        self.form_username = String::new();
        self.form_password = String::new();
        self.form_protocol = MqttProtocol::default();
        self.form_subscriptions = vec![("#".to_string(), 0)];
    }

    fn save_form_connection(&mut self, _connect: bool) -> Option<String> {
        use crate::config::ConnectionConfig;

        let port = self.form_port.parse().unwrap_or(1883);

        let config = if let View::ConnectionForm {
            editing_id: Some(ref id),
        } = self.view
        {
            let mut config = self.config.get_connection(id).cloned().unwrap_or_default();
            config.name = self.form_name.clone();
            config.host = self.form_host.clone();
            config.port = port;
            config.client_id = if self.form_client_id.is_empty() {
                None
            } else {
                Some(self.form_client_id.clone())
            };
            config.username = if self.form_username.is_empty() {
                None
            } else {
                Some(self.form_username.clone())
            };
            config.password = if self.form_password.is_empty() {
                None
            } else {
                Some(self.form_password.clone())
            };
            config.protocol = self.form_protocol;
            config.use_custom_client_id = !self.form_client_id.is_empty();
            config.subscriptions = self
                .form_subscriptions
                .iter()
                .map(|(topic, qos)| MqttSubscription {
                    topic: topic.clone(),
                    qos: *qos,
                })
                .collect();
            config
        } else {
            ConnectionConfig {
                id: uuid::Uuid::new_v4().to_string(),
                name: self.form_name.clone(),
                host: self.form_host.clone(),
                port,
                protocol: self.form_protocol,
                version: Default::default(),
                client_id: if self.form_client_id.is_empty() {
                    None
                } else {
                    Some(self.form_client_id.clone())
                },
                username: if self.form_username.is_empty() {
                    None
                } else {
                    Some(self.form_username.clone())
                },
                password: if self.form_password.is_empty() {
                    None
                } else {
                    Some(self.form_password.clone())
                },
                use_custom_client_id: !self.form_client_id.is_empty(),
                subscriptions: self
                    .form_subscriptions
                    .iter()
                    .map(|(topic, qos)| MqttSubscription {
                        topic: topic.clone(),
                        qos: *qos,
                    })
                    .collect(),
                created_at: Utc::now(),
                last_connected: None,
            }
        };

        let id = config.id.clone();

        if self.view
            == (View::ConnectionForm {
                editing_id: Some(id.clone()),
            })
        {
            self.config.update_connection(config);
        } else {
            self.config.add_connection(config);
        }

        self.save_config();
        Some(id)
    }

    fn open_connection(&mut self, id: &str) {
        if !self.open_tabs.contains(&id.to_string()) {
            self.open_tabs.push(id.to_string());
        }
        self.active_tab = Some(id.to_string());
        self.view = View::Connection(id.to_string());

        if !self.topic_trees.contains_key(id) {
            self.topic_trees.insert(id.to_string(), TopicTree::new());
        }

        self.save_config();
    }

    fn close_tab(&mut self, id: &str) {
        self.open_tabs.retain(|t| t != id);

        if self.active_tab.as_ref() == Some(&id.to_string()) {
            self.active_tab = self.open_tabs.last().cloned();
            if let Some(ref tab_id) = self.active_tab {
                self.view = View::Connection(tab_id.clone());
            } else {
                self.view = View::Home;
            }
        }

        self.save_config();
    }

    fn start_connection(&mut self, id: &str) {
        if let Some(config) = self.config.get_connection(id).cloned() {
            let (cmd_tx, cmd_rx) = mpsc::channel();
            // Use sync_channel with bounded capacity to prevent memory issues with high message volume
            let (evt_tx, evt_rx) = mpsc::sync_channel(1000);

            let conn_state = ConnectionState {
                config: config.clone(),
                status: ConnectionStatus::Connecting,
                messages: Vec::new(),
                command_tx: Some(cmd_tx),
                event_rx: Some(evt_rx),
            };

            self.connections.insert(id.to_string(), conn_state);

            // Update last connected time
            if let Some(cfg) = self.config.get_connection_mut(id) {
                cfg.last_connected = Some(Utc::now());
            }
            self.save_config();

            // Spawn MQTT worker with panic handling
            let evt_tx_panic = evt_tx.clone();
            thread::spawn(move || {
                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    mqtt_worker::run_mqtt_worker(config, cmd_rx, evt_tx);
                }));
                if let Err(e) = result {
                    let msg = if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic in MQTT worker".to_string()
                    };
                    let _ = evt_tx_panic.try_send(MqttEvent::Error(format!("Worker crashed: {}", msg)));
                }
            });
        }
    }

    fn stop_connection(&mut self, id: &str) {
        if let Some(conn) = self.connections.get_mut(id) {
            if let Some(tx) = &conn.command_tx {
                let _ = tx.send(MqttCommand::Disconnect);
            }
            conn.status = ConnectionStatus::Disconnected;
            conn.command_tx = None;
            conn.event_rx = None;
        }
    }

    fn poll_connections(&mut self) {
        const MAX_MESSAGES_PER_TICK: usize = 50;
        let ids: Vec<String> = self.connections.keys().cloned().collect();

        for id in ids {
            if let Some(conn) = self.connections.get_mut(&id) {
                if let Some(rx) = &conn.event_rx {
                    let mut msg_count = 0;
                    while let Ok(event) = rx.try_recv() {
                        match event {
                            MqttEvent::Connected => {
                                conn.status = ConnectionStatus::Connected;
                            }
                            MqttEvent::Disconnected => {
                                conn.status = ConnectionStatus::Disconnected;
                            }
                            MqttEvent::Message(msg) => {
                                msg_count += 1;
                                // Update selected message if this topic is selected
                                if let Some(Some(selected_topic)) = self.selected_topics.get(&id) {
                                    if selected_topic == &msg.topic {
                                        self.selected_messages.insert(id.clone(), Some(msg.clone()));
                                    }
                                }
                                // Store in topic tree (has per-topic ring buffer of 100 msgs)
                                let tree = self.topic_trees.entry(id.clone()).or_default();
                                tree.insert(msg);
                                // Limit messages per tick to prevent UI overload
                                if msg_count >= MAX_MESSAGES_PER_TICK {
                                    break;
                                }
                            }
                            MqttEvent::Error(e) => {
                                conn.status = ConnectionStatus::Error(e);
                            }
                        }
                    }
                }
            }
        }
    }

    fn save_config(&mut self) {
        self.config.last_opened_tabs = self.open_tabs.clone();
        let _ = self.config.save();
    }
}
