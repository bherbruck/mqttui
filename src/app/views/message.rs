//! Message panel view

use iced::widget::{column, container, horizontal_rule, row, scrollable, text, Column};
use iced::{Element, Length};

use crate::styles::{self, colors, icons, spacing, typography};

use crate::app::{Message, MqttUi};

impl MqttUi {
    pub fn view_message_panel(&self, id: &str) -> Element<'_, Message> {
        let selected_msg = self.selected_messages.get(id).cloned().flatten();

        let mut content = Column::new().spacing(spacing::SM).padding(spacing::MD);
        content = content.push(
            row![
                text(icons::MESSAGE)
                    .size(typography::SIZE_MD)
                    .color(colors::CYAN),
                text(" Message")
                    .size(typography::SIZE_LG)
                    .color(colors::CYAN)
            ]
            .spacing(spacing::XS),
        );
        content = content.push(horizontal_rule(1));

        if let Some(msg) = selected_msg {
            let topic = msg.topic.clone();
            let qos = msg.qos.to_string();
            let retain = if msg.retain { "Yes" } else { "No" };
            let time = msg.timestamp.format("%H:%M:%S").to_string();
            let payload = msg.formatted_payload();

            content = content.push(
                column![
                    row![
                        text("Topic:")
                            .size(typography::SIZE_SM)
                            .color(colors::TEXT_SECONDARY),
                        text(topic)
                            .size(typography::SIZE_SM)
                            .color(colors::MAGENTA)
                    ]
                    .spacing(spacing::SM),
                    row![
                        text("QoS:")
                            .size(typography::SIZE_SM)
                            .color(colors::TEXT_SECONDARY),
                        text(qos).size(typography::SIZE_SM).color(colors::TEXT_PRIMARY),
                        text("Retain:")
                            .size(typography::SIZE_SM)
                            .color(colors::TEXT_SECONDARY),
                        text(retain)
                            .size(typography::SIZE_SM)
                            .color(colors::TEXT_PRIMARY),
                    ]
                    .spacing(spacing::SM),
                    row![
                        text("Time:")
                            .size(typography::SIZE_SM)
                            .color(colors::TEXT_SECONDARY),
                        text(time).size(typography::SIZE_SM).color(colors::TEXT_PRIMARY)
                    ]
                    .spacing(spacing::SM),
                    horizontal_rule(1),
                    text("Payload:")
                        .size(typography::SIZE_SM)
                        .color(colors::TEXT_SECONDARY),
                    scrollable(
                        container(
                            text(payload)
                                .size(typography::SIZE_SM)
                                .color(colors::GREEN)
                        )
                        .padding(spacing::MD)
                        .style(styles::container_code)
                    )
                    .height(Length::Fill)
                ]
                .spacing(spacing::SM),
            );
        } else {
            content = content.push(
                text("Select a topic to view messages")
                    .size(typography::SIZE_MD)
                    .color(colors::TEXT_MUTED),
            );
        }

        content.height(Length::Fill).into()
    }
}
