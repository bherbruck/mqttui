//! Home view with connection cards

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text, Column,
    Row,
};
use iced::{Element, Length};

use crate::config::ConnectionConfig;
use crate::mqtt::ConnectionStatus;
use crate::styles::{self, colors, icons, spacing, typography};

use crate::app::{Message, MqttUi};

impl MqttUi {
    pub fn view_home(&self) -> Element<'_, Message> {
        let mut content = Column::new()
            .spacing(spacing::LG)
            .padding(spacing::LG);

        // Header
        content = content.push(
            row![
                text("Connections")
                    .size(typography::SIZE_3XL)
                    .color(colors::TEXT_PRIMARY),
                horizontal_space(),
                button(
                    row![
                        text(icons::PLUS).size(typography::SIZE_MD),
                        text(" New Connection").size(typography::SIZE_MD)
                    ]
                    .spacing(spacing::XS)
                )
                .padding([spacing::SM, spacing::MD])
                .style(styles::button_primary)
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
                            .size(typography::SIZE_XL)
                            .color(colors::TEXT_SECONDARY),
                        text("Click \"+ New Connection\" to get started")
                            .size(typography::SIZE_MD)
                            .color(colors::TEXT_MUTED)
                    ]
                    .spacing(spacing::SM)
                    .align_x(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .height(200)
                .center_x(Length::Fill)
                .center_y(200),
            );
        } else {
            let mut cards_row = Row::new().spacing(spacing::MD);

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

    pub fn view_connection_card(
        &self,
        config: &ConnectionConfig,
        status: &ConnectionStatus,
    ) -> Element<'_, Message> {
        let status_text = match status {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Error(_) => "Error",
        };

        let status_color = match status {
            ConnectionStatus::Connected => colors::GREEN,
            ConnectionStatus::Connecting => colors::AMBER,
            ConnectionStatus::Error(_) => colors::RED,
            _ => colors::TEXT_MUTED,
        };

        let id = config.id.clone();
        let is_connected = status.is_connected();

        let connect_btn = if is_connected {
            button(
                row![
                    text(icons::DISCONNECT).size(typography::SIZE_SM),
                    text(" Disconnect").size(typography::SIZE_SM)
                ]
                .spacing(spacing::XS),
            )
            .padding([spacing::XS, spacing::SM])
            .style(styles::button_secondary)
            .on_press(Message::Disconnect(id.clone()))
        } else {
            button(
                row![
                    text(icons::CONNECT).size(typography::SIZE_SM),
                    text(" Connect").size(typography::SIZE_SM)
                ]
                .spacing(spacing::XS),
            )
            .padding([spacing::XS, spacing::SM])
            .style(styles::button_primary)
            .on_press(Message::Connect(id.clone()))
        };

        let name = config.name.clone();
        let uri = config.uri();

        let card_content = column![
            row![
                text(icons::CIRCLE_FILLED)
                    .size(typography::SIZE_XS)
                    .color(status_color),
                text(status_text)
                    .size(typography::SIZE_SM)
                    .color(status_color),
            ]
            .spacing(spacing::XS),
            text(name).size(typography::SIZE_LG).color(colors::TEXT_PRIMARY),
            text(uri).size(typography::SIZE_SM).color(colors::TEXT_SECONDARY),
            row![
                connect_btn,
                button(text("Edit").size(typography::SIZE_SM))
                    .padding([spacing::XS, spacing::SM])
                    .style(styles::button_secondary)
                    .on_press(Message::EditConnection(id.clone())),
                button(text("Delete").size(typography::SIZE_SM))
                    .padding([spacing::XS, spacing::SM])
                    .style(styles::button_danger)
                    .on_press(Message::DeleteConnection(id.clone())),
            ]
            .spacing(spacing::SM)
        ]
        .spacing(spacing::MD)
        .padding(spacing::MD)
        .width(260);

        container(
            button(card_content)
                .style(styles::button_secondary)
                .on_press(Message::OpenConnection(id)),
        )
        .style(styles::container_card)
        .into()
    }
}
