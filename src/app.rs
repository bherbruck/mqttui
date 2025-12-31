use std::collections::HashMap;

use chrono::Utc;
use eframe::egui;

use crate::config::AppConfig;
use crate::mqtt::{ConnectionStatus, MqttConnection, MqttMessage, TopicTree};
use crate::ui::{
    ConnectionForm, ConnectionsGrid, ConnectionsGridAction, FormAction, MessageView, MqttUiTheme,
    PublishAction, PublishPanel, TabBar, TabBarAction, TabInfo, TopicTreeView,
};

#[derive(Default, PartialEq, Clone)]
enum View {
    #[default]
    Home,
    ConnectionForm(String), // Connection ID or empty for new
    Connection(String),     // Connection ID
}

pub struct MqttUiApp {
    config: AppConfig,
    view: View,

    // Active connections (by connection ID)
    connections: HashMap<String, MqttConnection>,
    topic_trees: HashMap<String, TopicTree>,

    // Open tabs
    open_tabs: Vec<String>,
    active_tab: Option<String>,

    // UI state
    connection_form: Option<ConnectionForm>,
    topic_tree_views: HashMap<String, TopicTreeView>,
    message_views: HashMap<String, MessageView>,
    publish_panels: HashMap<String, PublishPanel>,
    show_publish_panel: bool,
    show_connection_details: bool,

    // Selected message for viewer
    selected_messages: HashMap<String, Option<MqttMessage>>,
}

impl Default for MqttUiApp {
    fn default() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let open_tabs = config.last_opened_tabs.clone();

        Self {
            config,
            view: View::Home,
            connections: HashMap::new(),
            topic_trees: HashMap::new(),
            open_tabs,
            active_tab: None,
            connection_form: None,
            topic_tree_views: HashMap::new(),
            message_views: HashMap::new(),
            publish_panels: HashMap::new(),
            show_publish_panel: true,
            show_connection_details: false,
            selected_messages: HashMap::new(),
        }
    }
}

impl MqttUiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Apply custom theme
        MqttUiTheme::apply(&cc.egui_ctx);

        // Request dark mode
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        Self::default()
    }

    fn get_connection_statuses(&self) -> HashMap<String, ConnectionStatus> {
        let mut statuses = HashMap::new();
        for (id, conn) in &self.connections {
            statuses.insert(id.clone(), conn.status.clone());
        }
        statuses
    }

    fn open_connection(&mut self, id: &str) {
        if !self.open_tabs.contains(&id.to_string()) {
            self.open_tabs.push(id.to_string());
        }
        self.active_tab = Some(id.to_string());
        self.view = View::Connection(id.to_string());

        // Initialize UI components if needed
        if !self.topic_tree_views.contains_key(id) {
            self.topic_tree_views
                .insert(id.to_string(), TopicTreeView::new());
        }
        if !self.message_views.contains_key(id) {
            self.message_views
                .insert(id.to_string(), MessageView::new());
        }
        if !self.publish_panels.contains_key(id) {
            self.publish_panels
                .insert(id.to_string(), PublishPanel::new());
        }
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

    fn connect(&mut self, id: &str) {
        if let Some(config) = self.config.get_connection(id).cloned() {
            let mut connection = MqttConnection::new(config);
            connection.connect();
            self.connections.insert(id.to_string(), connection);

            // Update last connected time
            if let Some(cfg) = self.config.get_connection_mut(id) {
                cfg.last_connected = Some(Utc::now());
            }
            self.save_config();
        }
    }

    fn disconnect(&mut self, id: &str) {
        if let Some(conn) = self.connections.get_mut(id) {
            conn.disconnect();
        }
    }

    fn save_config(&mut self) {
        self.config.last_opened_tabs = self.open_tabs.clone();
        let _ = self.config.save();
    }

    fn poll_connections(&mut self) {
        for (id, conn) in self.connections.iter_mut() {
            conn.poll_events();

            // Process new messages into topic tree
            let tree = self.topic_trees.entry(id.clone()).or_default();
            while conn.messages.len() > tree.total_messages {
                if let Some(msg) = conn.messages.get(tree.total_messages).cloned() {
                    tree.insert(msg);
                }
            }
        }
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        let tabs: Vec<TabInfo> = self
            .open_tabs
            .iter()
            .filter_map(|id| {
                self.config.get_connection(id).map(|cfg| TabInfo {
                    id: id.clone(),
                    name: cfg.name.clone(),
                    status: self
                        .connections
                        .get(id)
                        .map(|c| c.status.clone())
                        .unwrap_or(ConnectionStatus::Disconnected),
                })
            })
            .collect();

        if let Some(action) = TabBar::ui(ui, &tabs, self.active_tab.as_deref()) {
            match action {
                TabBarAction::SelectHome => {
                    self.view = View::Home;
                    self.active_tab = None;
                }
                TabBarAction::SelectTab(id) => {
                    self.view = View::Connection(id.clone());
                    self.active_tab = Some(id);
                }
                TabBarAction::CloseTab(id) => {
                    self.close_tab(&id);
                }
                TabBarAction::NewTab => {
                    self.view = View::Home;
                    self.active_tab = None;
                }
                TabBarAction::Feedback => {
                    // Could open browser to feedback page
                }
            }
        }
    }

    fn render_home(&mut self, ui: &mut egui::Ui) {
        let statuses = self.get_connection_statuses();

        if let Some(action) = ConnectionsGrid::ui(ui, &self.config.connections, &statuses) {
            match action {
                ConnectionsGridAction::NewConnection => {
                    self.connection_form = Some(ConnectionForm::default_new());
                    self.view = View::ConnectionForm(String::new());
                }
                ConnectionsGridAction::OpenConnection(id) => {
                    self.open_connection(&id);
                    if !self.connections.contains_key(&id) {
                        self.connect(&id);
                    }
                }
                ConnectionsGridAction::EditConnection(id) => {
                    if let Some(config) = self.config.get_connection(&id).cloned() {
                        self.connection_form = Some(ConnectionForm::new(config));
                        self.view = View::ConnectionForm(id);
                    }
                }
                ConnectionsGridAction::Connect(id) => {
                    self.connect(&id);
                }
                ConnectionsGridAction::Disconnect(id) => {
                    self.disconnect(&id);
                }
                ConnectionsGridAction::DeleteConnection(id) => {
                    self.config.remove_connection(&id);
                    self.connections.remove(&id);
                    self.close_tab(&id);
                    self.save_config();
                }
            }
        }
    }

    fn render_connection_form(&mut self, ui: &mut egui::Ui, editing_id: &str) {
        if let Some(ref mut form) = self.connection_form {
            if let Some(action) = form.ui(ui) {
                match action {
                    FormAction::Save => {
                        if editing_id.is_empty() {
                            self.config.add_connection(form.config.clone());
                        } else {
                            self.config.update_connection(form.config.clone());
                        }
                        self.save_config();
                        self.view = View::Home;
                        self.connection_form = None;
                    }
                    FormAction::Connect => {
                        let config = form.config.clone();
                        let id = config.id.clone();

                        if editing_id.is_empty() {
                            self.config.add_connection(config.clone());
                        } else {
                            self.config.update_connection(config.clone());
                        }
                        self.save_config();

                        self.open_connection(&id);
                        self.connect(&id);
                        self.connection_form = None;
                    }
                }
            }
        }
    }

    fn render_connection_view(&mut self, ui: &mut egui::Ui, id: &str) {
        let Some(config) = self.config.get_connection(id).cloned() else {
            ui.label("Connection not found");
            return;
        };

        let is_connected = self
            .connections
            .get(id)
            .map(|c| c.is_connected())
            .unwrap_or(false);

        // Top bar with connection info
        ui.horizontal(|ui| {
            // Connection details toggle
            if ui
                .button(
                    egui::RichText::new("Connection Details").color(MqttUiTheme::TEXT_SECONDARY),
                )
                .clicked()
            {
                self.show_connection_details = !self.show_connection_details;
            }

            ui.separator();

            // Connection status and URI
            let status = self
                .connections
                .get(id)
                .map(|c| c.status.clone())
                .unwrap_or(ConnectionStatus::Disconnected);

            ui.colored_label(status.color(), "\u{25CF}");
            ui.label(
                egui::RichText::new(&config.name)
                    .color(MqttUiTheme::TEXT_PRIMARY)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(config.uri())
                    .color(MqttUiTheme::TEXT_MUTED)
                    .small(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Connect/Disconnect button
                if is_connected
                    && ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Disconnect").color(MqttUiTheme::TEXT_PRIMARY),
                            )
                            .fill(MqttUiTheme::BG_LIGHT),
                        )
                        .clicked()
                {
                    self.disconnect(id);
                } else if !is_connected
                    && ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("\u{25B6} Connect")
                                    .color(MqttUiTheme::TEXT_PRIMARY),
                            )
                            .fill(MqttUiTheme::ACCENT_PRIMARY),
                        )
                        .clicked()
                {
                    self.connect(id);
                }
            });
        });

        ui.add_space(8.0);

        // Show connection form if details panel is open
        if self.show_connection_details {
            egui::Frame::none()
                .fill(MqttUiTheme::BG_MEDIUM)
                .rounding(8.0)
                .inner_margin(16.0)
                .show(ui, |ui| {
                    if self.connection_form.is_none() {
                        self.connection_form = Some(ConnectionForm::new(config.clone()));
                    }
                    if let Some(ref mut form) = self.connection_form {
                        if let Some(action) = form.ui(ui) {
                            match action {
                                FormAction::Save => {
                                    self.config.update_connection(form.config.clone());
                                    self.save_config();
                                    self.show_connection_details = false;
                                }
                                FormAction::Connect => {
                                    self.config.update_connection(form.config.clone());
                                    self.save_config();
                                    self.connect(id);
                                    self.show_connection_details = false;
                                }
                            }
                        }
                    }
                });
            ui.add_space(8.0);
        }

        // Main content area - three panel layout
        let id_string = id.to_string();

        egui::SidePanel::left(format!("publish_panel_{}", id))
            .resizable(true)
            .default_width(250.0)
            .min_width(200.0)
            .max_width(400.0)
            .show_inside(ui, |ui| {
                let panel = self.publish_panels.entry(id_string.clone()).or_default();

                if let Some(action) = panel.ui(ui, is_connected) {
                    match action {
                        PublishAction::Send {
                            topic,
                            payload,
                            qos,
                            retain,
                        } => {
                            if let Some(conn) = self.connections.get_mut(&id_string) {
                                conn.publish(&topic, &payload, qos, retain);
                            }
                        }
                        PublishAction::TogglePanel => {
                            self.show_publish_panel = !self.show_publish_panel;
                        }
                    }
                }
            });

        egui::SidePanel::right(format!("message_panel_{}", id))
            .resizable(true)
            .default_width(400.0)
            .min_width(300.0)
            .max_width(600.0)
            .show_inside(ui, |ui| {
                let message_view = self.message_views.entry(id_string.clone()).or_default();

                let selected_msg = self.selected_messages.get(&id_string).cloned().flatten();
                let selected_topic = self
                    .topic_tree_views
                    .get(&id_string)
                    .and_then(|v| v.selected_topic.clone());

                message_view.ui(ui, selected_msg.as_ref(), selected_topic.as_deref());
            });

        // Central panel - Topic tree
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let tree = self.topic_trees.entry(id_string.clone()).or_default();
            let tree_view = self.topic_tree_views.entry(id_string.clone()).or_default();

            if let Some(msg) = tree_view.ui(ui, &mut tree.root) {
                self.selected_messages.insert(id_string.clone(), Some(msg));
            }
        });
    }
}

impl eframe::App for MqttUiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll connections for new events
        self.poll_connections();

        // Request repaint for real-time updates
        ctx.request_repaint();

        // Top panel with tabs
        egui::TopBottomPanel::top("tab_bar")
            .frame(
                egui::Frame::none()
                    .fill(MqttUiTheme::BG_DARK)
                    .inner_margin(egui::Margin::symmetric(8.0, 4.0)),
            )
            .show(ctx, |ui| {
                self.render_tab_bar(ui);
            });

        // Main content
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(MqttUiTheme::BG_DARK)
                    .inner_margin(16.0),
            )
            .show(ctx, |ui| match self.view.clone() {
                View::Home => self.render_home(ui),
                View::ConnectionForm(id) => self.render_connection_form(ui, &id),
                View::Connection(id) => self.render_connection_view(ui, &id),
            });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.save_config();
    }
}
