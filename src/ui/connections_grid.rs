use chrono::{DateTime, Utc};
use egui::{RichText, Ui, Vec2};

use crate::config::ConnectionConfig;
use crate::mqtt::ConnectionStatus;

use super::identicon::Identicon;
use super::theme::MqttUiTheme;

pub struct ConnectionsGrid;

impl ConnectionsGrid {
    pub fn ui(
        ui: &mut Ui,
        connections: &[ConnectionConfig],
        connection_statuses: &std::collections::HashMap<String, ConnectionStatus>,
    ) -> Option<ConnectionsGridAction> {
        let mut action = None;

        // Header
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new("Connections")
                    .color(MqttUiTheme::TEXT_PRIMARY)
                    .size(24.0),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("+ Add Connection").color(MqttUiTheme::TEXT_PRIMARY),
                        )
                        .fill(MqttUiTheme::ACCENT_PRIMARY)
                        .rounding(8.0),
                    )
                    .clicked()
                {
                    action = Some(ConnectionsGridAction::NewConnection);
                }
            });
        });

        ui.add_space(16.0);

        // Section label
        ui.label(
            RichText::new("All connections")
                .color(MqttUiTheme::TEXT_SECONDARY)
                .size(14.0),
        );

        ui.add_space(12.0);

        // Connection cards grid
        let available_width = ui.available_width();
        let card_width = 200.0;
        let card_height = 180.0;
        let spacing = 16.0;
        let cards_per_row = ((available_width + spacing) / (card_width + spacing)).floor() as usize;
        let cards_per_row = cards_per_row.max(1);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(spacing);

                    for config in connections {
                        let status = connection_statuses
                            .get(&config.id)
                            .cloned()
                            .unwrap_or(ConnectionStatus::Disconnected);
                        let is_connected = status.is_connected();

                        let response = ui.allocate_ui(Vec2::new(card_width, card_height), |ui| {
                            Self::render_connection_card(ui, config, &status, card_width, card_height);
                        });

                        // Handle click on card
                        if response.response.interact(egui::Sense::click()).clicked() {
                            action = Some(ConnectionsGridAction::OpenConnection(config.id.clone()));
                        }

                        // Context menu
                        response.response.context_menu(|ui| {
                            if ui.button("Edit").clicked() {
                                action = Some(ConnectionsGridAction::EditConnection(config.id.clone()));
                                ui.close_menu();
                            }
                            if is_connected {
                                if ui.button("Disconnect").clicked() {
                                    action = Some(ConnectionsGridAction::Disconnect(config.id.clone()));
                                    ui.close_menu();
                                }
                            } else {
                                if ui.button("Connect").clicked() {
                                    action = Some(ConnectionsGridAction::Connect(config.id.clone()));
                                    ui.close_menu();
                                }
                            }
                            ui.separator();
                            if ui
                                .button(RichText::new("Delete").color(MqttUiTheme::ACCENT_ERROR))
                                .clicked()
                            {
                                action = Some(ConnectionsGridAction::DeleteConnection(config.id.clone()));
                                ui.close_menu();
                            }
                        });
                    }

                    // Empty state
                    if connections.is_empty() {
                        ui.allocate_ui(Vec2::new(available_width, 200.0), |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(40.0);
                                    ui.label(
                                        RichText::new("No connections yet")
                                            .color(MqttUiTheme::TEXT_MUTED)
                                            .size(18.0),
                                    );
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new("Click \"+ Add Connection\" to get started")
                                            .color(MqttUiTheme::TEXT_MUTED)
                                            .size(14.0),
                                    );
                                });
                            });
                        });
                    }
                });
            });

        action
    }

    fn render_connection_card(
        ui: &mut Ui,
        config: &ConnectionConfig,
        status: &ConnectionStatus,
        width: f32,
        height: f32,
    ) {
        let is_connected = status.is_connected();

        egui::Frame::none()
            .fill(MqttUiTheme::BG_MEDIUM)
            .rounding(12.0)
            .stroke(egui::Stroke::new(
                1.0,
                if is_connected {
                    MqttUiTheme::ACCENT_SUCCESS.gamma_multiply(0.5)
                } else {
                    MqttUiTheme::BORDER
                },
            ))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(width - 32.0, height - 32.0));

                ui.vertical(|ui| {
                    // Status indicator and last connected time
                    ui.horizontal(|ui| {
                        Identicon::draw_status_dot(ui, is_connected);
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(status.text())
                                .color(status.color())
                                .small(),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(last) = &config.last_connected {
                                ui.label(
                                    RichText::new(Self::format_relative_time(last))
                                        .color(MqttUiTheme::TEXT_MUTED)
                                        .small(),
                                );
                                ui.label(
                                    RichText::new("\u{1F551}")
                                        .color(MqttUiTheme::TEXT_MUTED)
                                        .small(),
                                );
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Identicon
                    ui.centered_and_justified(|ui| {
                        Identicon::draw(ui, &config.id, 80.0);
                    });

                    ui.add_space(8.0);

                    // Connection name
                    ui.label(
                        RichText::new(&config.name)
                            .color(MqttUiTheme::TEXT_PRIMARY)
                            .strong()
                            .size(14.0),
                    );

                    // URI
                    ui.label(
                        RichText::new(config.uri())
                            .color(MqttUiTheme::TEXT_MUTED)
                            .small(),
                    );
                });
            });
    }

    fn format_relative_time(dt: &DateTime<Utc>) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(*dt);

        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} min ago", duration.num_minutes())
        } else {
            "Just now".to_string()
        }
    }
}

pub enum ConnectionsGridAction {
    NewConnection,
    OpenConnection(String),
    EditConnection(String),
    Connect(String),
    Disconnect(String),
    DeleteConnection(String),
}
