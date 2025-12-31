use egui::{RichText, Ui};

use super::theme::{styled_button, styled_text_edit, MqttUiTheme};

#[derive(Default, Clone, Copy, PartialEq)]
pub enum PublishFormat {
    #[default]
    Text,
    Json,
    Hex,
    Base64,
}

impl PublishFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            PublishFormat::Text => "Text",
            PublishFormat::Json => "JSON",
            PublishFormat::Hex => "Hex",
            PublishFormat::Base64 => "Base64",
        }
    }

    pub fn all() -> &'static [PublishFormat] {
        &[
            PublishFormat::Text,
            PublishFormat::Json,
            PublishFormat::Hex,
            PublishFormat::Base64,
        ]
    }
}

pub struct PublishPanel {
    pub topic: String,
    pub payload: String,
    pub qos: u8,
    pub retain: bool,
    pub format: PublishFormat,
    pub history: Vec<PublishHistoryEntry>,
}

#[derive(Clone)]
pub struct PublishHistoryEntry {
    pub topic: String,
    pub payload: String,
    pub qos: u8,
    pub retain: bool,
}

impl Default for PublishPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PublishPanel {
    pub fn new() -> Self {
        Self {
            topic: String::new(),
            payload: String::new(),
            qos: 0,
            retain: false,
            format: PublishFormat::Text,
            history: Vec::new(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, connected: bool) -> Option<PublishAction> {
        let mut action = None;

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Publish")
                        .color(MqttUiTheme::TEXT_PRIMARY)
                        .strong()
                        .size(16.0),
                );

                if ui
                    .button(RichText::new("\u{2190}").size(14.0))
                    .on_hover_text("Collapse")
                    .clicked()
                {
                    action = Some(PublishAction::TogglePanel);
                }
            });

            ui.add_space(12.0);

            // Topic input
            ui.label(
                RichText::new("Enter a topic")
                    .color(MqttUiTheme::TEXT_SECONDARY)
                    .small(),
            );
            styled_text_edit(ui, &mut self.topic, "topic/path");

            ui.add_space(12.0);

            // Format and encoding
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Encode")
                        .color(MqttUiTheme::TEXT_SECONDARY)
                        .small(),
                );
                egui::ComboBox::from_id_salt("publish_format")
                    .selected_text(self.format.as_str())
                    .show_ui(ui, |ui| {
                        for format in PublishFormat::all() {
                            ui.selectable_value(&mut self.format, *format, format.as_str());
                        }
                    });

                ui.label(
                    RichText::new("Format")
                        .color(MqttUiTheme::TEXT_SECONDARY)
                        .small(),
                );

                // Format button (for JSON prettify)
                if self.format == PublishFormat::Json
                    && ui.button("\u{2630}").on_hover_text("Format JSON").clicked()
                {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.payload) {
                        if let Ok(formatted) = serde_json::to_string_pretty(&json) {
                            self.payload = formatted;
                        }
                    }
                }
            });

            ui.add_space(8.0);

            // Payload input
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.payload)
                            .hint_text("Message payload...")
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8),
                    );
                });

            ui.add_space(12.0);

            // QoS and Retain options
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("QoS")
                        .color(MqttUiTheme::TEXT_SECONDARY)
                        .small(),
                );
                egui::ComboBox::from_id_salt("publish_qos")
                    .width(50.0)
                    .selected_text(self.qos.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.qos, 0, "0");
                        ui.selectable_value(&mut self.qos, 1, "1");
                        ui.selectable_value(&mut self.qos, 2, "2");
                    });

                ui.add_space(16.0);

                ui.checkbox(&mut self.retain, "");
                ui.label(
                    RichText::new("Don't Retain")
                        .color(MqttUiTheme::TEXT_SECONDARY)
                        .small(),
                );
            });

            ui.add_space(12.0);

            // Send button
            let send_enabled = connected && !self.topic.is_empty();
            ui.add_enabled_ui(send_enabled, |ui| {
                if styled_button(ui, "Send", true).clicked() {
                    // Add to history
                    self.history.push(PublishHistoryEntry {
                        topic: self.topic.clone(),
                        payload: self.payload.clone(),
                        qos: self.qos,
                        retain: self.retain,
                    });

                    // Keep last 20 history entries
                    if self.history.len() > 20 {
                        self.history.remove(0);
                    }

                    action = Some(PublishAction::Send {
                        topic: self.topic.clone(),
                        payload: self.encode_payload(),
                        qos: self.qos,
                        retain: self.retain,
                    });
                }
            });

            if !send_enabled && !connected {
                ui.label(
                    RichText::new("Connect to publish")
                        .color(MqttUiTheme::TEXT_MUTED)
                        .small()
                        .italics(),
                );
            }

            // History section
            if !self.history.is_empty() {
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(
                    RichText::new("History")
                        .color(MqttUiTheme::TEXT_SECONDARY)
                        .small(),
                );

                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for (i, entry) in self.history.iter().rev().take(10).enumerate() {
                            ui.push_id(i, |ui| {
                                if ui
                                    .add(
                                        egui::Label::new(
                                            RichText::new(&entry.topic)
                                                .color(MqttUiTheme::TEXT_PRIMARY)
                                                .small(),
                                        )
                                        .sense(egui::Sense::click()),
                                    )
                                    .on_hover_text("Click to restore")
                                    .clicked()
                                {
                                    self.topic = entry.topic.clone();
                                    self.payload = entry.payload.clone();
                                    self.qos = entry.qos;
                                    self.retain = entry.retain;
                                }
                            });
                        }
                    });
            }
        });

        action
    }

    fn encode_payload(&self) -> Vec<u8> {
        match self.format {
            PublishFormat::Text | PublishFormat::Json => self.payload.as_bytes().to_vec(),
            PublishFormat::Hex => self
                .payload
                .split_whitespace()
                .filter_map(|s| u8::from_str_radix(s, 16).ok())
                .collect(),
            PublishFormat::Base64 => {
                use base64::{engine::general_purpose::STANDARD, Engine};
                STANDARD
                    .decode(&self.payload)
                    .unwrap_or_else(|_| self.payload.as_bytes().to_vec())
            }
        }
    }
}

pub enum PublishAction {
    Send {
        topic: String,
        payload: Vec<u8>,
        qos: u8,
        retain: bool,
    },
    TogglePanel,
}
