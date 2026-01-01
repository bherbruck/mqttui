//! Publish panel view

use iced::widget::{button, column, horizontal_rule, pick_list, row, text, text_input, toggler};
use iced::{Element, Length};

use crate::styles::{self, colors, icons, spacing, typography};

use crate::app::{Message, MqttUi};

impl MqttUi {
    pub fn view_publish_panel(&self, _id: &str, is_connected: bool) -> Element<'_, Message> {
        let qos_options = vec![0u8, 1, 2];

        column![
            row![
                text(icons::SEND).size(typography::SIZE_MD).color(colors::CYAN),
                text(" Publish")
                    .size(typography::SIZE_LG)
                    .color(colors::CYAN)
            ]
            .spacing(spacing::XS),
            horizontal_rule(1),
            column![
                text("Topic")
                    .size(typography::SIZE_SM)
                    .color(colors::TEXT_SECONDARY),
                text_input("topic/path", &self.publish_topic)
                    .padding(spacing::SM)
                    .style(styles::text_input_default)
                    .on_input(Message::PublishTopicChanged)
            ]
            .spacing(spacing::XS),
            column![
                text("Payload")
                    .size(typography::SIZE_SM)
                    .color(colors::TEXT_SECONDARY),
                text_input("{\"key\": \"value\"}", &self.publish_payload)
                    .padding(spacing::SM)
                    .style(styles::text_input_default)
                    .on_input(Message::PublishPayloadChanged)
            ]
            .spacing(spacing::XS),
            row![
                column![
                    text("QoS")
                        .size(typography::SIZE_SM)
                        .color(colors::TEXT_SECONDARY),
                    pick_list(
                        qos_options,
                        Some(self.publish_qos),
                        Message::PublishQosChanged
                    )
                    .padding(spacing::SM)
                ]
                .spacing(spacing::XS)
                .width(Length::FillPortion(1)),
                column![
                    text("Retain")
                        .size(typography::SIZE_SM)
                        .color(colors::TEXT_SECONDARY),
                    toggler(self.publish_retain).on_toggle(Message::PublishRetainChanged)
                ]
                .spacing(spacing::XS)
                .width(Length::FillPortion(1)),
            ]
            .spacing(spacing::MD),
            if is_connected {
                button(
                    row![
                        text(icons::SEND).size(typography::SIZE_MD),
                        text(" Send").size(typography::SIZE_MD)
                    ]
                    .spacing(spacing::XS)
                    .width(Length::Fill)
                )
                .padding([spacing::SM, spacing::MD])
                .width(Length::Fill)
                .style(styles::button_primary)
                .on_press(Message::SendMessage)
            } else {
                button(text("Send").size(typography::SIZE_MD).width(Length::Fill))
                    .padding([spacing::SM, spacing::MD])
                    .width(Length::Fill)
                    .style(styles::button_secondary)
            }
        ]
        .spacing(spacing::MD)
        .padding(spacing::MD)
        .into()
    }
}
