//! Connection form views

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, text,
    text_input, Column,
};
use iced::{Element, Length};

use crate::config::MqttProtocol;
use crate::styles::{self, colors, icons, spacing, typography};

use crate::app::{Message, MqttUi};

impl MqttUi {
    pub fn view_connection_form(&self, editing_id: Option<&str>) -> Element<'_, Message> {
        let title = if editing_id.is_some() {
            "Edit Connection"
        } else {
            "New Connection"
        };

        let protocols: Vec<MqttProtocol> = vec![
            MqttProtocol::Mqtt,
            MqttProtocol::Mqtts,
            MqttProtocol::MqttWs,
            MqttProtocol::MqttsWs,
        ];

        let form = column![
            text(title)
                .size(typography::SIZE_2XL)
                .color(colors::CYAN),
            horizontal_rule(1),
            // Name
            column![
                text("Name")
                    .size(typography::SIZE_SM)
                    .color(colors::TEXT_SECONDARY),
                text_input("Connection name", &self.form_name)
                    .padding(spacing::SM)
                    .style(styles::text_input_default)
                    .on_input(Message::FormNameChanged)
            ]
            .spacing(spacing::XS),
            // Host and Port
            row![
                column![
                    text("Host")
                        .size(typography::SIZE_SM)
                        .color(colors::TEXT_SECONDARY),
                    text_input("localhost", &self.form_host)
                        .padding(spacing::SM)
                        .style(styles::text_input_default)
                        .on_input(Message::FormHostChanged)
                ]
                .spacing(spacing::XS)
                .width(Length::FillPortion(3)),
                column![
                    text("Port")
                        .size(typography::SIZE_SM)
                        .color(colors::TEXT_SECONDARY),
                    text_input("1883", &self.form_port)
                        .padding(spacing::SM)
                        .style(styles::text_input_default)
                        .on_input(Message::FormPortChanged)
                ]
                .spacing(spacing::XS)
                .width(Length::FillPortion(1)),
            ]
            .spacing(spacing::MD),
            // Protocol
            column![
                text("Protocol")
                    .size(typography::SIZE_SM)
                    .color(colors::TEXT_SECONDARY),
                pick_list(
                    protocols,
                    Some(self.form_protocol),
                    Message::FormProtocolChanged
                )
                .padding(spacing::SM)
                .width(Length::Fill)
            ]
            .spacing(spacing::XS),
            // Client ID
            column![
                text("Client ID (optional)")
                    .size(typography::SIZE_SM)
                    .color(colors::TEXT_SECONDARY),
                text_input("Auto-generated if empty", &self.form_client_id)
                    .padding(spacing::SM)
                    .style(styles::text_input_default)
                    .on_input(Message::FormClientIdChanged)
            ]
            .spacing(spacing::XS),
            // Username
            column![
                text("Username (optional)")
                    .size(typography::SIZE_SM)
                    .color(colors::TEXT_SECONDARY),
                text_input("Username", &self.form_username)
                    .padding(spacing::SM)
                    .style(styles::text_input_default)
                    .on_input(Message::FormUsernameChanged)
            ]
            .spacing(spacing::XS),
            // Password
            column![
                text("Password (optional)")
                    .size(typography::SIZE_SM)
                    .color(colors::TEXT_SECONDARY),
                text_input("Password", &self.form_password)
                    .padding(spacing::SM)
                    .secure(true)
                    .style(styles::text_input_default)
                    .on_input(Message::FormPasswordChanged)
            ]
            .spacing(spacing::XS),
            horizontal_rule(1),
            // Subscriptions
            self.view_subscriptions_form(),
            horizontal_rule(1),
            // Buttons
            row![
                button(text("Cancel").size(typography::SIZE_MD))
                    .padding([spacing::SM, spacing::LG])
                    .style(styles::button_secondary)
                    .on_press(Message::FormCancel),
                horizontal_space(),
                button(text("Save").size(typography::SIZE_MD))
                    .padding([spacing::SM, spacing::LG])
                    .style(styles::button_secondary)
                    .on_press(Message::FormSaveConnection),
                button(
                    row![
                        text(icons::CONNECT).size(typography::SIZE_MD),
                        text(" Connect").size(typography::SIZE_MD)
                    ]
                    .spacing(spacing::XS)
                )
                .padding([spacing::SM, spacing::LG])
                .style(styles::button_primary)
                .on_press(Message::FormConnectAndSave),
            ]
            .spacing(spacing::MD)
        ]
        .spacing(spacing::MD)
        .padding(spacing::LG)
        .max_width(500);

        container(
            container(form).style(styles::container_panel),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .padding(spacing::LG)
        .into()
    }

    pub fn view_subscriptions_form(&self) -> Element<'_, Message> {
        let qos_options: Vec<u8> = vec![0, 1, 2];

        let mut content = Column::new().spacing(spacing::SM);

        content = content.push(
            row![
                text("Subscriptions")
                    .size(typography::SIZE_MD)
                    .color(colors::TEXT_PRIMARY),
                horizontal_space(),
                button(
                    text(icons::PLUS)
                        .size(typography::SIZE_SM)
                        .center(),
                )
                .padding([spacing::XS, spacing::SM])
                .style(styles::button_secondary)
                .on_press(Message::FormAddSubscription)
            ]
            .align_y(iced::Alignment::Center),
        );

        for (idx, (topic, qos)) in self.form_subscriptions.iter().enumerate() {
            let topic_clone = topic.clone();
            let sub_row = row![
                text_input("topic/#", &topic_clone)
                    .padding(spacing::SM)
                    .style(styles::text_input_default)
                    .on_input(move |v| Message::FormSubscriptionTopicChanged(idx, v))
                    .width(Length::Fill),
                pick_list(qos_options.clone(), Some(*qos), move |v| {
                    Message::FormSubscriptionQosChanged(idx, v)
                })
                .padding(spacing::XS)
                .width(60),
                button(
                    text(icons::TIMES)
                        .size(typography::SIZE_SM)
                        .center(),
                )
                .padding([spacing::XS, spacing::SM])
                .style(styles::button_text)
                .on_press(Message::FormRemoveSubscription(idx))
            ]
            .spacing(spacing::SM)
            .align_y(iced::Alignment::Center);

            content = content.push(sub_row);
        }

        content.into()
    }
}
