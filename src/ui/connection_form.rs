use egui::{RichText, Ui};

use crate::config::{ConnectionConfig, MqttProtocol, MqttVersion, Subscription};

use super::theme::{section_header, styled_button, styled_text_edit, MqttUiTheme};

pub struct ConnectionForm {
    pub config: ConnectionConfig,
    pub port_string: String,
    show_password: bool,
}

impl ConnectionForm {
    pub fn new(config: ConnectionConfig) -> Self {
        let port_string = config.port.to_string();
        Self {
            config,
            port_string,
            show_password: false,
        }
    }

    pub fn default_new() -> Self {
        Self::new(ConnectionConfig::default())
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Option<FormAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("Connection Details")
                        .color(MqttUiTheme::TEXT_PRIMARY)
                        .size(18.0),
                );
                ui.add_space(8.0);
                if ui
                    .button(RichText::new("\u{2699}").size(16.0))
                    .on_hover_text("Settings")
                    .clicked()
                {
                    // TODO: Advanced settings
                }
            });

            ui.add_space(16.0);

            // Name and Version row
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    section_header(ui, "Name");
                    ui.set_min_width(300.0);
                    styled_text_edit(ui, &mut self.config.name, "Connection name");
                });

                ui.add_space(16.0);

                ui.vertical(|ui| {
                    section_header(ui, "Version");
                    egui::ComboBox::from_id_salt("version_combo")
                        .selected_text(self.config.version.as_str())
                        .show_ui(ui, |ui| {
                            for version in MqttVersion::all() {
                                ui.selectable_value(
                                    &mut self.config.version,
                                    version.clone(),
                                    version.as_str(),
                                );
                            }
                        });
                });
            });

            ui.add_space(12.0);

            // Protocol, Host, Port row
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    section_header(ui, "Protocol");
                    egui::ComboBox::from_id_salt("protocol_combo")
                        .selected_text(self.config.protocol.as_str())
                        .show_ui(ui, |ui| {
                            for protocol in MqttProtocol::all() {
                                if ui
                                    .selectable_value(
                                        &mut self.config.protocol,
                                        protocol.clone(),
                                        protocol.as_str(),
                                    )
                                    .changed()
                                {
                                    self.config.port = protocol.default_port();
                                    self.port_string = self.config.port.to_string();
                                }
                            }
                        });
                });

                ui.add_space(16.0);

                ui.vertical(|ui| {
                    section_header(ui, "Host");
                    ui.set_min_width(200.0);
                    styled_text_edit(ui, &mut self.config.host, "localhost");
                });

                ui.add_space(16.0);

                ui.vertical(|ui| {
                    section_header(ui, "Port");
                    ui.set_min_width(80.0);
                    if styled_text_edit(ui, &mut self.port_string, "1883").changed() {
                        if let Ok(port) = self.port_string.parse() {
                            self.config.port = port;
                        }
                    }
                });
            });

            ui.add_space(12.0);

            // Username and Password row
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    section_header(ui, "Username");
                    ui.set_min_width(200.0);
                    let mut username = self.config.username.clone().unwrap_or_default();
                    if styled_text_edit(ui, &mut username, "Username").changed() {
                        self.config.username = if username.is_empty() {
                            None
                        } else {
                            Some(username)
                        };
                    }
                });

                ui.add_space(16.0);

                ui.vertical(|ui| {
                    section_header(ui, "Password");
                    ui.set_min_width(200.0);
                    let mut password = self.config.password.clone().unwrap_or_default();
                    ui.horizontal(|ui| {
                        let text_edit = if self.show_password {
                            egui::TextEdit::singleline(&mut password)
                        } else {
                            egui::TextEdit::singleline(&mut password).password(true)
                        };
                        if ui.add(text_edit.hint_text("Password")).changed() {
                            self.config.password = if password.is_empty() {
                                None
                            } else {
                                Some(password.clone())
                            };
                        }
                        if ui
                            .button(if self.show_password { "\u{1F441}" } else { "\u{1F441}\u{0336}" })
                            .clicked()
                        {
                            self.show_password = !self.show_password;
                        }
                    });
                });
            });

            ui.add_space(16.0);

            // Toggle options
            ui.horizontal(|ui| {
                if ui
                    .checkbox(&mut self.config.use_custom_client_id, "")
                    .changed()
                    && !self.config.use_custom_client_id
                {
                    self.config.client_id = None;
                }
                ui.label(RichText::new("Use custom Client ID").color(MqttUiTheme::TEXT_PRIMARY));
            });

            if self.config.use_custom_client_id {
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    let mut client_id = self.config.client_id.clone().unwrap_or_default();
                    if styled_text_edit(ui, &mut client_id, "Client ID").changed() {
                        self.config.client_id = Some(client_id);
                    }
                });
            }

            ui.add_space(24.0);

            // Subscriptions section
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("Subscriptions")
                        .color(MqttUiTheme::TEXT_PRIMARY)
                        .size(16.0),
                );
            });

            ui.add_space(8.0);

            let mut to_remove = None;
            for (i, sub) in self.config.subscriptions.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    // QoS selector
                    ui.vertical(|ui| {
                        if i == 0 {
                            section_header(ui, "QoS");
                        }
                        egui::ComboBox::from_id_salt(format!("qos_{}", i))
                            .width(50.0)
                            .selected_text(sub.qos.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut sub.qos, 0, "0");
                                ui.selectable_value(&mut sub.qos, 1, "1");
                                ui.selectable_value(&mut sub.qos, 2, "2");
                            });
                    });

                    ui.add_space(8.0);

                    // Topic input
                    ui.vertical(|ui| {
                        if i == 0 {
                            section_header(ui, "Topic");
                        }
                        ui.set_min_width(400.0);
                        styled_text_edit(ui, &mut sub.topic, "#");
                    });

                    ui.add_space(8.0);

                    // Remove button
                    ui.vertical(|ui| {
                        if i == 0 {
                            ui.add_space(24.0);
                        }
                        if ui
                            .button(RichText::new("\u{2715}").color(MqttUiTheme::TEXT_MUTED))
                            .on_hover_text("Remove subscription")
                            .clicked()
                        {
                            to_remove = Some(i);
                        }
                    });
                });
            }

            if let Some(idx) = to_remove {
                if self.config.subscriptions.len() > 1 {
                    self.config.subscriptions.remove(idx);
                }
            }

            ui.add_space(8.0);

            // Add subscription button
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("+ Add Subscription").color(MqttUiTheme::ACCENT_PRIMARY),
                    )
                    .fill(MqttUiTheme::BG_LIGHT)
                    .stroke(egui::Stroke::new(1.0, MqttUiTheme::BORDER)),
                )
                .clicked()
            {
                self.config.subscriptions.push(Subscription::default());
            }

            ui.add_space(24.0);

            // Action buttons
            ui.horizontal(|ui| {
                if styled_button(ui, "Save", false).clicked() {
                    action = Some(FormAction::Save);
                }

                ui.add_space(8.0);

                if styled_button(ui, "Connect", true).clicked() {
                    action = Some(FormAction::Connect);
                }
            });
        });

        action
    }
}

pub enum FormAction {
    Save,
    Connect,
}
