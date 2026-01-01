//! Custom dark hacker theme for MQTT UI

use iced::theme::Palette;
use iced::Theme;

use crate::styles::colors;

/// Creates the custom dark hacker theme
pub fn mqtt_theme() -> Theme {
    Theme::custom(
        "Hacker".to_string(),
        Palette {
            background: colors::BG_DARK,
            text: colors::TEXT_PRIMARY,
            primary: colors::CYAN,
            success: colors::GREEN,
            danger: colors::RED,
        },
    )
}
