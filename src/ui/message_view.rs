use egui::{RichText, Ui};

use crate::mqtt::MqttMessage;

use super::theme::MqttUiTheme;

#[derive(Default, Clone, Copy, PartialEq)]
pub enum PayloadFormat {
    #[default]
    Auto,
    Json,
    Text,
    Hex,
    Base64,
}

impl PayloadFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayloadFormat::Auto => "Auto",
            PayloadFormat::Json => "JSON",
            PayloadFormat::Text => "Text",
            PayloadFormat::Hex => "Hex",
            PayloadFormat::Base64 => "Base64",
        }
    }

    pub fn all() -> &'static [PayloadFormat] {
        &[
            PayloadFormat::Auto,
            PayloadFormat::Json,
            PayloadFormat::Text,
            PayloadFormat::Hex,
            PayloadFormat::Base64,
        ]
    }
}

pub struct MessageView {
    pub format: PayloadFormat,
    pub wrap_text: bool,
}

impl Default for MessageView {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageView {
    pub fn new() -> Self {
        Self {
            format: PayloadFormat::Auto,
            wrap_text: true,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, message: Option<&MqttMessage>, topic: Option<&str>) {
        ui.vertical(|ui| {
            // Header with format options
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Message Viewer")
                        .color(MqttUiTheme::TEXT_PRIMARY)
                        .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Wrap toggle
                    ui.checkbox(&mut self.wrap_text, "Wrap");

                    ui.add_space(8.0);

                    // Format selector
                    egui::ComboBox::from_id_salt("format_selector")
                        .selected_text(self.format.as_str())
                        .show_ui(ui, |ui| {
                            for format in PayloadFormat::all() {
                                ui.selectable_value(&mut self.format, *format, format.as_str());
                            }
                        });

                    ui.label(
                        RichText::new("Format:")
                            .color(MqttUiTheme::TEXT_SECONDARY)
                            .small(),
                    );
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            if let Some(msg) = message {
                self.render_message(ui, msg);
            } else if let Some(topic) = topic {
                // Show topic info without message
                ui.label(
                    RichText::new(format!("Topic: {}", topic))
                        .color(MqttUiTheme::TEXT_SECONDARY),
                );
                ui.add_space(16.0);
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("No messages yet")
                            .color(MqttUiTheme::TEXT_MUTED)
                            .italics(),
                    );
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Select a topic to view messages")
                            .color(MqttUiTheme::TEXT_MUTED)
                            .italics(),
                    );
                });
            }
        });
    }

    fn render_message(&mut self, ui: &mut Ui, msg: &MqttMessage) {
        // Message metadata
        ui.horizontal(|ui| {
            ui.label(RichText::new("Topic:").color(MqttUiTheme::TEXT_SECONDARY).small());
            ui.label(RichText::new(&msg.topic).color(MqttUiTheme::ACCENT_INFO).small());
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("QoS:").color(MqttUiTheme::TEXT_SECONDARY).small());
            ui.label(
                RichText::new(msg.qos.to_string())
                    .color(MqttUiTheme::qos_color(msg.qos))
                    .small(),
            );

            ui.add_space(16.0);

            ui.label(RichText::new("Retain:").color(MqttUiTheme::TEXT_SECONDARY).small());
            ui.label(
                RichText::new(if msg.retain { "Yes" } else { "No" })
                    .color(if msg.retain {
                        MqttUiTheme::ACCENT_WARNING
                    } else {
                        MqttUiTheme::TEXT_MUTED
                    })
                    .small(),
            );

            ui.add_space(16.0);

            ui.label(RichText::new("Size:").color(MqttUiTheme::TEXT_SECONDARY).small());
            ui.label(
                RichText::new(format!("{} bytes", msg.payload.len()))
                    .color(MqttUiTheme::TEXT_PRIMARY)
                    .small(),
            );
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Time:").color(MqttUiTheme::TEXT_SECONDARY).small());
            ui.label(
                RichText::new(msg.timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
                    .color(MqttUiTheme::TEXT_PRIMARY)
                    .small(),
            );
        });

        ui.add_space(12.0);

        // Payload
        ui.label(RichText::new("Payload:").color(MqttUiTheme::TEXT_SECONDARY));

        ui.add_space(4.0);

        let payload_text = self.format_payload(msg);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::none()
                    .fill(MqttUiTheme::BG_DARK)
                    .rounding(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        if self.wrap_text {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&payload_text)
                                        .color(MqttUiTheme::TEXT_PRIMARY)
                                        .family(egui::FontFamily::Monospace),
                                )
                                .wrap(),
                            );
                        } else {
                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                ui.label(
                                    RichText::new(&payload_text)
                                        .color(MqttUiTheme::TEXT_PRIMARY)
                                        .family(egui::FontFamily::Monospace),
                                );
                            });
                        }
                    });
            });
    }

    fn format_payload(&self, msg: &MqttMessage) -> String {
        match self.format {
            PayloadFormat::Auto => {
                if msg.is_json() {
                    msg.formatted_payload()
                } else {
                    msg.payload_as_string()
                }
            }
            PayloadFormat::Json => {
                if let Some(json) = msg.payload_as_json() {
                    serde_json::to_string_pretty(&json).unwrap_or_else(|_| msg.payload_as_string())
                } else {
                    "Invalid JSON".to_string()
                }
            }
            PayloadFormat::Text => msg.payload_as_string(),
            PayloadFormat::Hex => {
                msg.payload
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            PayloadFormat::Base64 => {
                use base64::{engine::general_purpose::STANDARD, Engine};
                STANDARD.encode(&msg.payload)
            }
        }
    }
}
