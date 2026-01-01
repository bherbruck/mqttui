mod app;
mod config;
mod mqtt;
mod theme;

use app::MqttUi;
use iced::{window, Font, Size};

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("MQTT UI", MqttUi::update, MqttUi::view)
        .subscription(MqttUi::subscription)
        .theme(MqttUi::theme)
        .window(window::Settings {
            size: Size::new(1200.0, 800.0),
            min_size: Some(Size::new(800.0, 600.0)),
            ..Default::default()
        })
        .default_font(Font::MONOSPACE)
        .run_with(MqttUi::new)
}
