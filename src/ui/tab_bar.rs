use egui::{RichText, Ui, Vec2};

use crate::mqtt::ConnectionStatus;

use super::identicon::Identicon;
use super::theme::MqttUiTheme;

pub struct TabBar;

#[derive(Debug, Clone)]
pub struct TabInfo {
    pub id: String,
    pub name: String,
    pub status: ConnectionStatus,
}

impl TabBar {
    pub fn ui(ui: &mut Ui, tabs: &[TabInfo], active_tab: Option<&str>) -> Option<TabBarAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            // Home button
            let home_active = active_tab.is_none();
            let home_response = ui.allocate_ui(Vec2::new(40.0, 32.0), |ui| {
                egui::Frame::none()
                    .fill(if home_active {
                        MqttUiTheme::BG_MEDIUM
                    } else {
                        MqttUiTheme::BG_DARK
                    })
                    .rounding(egui::Rounding {
                        nw: 8.0,
                        ne: 8.0,
                        sw: 0.0,
                        se: 0.0,
                    })
                    .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("\u{2302}")
                                .color(if home_active {
                                    MqttUiTheme::TEXT_PRIMARY
                                } else {
                                    MqttUiTheme::TEXT_SECONDARY
                                })
                                .size(16.0),
                        );
                    });
            });

            if home_response
                .response
                .interact(egui::Sense::click())
                .clicked()
            {
                action = Some(TabBarAction::SelectHome);
            }

            // Connection tabs
            for tab in tabs {
                let is_active = active_tab == Some(&tab.id);
                let tab_response = Self::render_tab(ui, tab, is_active);

                if let Some(tab_action) = tab_response {
                    action = Some(tab_action);
                }
            }

            // Add new tab button
            let add_response = ui.allocate_ui(Vec2::new(32.0, 32.0), |ui| {
                egui::Frame::none()
                    .fill(MqttUiTheme::BG_DARK)
                    .rounding(egui::Rounding {
                        nw: 8.0,
                        ne: 8.0,
                        sw: 0.0,
                        se: 0.0,
                    })
                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("+")
                                .color(MqttUiTheme::TEXT_SECONDARY)
                                .size(16.0),
                        );
                    });
            });

            if add_response
                .response
                .interact(egui::Sense::click())
                .clicked()
            {
                action = Some(TabBarAction::NewTab);
            }

            // Right side - feedback button
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("\u{1F4AC} Feedback").color(MqttUiTheme::TEXT_SECONDARY),
                        )
                        .fill(egui::Color32::TRANSPARENT),
                    )
                    .clicked()
                {
                    action = Some(TabBarAction::Feedback);
                }
            });
        });

        action
    }

    fn render_tab(ui: &mut Ui, tab: &TabInfo, is_active: bool) -> Option<TabBarAction> {
        let mut action = None;

        let response = ui.allocate_ui(Vec2::new(180.0, 32.0), |ui| {
            egui::Frame::none()
                .fill(if is_active {
                    MqttUiTheme::BG_MEDIUM
                } else {
                    MqttUiTheme::BG_DARK
                })
                .rounding(egui::Rounding {
                    nw: 8.0,
                    ne: 8.0,
                    sw: 0.0,
                    se: 0.0,
                })
                .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Status indicator
                        Identicon::draw_status_dot(ui, tab.status.is_connected());

                        ui.add_space(4.0);

                        // Mini identicon
                        Identicon::draw(ui, &tab.id, 20.0);

                        ui.add_space(4.0);

                        // Tab name (truncated if needed)
                        let name = if tab.name.len() > 15 {
                            format!("{}...", &tab.name[..12])
                        } else {
                            tab.name.clone()
                        };

                        ui.label(
                            RichText::new(&name)
                                .color(if is_active {
                                    MqttUiTheme::TEXT_PRIMARY
                                } else {
                                    MqttUiTheme::TEXT_SECONDARY
                                })
                                .size(13.0),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Close button
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("\u{2715}")
                                            .color(MqttUiTheme::TEXT_MUTED)
                                            .size(10.0),
                                    )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .frame(false),
                                )
                                .clicked()
                            {
                                action = Some(TabBarAction::CloseTab(tab.id.clone()));
                            }
                        });
                    });
                });
        });

        if action.is_none() && response.response.interact(egui::Sense::click()).clicked() {
            action = Some(TabBarAction::SelectTab(tab.id.clone()));
        }

        action
    }
}

pub enum TabBarAction {
    SelectHome,
    SelectTab(String),
    CloseTab(String),
    NewTab,
    Feedback,
}
