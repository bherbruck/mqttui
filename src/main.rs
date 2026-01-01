// Hide console window on Windows
#![windows_subsystem = "windows"]

mod app;
mod config;
mod mqtt;
mod styles;
mod theme;

use app::MqttUi;
use iced::{window, Font, Size};

// Embed the Nerd Font at compile time
const NERD_FONT: &[u8] = include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf");

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application("MQTT UI", MqttUi::update, MqttUi::view)
        .subscription(MqttUi::subscription)
        .theme(MqttUi::theme)
        .window(window::Settings {
            size: Size::new(1200.0, 800.0),
            min_size: Some(Size::new(800.0, 600.0)),
            // Enable decorations for native resize handles
            decorations: true,
            resizable: true,
            ..Default::default()
        })
        .font(NERD_FONT)
        .default_font(Font::with_name("JetBrainsMono Nerd Font"))
        .run_with(MqttUi::new)
}
