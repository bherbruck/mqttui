//! Tab bar view

use iced::widget::{button, container, horizontal_space, row, text, Row};
use iced::{Element, Length};

use crate::mqtt::ConnectionStatus;
use crate::styles::{self, colors, icons, spacing, typography};

use crate::app::{Message, MqttUi};

impl MqttUi {
    pub fn view_tabs(&self) -> Element<'_, Message> {
        let mut tabs_row = Row::new().spacing(spacing::XS);

        // Home button
        let home_btn = button(
            container(text(icons::HOME).size(typography::SIZE_LG))
                .width(20)
                .height(20)
                .center_x(20)
                .center_y(20),
        )
        .padding(spacing::SM)
        .style(styles::button_tab(self.active_tab.is_none()))
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
                    ConnectionStatus::Connected => icons::CIRCLE_FILLED,
                    ConnectionStatus::Connecting => icons::CIRCLE_HALF,
                    _ => icons::CIRCLE_EMPTY,
                })
                .size(typography::SIZE_XS)
                .color(match status {
                    ConnectionStatus::Connected => colors::GREEN,
                    ConnectionStatus::Connecting => colors::AMBER,
                    ConnectionStatus::Error(_) => colors::RED,
                    _ => colors::TEXT_MUTED,
                });

                let tab_content = row![
                    status_dot,
                    text(&config.name).size(typography::SIZE_MD),
                    button(
                        container(text(icons::TIMES).size(typography::SIZE_XS))
                            .width(14)
                            .height(14)
                            .center_x(14)
                            .center_y(14),
                    )
                    .padding(2)
                    .style(styles::button_text)
                    .on_press(Message::CloseTab(tab_id.clone()))
                ]
                .spacing(spacing::SM)
                .align_y(iced::Alignment::Center);

                let tab_btn = button(tab_content)
                    .padding([spacing::SM, spacing::MD])
                    .style(styles::button_tab(is_active))
                    .on_press(Message::SelectTab(tab_id.clone()));

                tabs_row = tabs_row.push(tab_btn);
            }
        }

        // New tab button
        tabs_row = tabs_row.push(horizontal_space());
        tabs_row = tabs_row.push(
            button(
                container(text(icons::PLUS).size(typography::SIZE_LG))
                    .width(20)
                    .height(20)
                    .center_x(20)
                    .center_y(20),
            )
            .padding(spacing::SM)
            .style(styles::button_secondary)
            .on_press(Message::NewConnection),
        );

        container(tabs_row.padding(spacing::SM))
            .width(Length::Fill)
            .style(styles::container_panel)
            .into()
    }
}
