//! Connection view (pane grid layout)

use iced::widget::{button, column, container, pane_grid, row, text};
use iced::{Element, Length};

use crate::styles::{self, colors, icons, spacing, typography};

use crate::app::types::Pane;
use crate::app::{Message, MqttUi};

impl MqttUi {
    pub fn view_connection(&self, id: &str) -> Element<'_, Message> {
        let Some(conn) = self.connections.get(id) else {
            return self.view_connection_not_started(id);
        };

        let is_connected = conn.status.is_connected();
        let id_owned = id.to_string();

        pane_grid::PaneGrid::new(&self.panes, move |_pane_id, pane, _is_maximized| {
            let content: Element<Message> = match pane {
                Pane::Publish => self.view_publish_panel(&id_owned, is_connected),
                Pane::Topics => self.view_topic_tree(&id_owned),
                Pane::Message => self.view_message_panel(&id_owned),
            };

            pane_grid::Content::new(
                container(content)
                    .style(styles::container_panel)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
        })
        .on_resize(10, Message::PaneResized)
        .into()
    }

    pub fn view_connection_not_started(&self, id: &str) -> Element<'_, Message> {
        let Some(config) = self.config.get_connection(id) else {
            return text("Connection not found")
                .color(colors::TEXT_MUTED)
                .into();
        };

        let name = config.name.clone();
        let uri = config.uri();

        container(
            column![
                text(name)
                    .size(typography::SIZE_2XL)
                    .color(colors::TEXT_PRIMARY),
                text(uri)
                    .size(typography::SIZE_MD)
                    .color(colors::TEXT_SECONDARY),
                button(
                    row![
                        text(icons::CONNECT).size(typography::SIZE_LG),
                        text(" Connect").size(typography::SIZE_LG)
                    ]
                    .spacing(spacing::SM)
                )
                .padding([spacing::MD, spacing::XL])
                .style(styles::button_primary)
                .on_press(Message::Connect(id.to_string()))
            ]
            .spacing(spacing::MD)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}
