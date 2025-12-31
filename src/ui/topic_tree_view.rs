use egui::{RichText, Ui};

use crate::mqtt::{MqttMessage, TopicNode};

use super::theme::MqttUiTheme;

pub struct TopicTreeView {
    pub selected_topic: Option<String>,
    pub filter: String,
}

impl Default for TopicTreeView {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicTreeView {
    pub fn new() -> Self {
        Self {
            selected_topic: None,
            filter: String::new(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, root: &mut TopicNode) -> Option<MqttMessage> {
        let mut selected_message = None;

        // Filter bar
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.filter)
                    .hint_text("\u{1F50D} Filter by topic, payload or pattern")
                    .desired_width(ui.available_width() - 100.0),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Sort button (placeholder for future functionality)
                let _ = ui.button(
                    RichText::new("A → Z")
                        .color(MqttUiTheme::TEXT_SECONDARY)
                        .small(),
                );

                // Expand all button
                if ui
                    .button(RichText::new("\u{229E}").color(MqttUiTheme::TEXT_SECONDARY))
                    .on_hover_text("Expand all")
                    .clicked()
                {
                    Self::expand_all(root, true);
                }
            });
        });

        ui.add_space(8.0);

        // Collect children names first to avoid borrow issues
        let children: Vec<(String, String)> = root
            .sorted_children()
            .iter()
            .map(|(name, node)| (name.to_string(), node.full_path.clone()))
            .collect();

        // Tree view
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (name, _path) in &children {
                    if let Some(child) = root.children.get_mut(name) {
                        if self.filter.is_empty() || self.matches_filter_owned(child) {
                            if let Some(msg) = self.render_node(ui, name, child) {
                                selected_message = Some(msg);
                            }
                        }
                    }
                }
            });

        selected_message
    }

    fn matches_filter_owned(&self, node: &TopicNode) -> bool {
        let filter_lower = self.filter.to_lowercase();

        if node.full_path.to_lowercase().contains(&filter_lower) {
            return true;
        }

        if let Some(msg) = node.last_message() {
            if msg
                .payload_as_string()
                .to_lowercase()
                .contains(&filter_lower)
            {
                return true;
            }
        }

        for child in node.children.values() {
            if self.matches_filter_owned(child) {
                return true;
            }
        }

        false
    }

    fn expand_all(node: &mut TopicNode, expanded: bool) {
        node.expanded = expanded;
        for child in node.children.values_mut() {
            Self::expand_all(child, expanded);
        }
    }

    fn render_node(
        &mut self,
        ui: &mut Ui,
        name: &str,
        node: &mut TopicNode,
    ) -> Option<MqttMessage> {
        let mut selected_message = None;
        let has_children = !node.children.is_empty();
        let is_selected = self.selected_topic.as_ref() == Some(&node.full_path);
        let node_path = node.full_path.clone();
        let node_expanded = node.expanded;
        let message_count = node.message_count;
        let children_count = node.children.len();
        let last_msg = node.last_message().cloned();

        let bg_color = if is_selected {
            MqttUiTheme::ACCENT_PRIMARY.gamma_multiply(0.2)
        } else {
            egui::Color32::TRANSPARENT
        };

        ui.push_id(&node_path, |ui| {
            ui.horizontal(|ui| {
                // Background highlight for selected
                let rect = ui.available_rect_before_wrap();
                if is_selected {
                    ui.painter().rect_filled(rect, 4.0, bg_color);
                }

                // Expand/collapse arrow
                if has_children {
                    let arrow = if node_expanded {
                        "\u{25BC}"
                    } else {
                        "\u{25B6}"
                    };
                    if ui
                        .add(
                            egui::Label::new(
                                RichText::new(arrow)
                                    .color(MqttUiTheme::TEXT_MUTED)
                                    .size(10.0),
                            )
                            .sense(egui::Sense::click()),
                        )
                        .clicked()
                    {
                        node.expanded = !node.expanded;
                    }
                } else {
                    ui.add_space(14.0);
                }

                // Topic name
                let name_response = ui.add(
                    egui::Label::new(RichText::new(name).color(MqttUiTheme::TEXT_PRIMARY))
                        .sense(egui::Sense::click()),
                );

                if name_response.clicked() {
                    self.selected_topic = Some(node_path.clone());
                    if has_children {
                        node.expanded = !node.expanded;
                    }
                }

                // Message count badges
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Show payload preview if there are direct messages
                    if let Some(ref msg) = last_msg {
                        let preview = msg.payload_preview(40);
                        ui.add(egui::Label::new(
                            RichText::new(&preview)
                                .color(MqttUiTheme::TEXT_MUTED)
                                .small()
                                .family(egui::FontFamily::Monospace),
                        ));
                    }

                    // Total message count for this branch
                    if message_count > 0 {
                        ui.add(egui::Label::new(
                            RichText::new(format!("\u{2709} {}", message_count))
                                .color(MqttUiTheme::TEXT_SECONDARY)
                                .small(),
                        ));
                    }

                    // Children count
                    if has_children {
                        ui.add(egui::Label::new(
                            RichText::new(format!("\u{2193} {}", children_count))
                                .color(MqttUiTheme::TEXT_MUTED)
                                .small(),
                        ));
                    }
                });
            });

            // Set selected message if this node is selected and has messages
            if is_selected {
                if let Some(msg) = last_msg {
                    selected_message = Some(msg);
                }
            }

            // Render children if expanded
            if node.expanded && has_children {
                // Collect child names first
                let child_names: Vec<String> = node
                    .sorted_children()
                    .iter()
                    .map(|(n, _)| n.to_string())
                    .collect();

                ui.indent(&node_path, |ui| {
                    for child_name in child_names {
                        if let Some(child) = node.children.get_mut(&child_name) {
                            if let Some(msg) = self.render_node(ui, &child_name, child) {
                                selected_message = Some(msg);
                            }
                        }
                    }
                });
            }
        });

        selected_message
    }
}
