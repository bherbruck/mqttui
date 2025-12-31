use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable, text,
    text_input, toggler, vertical_rule, Column, Row,
};
use iced::{time, Color, Element, Length, Subscription, Task, Theme};

use crate::config::{AppConfig, ConnectionConfig, MqttProtocol};
use crate::mqtt::{ConnectionStatus, MqttMessage, TopicTree};
use crate::theme;

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

    // Publish
    PublishTopicChanged(String),
    PublishPayloadChanged(String),
    PublishQosChanged(u8),
    PublishRetainChanged(bool),
    SendMessage,

    // Tick for polling
    Tick,
}

#[derive(Default, PartialEq, Clone)]
enum View {
    #[default]
    Home,
    ConnectionForm {
        editing_id: Option<String>,
    },
    Connection(String),
}

pub struct MqttUi {
    config: AppConfig,
    view: View,

    // Connection form state
    form_name: String,
    form_host: String,
    form_port: String,
    form_client_id: String,
    form_username: String,
    form_password: String,
    form_protocol: MqttProtocol,

    // Active connections
    connections: HashMap<String, ConnectionState>,
    topic_trees: HashMap<String, TopicTree>,

    // Open tabs
    open_tabs: Vec<String>,
    active_tab: Option<String>,

    // Selected topics and messages
    selected_topics: HashMap<String, Option<String>>,
    selected_messages: HashMap<String, Option<MqttMessage>>,

    // Publish panel state
    publish_topic: String,
    publish_payload: String,
    publish_qos: u8,
    publish_retain: bool,
}

#[allow(dead_code)]
struct ConnectionState {
    config: ConnectionConfig,
    status: ConnectionStatus,
    messages: Vec<MqttMessage>,
    command_tx: Option<mpsc::Sender<MqttCommand>>,
    event_rx: Option<mpsc::Receiver<MqttEvent>>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum MqttCommand {
    Connect,
    Disconnect,
    Publish(String, Vec<u8>, u8, bool),
}

#[derive(Debug)]
enum MqttEvent {
    Connected,
    Disconnected,
    Message(MqttMessage),
    Error(String),
}

#[allow(mismatched_lifetime_syntaxes)]
impl MqttUi {
    pub fn new() -> (Self, Task<Message>) {
        let config = AppConfig::load().unwrap_or_default();
        let open_tabs = config.last_opened_tabs.clone();

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
                connections: HashMap::new(),
                topic_trees: HashMap::new(),
                open_tabs,
                active_tab: None,
                selected_topics: HashMap::new(),
                selected_messages: HashMap::new(),
                publish_topic: String::new(),
                publish_payload: String::new(),
                publish_qos: 0,
                publish_retain: false,
            },
            Task::none(),
        )
    }

    pub fn theme(&self) -> Theme {
        theme::mqtt_theme()
    }

    pub fn subscription(&self) -> Subscription<Message> {
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
                if let Some(conn) = self.connections.get_mut(&id) {
                    conn.messages.push(msg.clone());
                }
                let tree = self.topic_trees.entry(id).or_default();
                tree.insert(msg);
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
                }
            }

            Message::CollapseTopic(conn_id, topic) => {
                if let Some(tree) = self.topic_trees.get_mut(&conn_id) {
                    tree.collapse(&topic);
                }
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

            Message::Tick => {
                self.poll_connections();
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<Message> {
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

    fn view_tabs(&self) -> Element<Message> {
        let mut tabs_row = Row::new().spacing(2);

        // Home button
        let home_btn = button(text("⌂").size(16))
            .padding([8, 12])
            .style(if self.active_tab.is_none() {
                button::primary
            } else {
                button::secondary
            })
            .on_press(Message::GoHome);
        tabs_row = tabs_row.push(home_btn);

        // Connection tabs
        for tab_id in &self.open_tabs {
            if let Some(config) = self.config.get_connection(tab_id) {
                let is_active = self.active_tab.as_ref() == Some(tab_id);
                let status = self
                    .connections
                    .get(tab_id)
                    .map(|c| &c.status)
                    .unwrap_or(&ConnectionStatus::Disconnected);

                let status_dot = text(match status {
                    ConnectionStatus::Connected => "●",
                    ConnectionStatus::Connecting => "◐",
                    _ => "○",
                })
                .color(match status {
                    ConnectionStatus::Connected => Color::from_rgb(0.2, 0.8, 0.2),
                    ConnectionStatus::Connecting => Color::from_rgb(0.8, 0.8, 0.2),
                    ConnectionStatus::Error(_) => Color::from_rgb(0.8, 0.2, 0.2),
                    _ => Color::from_rgb(0.5, 0.5, 0.5),
                });

                let tab_content = row![
                    status_dot,
                    text(&config.name).size(14),
                    button(text("×").size(12))
                        .padding([2, 6])
                        .style(button::text)
                        .on_press(Message::CloseTab(tab_id.clone()))
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center);

                let tab_btn = button(tab_content)
                    .padding([8, 12])
                    .style(if is_active {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .on_press(Message::SelectTab(tab_id.clone()));

                tabs_row = tabs_row.push(tab_btn);
            }
        }

        // New tab button
        tabs_row = tabs_row.push(horizontal_space());
        tabs_row = tabs_row.push(
            button(text("+").size(16))
                .padding([8, 12])
                .style(button::secondary)
                .on_press(Message::NewConnection),
        );

        container(tabs_row.padding(8))
            .width(Length::Fill)
            .style(container::bordered_box)
            .into()
    }

    fn view_home(&self) -> Element<Message> {
        let mut content = Column::new().spacing(20).padding(20);

        // Header
        content = content.push(
            row![
                text("Connections").size(28),
                horizontal_space(),
                button(row![text("+").size(16), text(" New Connection")])
                    .padding([10, 16])
                    .style(button::primary)
                    .on_press(Message::NewConnection)
            ]
            .align_y(iced::Alignment::Center),
        );

        content = content.push(horizontal_rule(1));

        // Connection cards
        if self.config.connections.is_empty() {
            content = content.push(
                container(
                    column![
                        text("No connections yet")
                            .size(18)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        text("Click \"+ New Connection\" to get started")
                            .size(14)
                            .color(Color::from_rgb(0.5, 0.5, 0.5))
                    ]
                    .spacing(8)
                    .align_x(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .height(200)
                .center_x(Length::Fill)
                .center_y(200),
            );
        } else {
            let mut cards_row = Row::new().spacing(16);

            for config in &self.config.connections {
                let status = self
                    .connections
                    .get(&config.id)
                    .map(|c| c.status.clone())
                    .unwrap_or(ConnectionStatus::Disconnected);

                let card = self.view_connection_card(config, &status);
                cards_row = cards_row.push(card);
            }

            content = content.push(scrollable(cards_row));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_connection_card(
        &self,
        config: &ConnectionConfig,
        status: &ConnectionStatus,
    ) -> Element<Message> {
        let status_text = match status {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Error(_) => "Error",
        };

        let status_color = match status {
            ConnectionStatus::Connected => Color::from_rgb(0.2, 0.8, 0.2),
            ConnectionStatus::Connecting => Color::from_rgb(0.8, 0.8, 0.2),
            ConnectionStatus::Error(_) => Color::from_rgb(0.8, 0.2, 0.2),
            _ => Color::from_rgb(0.5, 0.5, 0.5),
        };

        let id = config.id.clone();
        let is_connected = status.is_connected();

        let connect_btn = if is_connected {
            button(text("Disconnect").size(12))
                .padding([6, 12])
                .style(button::secondary)
                .on_press(Message::Disconnect(id.clone()))
        } else {
            button(text("Connect").size(12))
                .padding([6, 12])
                .style(button::primary)
                .on_press(Message::Connect(id.clone()))
        };

        let name = config.name.clone();
        let uri = config.uri();

        let card_content = column![
            row![
                text("●").size(10).color(status_color),
                text(status_text).size(12).color(status_color),
            ]
            .spacing(4),
            text(name).size(16),
            text(uri).size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
            row![
                connect_btn,
                button(text("Edit").size(12))
                    .padding([6, 12])
                    .style(button::secondary)
                    .on_press(Message::EditConnection(id.clone())),
                button(text("Delete").size(12))
                    .padding([6, 12])
                    .style(button::danger)
                    .on_press(Message::DeleteConnection(id.clone())),
            ]
            .spacing(8)
        ]
        .spacing(12)
        .padding(16)
        .width(220);

        button(card_content)
            .style(button::secondary)
            .on_press(Message::OpenConnection(id))
            .into()
    }

    fn view_connection_form(&self, editing_id: Option<&str>) -> Element<Message> {
        let title = if editing_id.is_some() {
            "Edit Connection"
        } else {
            "New Connection"
        };

        let protocols: Vec<MqttProtocol> = vec![
            MqttProtocol::Mqtt,
            MqttProtocol::Mqtts,
            MqttProtocol::MqttWs,
            MqttProtocol::MqttsWs,
        ];

        let form = column![
            text(title).size(24),
            horizontal_rule(1),
            // Name
            column![
                text("Name").size(14),
                text_input("Connection name", &self.form_name)
                    .padding(10)
                    .on_input(Message::FormNameChanged)
            ]
            .spacing(4),
            // Host and Port
            row![
                column![
                    text("Host").size(14),
                    text_input("localhost", &self.form_host)
                        .padding(10)
                        .on_input(Message::FormHostChanged)
                ]
                .spacing(4)
                .width(Length::FillPortion(3)),
                column![
                    text("Port").size(14),
                    text_input("1883", &self.form_port)
                        .padding(10)
                        .on_input(Message::FormPortChanged)
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(12),
            // Protocol
            column![
                text("Protocol").size(14),
                pick_list(
                    protocols,
                    Some(self.form_protocol),
                    Message::FormProtocolChanged
                )
                .padding(10)
                .width(Length::Fill)
            ]
            .spacing(4),
            // Client ID
            column![
                text("Client ID (optional)").size(14),
                text_input("Auto-generated if empty", &self.form_client_id)
                    .padding(10)
                    .on_input(Message::FormClientIdChanged)
            ]
            .spacing(4),
            // Username
            column![
                text("Username (optional)").size(14),
                text_input("Username", &self.form_username)
                    .padding(10)
                    .on_input(Message::FormUsernameChanged)
            ]
            .spacing(4),
            // Password
            column![
                text("Password (optional)").size(14),
                text_input("Password", &self.form_password)
                    .padding(10)
                    .secure(true)
                    .on_input(Message::FormPasswordChanged)
            ]
            .spacing(4),
            horizontal_rule(1),
            // Buttons
            row![
                button(text("Cancel"))
                    .padding([10, 20])
                    .style(button::secondary)
                    .on_press(Message::FormCancel),
                horizontal_space(),
                button(text("Save"))
                    .padding([10, 20])
                    .style(button::secondary)
                    .on_press(Message::FormSaveConnection),
                button(text("Connect"))
                    .padding([10, 20])
                    .style(button::primary)
                    .on_press(Message::FormConnectAndSave),
            ]
            .spacing(12)
        ]
        .spacing(16)
        .padding(24)
        .max_width(500);

        container(form)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .padding(20)
            .into()
    }

    fn view_connection(&self, id: &str) -> Element<Message> {
        let Some(conn) = self.connections.get(id) else {
            return self.view_connection_not_started(id);
        };

        let is_connected = conn.status.is_connected();

        // Left panel - Publish
        let publish_panel = self.view_publish_panel(id, is_connected);

        // Center panel - Topic tree
        let topic_panel = self.view_topic_tree(id);

        // Right panel - Message view
        let message_panel = self.view_message_panel(id);

        row![
            container(publish_panel).width(250),
            vertical_rule(1),
            container(topic_panel).width(Length::FillPortion(2)),
            vertical_rule(1),
            container(message_panel).width(Length::FillPortion(3)),
        ]
        .height(Length::Fill)
        .into()
    }

    fn view_connection_not_started(&self, id: &str) -> Element<Message> {
        let Some(config) = self.config.get_connection(id) else {
            return text("Connection not found").into();
        };

        let name = config.name.clone();
        let uri = config.uri();

        container(
            column![
                text(name).size(24),
                text(uri).size(14).color(Color::from_rgb(0.6, 0.6, 0.6)),
                button(text("Connect"))
                    .padding([12, 24])
                    .style(button::primary)
                    .on_press(Message::Connect(id.to_string()))
            ]
            .spacing(16)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn view_publish_panel(&self, _id: &str, is_connected: bool) -> Element<Message> {
        let qos_options = vec![0u8, 1, 2];

        column![
            text("Publish").size(16),
            horizontal_rule(1),
            column![
                text("Topic").size(12),
                text_input("topic/path", &self.publish_topic)
                    .padding(8)
                    .on_input(Message::PublishTopicChanged)
            ]
            .spacing(4),
            column![
                text("Payload").size(12),
                text_input("{\"key\": \"value\"}", &self.publish_payload)
                    .padding(8)
                    .on_input(Message::PublishPayloadChanged)
            ]
            .spacing(4),
            row![
                column![
                    text("QoS").size(12),
                    pick_list(
                        qos_options,
                        Some(self.publish_qos),
                        Message::PublishQosChanged
                    )
                    .padding(8)
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
                column![
                    text("Retain").size(12),
                    toggler(self.publish_retain).on_toggle(Message::PublishRetainChanged)
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(12),
            if is_connected {
                button(text("Send").width(Length::Fill))
                    .padding([10, 20])
                    .width(Length::Fill)
                    .style(button::primary)
                    .on_press(Message::SendMessage)
            } else {
                button(text("Send").width(Length::Fill))
                    .padding([10, 20])
                    .width(Length::Fill)
                    .style(button::secondary)
            }
        ]
        .spacing(12)
        .padding(16)
        .into()
    }

    fn view_topic_tree(&self, id: &str) -> Element<Message> {
        let tree = self.topic_trees.get(id);

        let mut content = Column::new().spacing(4).padding(16);
        content = content.push(text("Topics").size(16));
        content = content.push(horizontal_rule(1));

        if let Some(tree) = tree {
            let topics = tree.get_all_topics();
            if topics.is_empty() {
                content = content.push(
                    text("No messages received yet")
                        .size(14)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                );
            } else {
                let selected = self.selected_topics.get(id).cloned().flatten();
                for topic in topics {
                    let is_selected = selected.as_ref() == Some(&topic);
                    let topic_clone = topic.clone();
                    let topic_btn = button(text(topic).size(13))
                        .padding([6, 10])
                        .width(Length::Fill)
                        .style(if is_selected {
                            button::primary
                        } else {
                            button::text
                        })
                        .on_press(Message::SelectTopic(id.to_string(), topic_clone));
                    content = content.push(topic_btn);
                }
            }
        } else {
            content = content.push(
                text("Waiting for connection...")
                    .size(14)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            );
        }

        scrollable(content).height(Length::Fill).into()
    }

    fn view_message_panel(&self, id: &str) -> Element<Message> {
        let selected_msg = self.selected_messages.get(id).cloned().flatten();

        let mut content = Column::new().spacing(8).padding(16);
        content = content.push(text("Message").size(16));
        content = content.push(horizontal_rule(1));

        if let Some(msg) = selected_msg {
            let topic = msg.topic.clone();
            let qos = msg.qos.to_string();
            let retain = if msg.retain { "Yes" } else { "No" };
            let time = msg.timestamp.format("%H:%M:%S").to_string();
            let payload = msg.formatted_payload();

            content = content.push(
                column![
                    row![
                        text("Topic:")
                            .size(12)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        text(topic).size(12)
                    ]
                    .spacing(8),
                    row![
                        text("QoS:").size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
                        text(qos).size(12),
                        text("Retain:")
                            .size(12)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                        text(retain).size(12),
                    ]
                    .spacing(8),
                    row![
                        text("Time:").size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
                        text(time).size(12)
                    ]
                    .spacing(8),
                    horizontal_rule(1),
                    text("Payload:")
                        .size(12)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    scrollable(
                        container(text(payload).size(13))
                            .padding(12)
                            .style(container::bordered_box)
                    )
                    .height(Length::Fill)
                ]
                .spacing(8),
            );
        } else {
            content = content.push(
                text("Select a topic to view messages")
                    .size(14)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            );
        }

        content.height(Length::Fill).into()
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
    }

    fn save_form_connection(&mut self, _connect: bool) -> Option<String> {
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
                subscriptions: vec![],
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
            let (evt_tx, evt_rx) = mpsc::channel();

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

            // Spawn MQTT worker
            thread::spawn(move || {
                Self::mqtt_worker(config, cmd_rx, evt_tx);
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

    fn mqtt_worker(
        config: ConnectionConfig,
        cmd_rx: mpsc::Receiver<MqttCommand>,
        evt_tx: mpsc::Sender<MqttEvent>,
    ) {
        use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

        let client_id = config.effective_client_id();
        let mut mqttoptions = MqttOptions::new(&client_id, &config.host, config.port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));

        if let (Some(username), password) = (&config.username, &config.password) {
            mqttoptions.set_credentials(username, password.clone().unwrap_or_default());
        }

        let (client, mut connection) = Client::new(mqttoptions, 100);

        // Spawn command handler
        let client_clone = client.clone();
        let subscriptions = config.subscriptions.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(500));

            // Subscribe to configured topics
            for sub in &subscriptions {
                let qos = match sub.qos {
                    0 => QoS::AtMostOnce,
                    1 => QoS::AtLeastOnce,
                    _ => QoS::ExactlyOnce,
                };
                let _ = client_clone.subscribe(&sub.topic, qos);
            }

            // Also subscribe to # to get all messages
            let _ = client_clone.subscribe("#", QoS::AtMostOnce);

            for cmd in cmd_rx {
                match cmd {
                    MqttCommand::Connect => {}
                    MqttCommand::Disconnect => {
                        let _ = client_clone.disconnect();
                        break;
                    }
                    MqttCommand::Publish(topic, payload, qos, retain) => {
                        let qos = match qos {
                            0 => QoS::AtMostOnce,
                            1 => QoS::AtLeastOnce,
                            _ => QoS::ExactlyOnce,
                        };
                        let _ = client_clone.publish(&topic, qos, retain, payload);
                    }
                }
            }
        });

        // Event loop
        for notification in connection.iter() {
            match notification {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    let _ = evt_tx.send(MqttEvent::Connected);
                }
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let msg = MqttMessage::new(
                        publish.topic.to_string(),
                        publish.payload.to_vec(),
                        publish.qos as u8,
                        publish.retain,
                    );
                    let _ = evt_tx.send(MqttEvent::Message(msg));
                }
                Ok(Event::Incoming(Packet::Disconnect)) => {
                    let _ = evt_tx.send(MqttEvent::Disconnected);
                    break;
                }
                Err(e) => {
                    let _ = evt_tx.send(MqttEvent::Error(e.to_string()));
                    break;
                }
                _ => {}
            }
        }

        let _ = evt_tx.send(MqttEvent::Disconnected);
    }

    fn poll_connections(&mut self) {
        let ids: Vec<String> = self.connections.keys().cloned().collect();

        for id in ids {
            if let Some(conn) = self.connections.get_mut(&id) {
                if let Some(rx) = &conn.event_rx {
                    while let Ok(event) = rx.try_recv() {
                        match event {
                            MqttEvent::Connected => {
                                conn.status = ConnectionStatus::Connected;
                            }
                            MqttEvent::Disconnected => {
                                conn.status = ConnectionStatus::Disconnected;
                            }
                            MqttEvent::Message(msg) => {
                                conn.messages.push(msg.clone());
                                let tree = self.topic_trees.entry(id.clone()).or_default();
                                tree.insert(msg);
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
