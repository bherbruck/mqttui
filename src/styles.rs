//! Design tokens and custom styles for the dark hacker theme

use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Theme};

// =============================================================================
// SPACING SCALE
// =============================================================================

pub mod spacing {
    pub const XS: u16 = 4;
    pub const SM: u16 = 8;
    pub const MD: u16 = 16;
    pub const LG: u16 = 24;
    pub const XL: u16 = 32;
}

// =============================================================================
// COLOR PALETTE - Dark Hacker Theme
// =============================================================================

#[allow(dead_code)]
pub mod colors {
    use iced::Color;

    // Backgrounds
    pub const BG_DARKEST: Color = Color::from_rgb(0.02, 0.02, 0.02); // #050505
    pub const BG_DARK: Color = Color::from_rgb(0.05, 0.05, 0.07); // #0d0d12
    pub const BG_SURFACE: Color = Color::from_rgb(0.08, 0.08, 0.11); // #14141c
    pub const BG_ELEVATED: Color = Color::from_rgb(0.11, 0.11, 0.15); // #1c1c26

    // Borders
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0.15, 0.15, 0.20); // #262633
    pub const BORDER_DEFAULT: Color = Color::from_rgb(0.20, 0.20, 0.28); // #333347

    // Text
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.94, 0.94, 0.96); // #f0f0f5
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.55, 0.62); // #8c8c9e
    pub const TEXT_MUTED: Color = Color::from_rgb(0.35, 0.35, 0.42); // #59596b

    // Accent colors - Neon
    pub const CYAN: Color = Color::from_rgb(0.0, 0.85, 1.0); // #00d9ff - Primary
    pub const CYAN_DIM: Color = Color::from_rgb(0.0, 0.50, 0.60); // #008099
    pub const GREEN: Color = Color::from_rgb(0.0, 1.0, 0.25); // #00ff41 - Success/Connected
    pub const GREEN_DIM: Color = Color::from_rgb(0.0, 0.50, 0.13); // #008021
    pub const AMBER: Color = Color::from_rgb(1.0, 0.69, 0.0); // #ffb000 - Warning/Connecting
    pub const AMBER_DIM: Color = Color::from_rgb(0.50, 0.35, 0.0); // #805900
    pub const RED: Color = Color::from_rgb(1.0, 0.0, 0.25); // #ff0040 - Error/Danger
    pub const RED_DIM: Color = Color::from_rgb(0.50, 0.0, 0.13); // #800021
    pub const MAGENTA: Color = Color::from_rgb(1.0, 0.0, 1.0); // #ff00ff - Accent alt

    // Transparent variants
    pub const CYAN_ALPHA: Color = Color::from_rgba(0.0, 0.85, 1.0, 0.15);
    pub const GREEN_ALPHA: Color = Color::from_rgba(0.0, 1.0, 0.25, 0.15);
    pub const RED_ALPHA: Color = Color::from_rgba(1.0, 0.0, 0.25, 0.15);
}

// =============================================================================
// TYPOGRAPHY
// =============================================================================

pub mod typography {
    pub const SIZE_XS: f32 = 10.0;
    pub const SIZE_SM: f32 = 12.0;
    pub const SIZE_MD: f32 = 14.0;
    pub const SIZE_LG: f32 = 16.0;
    pub const SIZE_XL: f32 = 20.0;
    pub const SIZE_2XL: f32 = 24.0;
    pub const SIZE_3XL: f32 = 28.0;
}

// =============================================================================
// NERD FONT ICONS
// =============================================================================

pub mod icons {
    // Navigation
    pub const HOME: &str = "\u{f015}"; //
    pub const PLUS: &str = "\u{f067}"; //
    pub const TIMES: &str = "\u{f00d}"; //

    // Status indicators
    pub const CIRCLE_FILLED: &str = "\u{f111}"; //
    pub const CIRCLE_HALF: &str = "\u{f042}"; //
    pub const CIRCLE_EMPTY: &str = "\u{f10c}"; //

    // MQTT specific
    pub const CONNECT: &str = "\u{f0c1}"; //
    pub const DISCONNECT: &str = "\u{f127}"; //
    pub const SEND: &str = "\u{f1d8}"; //
    pub const TOPIC: &str = "\u{f07c}"; //
    pub const MESSAGE: &str = "\u{f075}"; //

    // Tree
    pub const CHEVRON_RIGHT: &str = "\u{f054}"; //
    pub const CHEVRON_DOWN: &str = "\u{f078}"; //

    // Actions
    pub const TRASH: &str = "\u{f1f8}"; //
}

// =============================================================================
// BUTTON STYLES
// =============================================================================

/// Primary button - cyan accent
pub fn button_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(colors::CYAN)),
        text_color: colors::BG_DARKEST,
        border: Border {
            color: colors::CYAN,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.9, 1.0))),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::CYAN_DIM)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(colors::BG_ELEVATED)),
            text_color: colors::TEXT_MUTED,
            border: Border {
                color: colors::BORDER_SUBTLE,
                ..base.border
            },
            ..base
        },
    }
}

/// Secondary button - subtle background
pub fn button_secondary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(colors::BG_ELEVATED)),
        text_color: colors::TEXT_PRIMARY,
        border: Border {
            color: colors::BORDER_DEFAULT,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::BG_SURFACE)),
            border: Border {
                color: colors::CYAN_DIM,
                ..base.border
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::BG_DARK)),
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::TEXT_MUTED,
            ..base
        },
    }
}

/// Danger button - red accent
pub fn button_danger(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(colors::RED_DIM)),
        text_color: colors::RED,
        border: Border {
            color: colors::RED_DIM,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::RED)),
            text_color: Color::WHITE,
            border: Border {
                color: colors::RED,
                ..base.border
            },
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::RED_DIM)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(colors::BG_ELEVATED)),
            text_color: colors::TEXT_MUTED,
            border: Border {
                color: colors::BORDER_SUBTLE,
                ..base.border
            },
            ..base
        },
    }
}

/// Text button - minimal, for icons
pub fn button_text(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: colors::TEXT_SECONDARY,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    };

    match status {
        button::Status::Active => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(colors::BG_ELEVATED)),
            text_color: colors::TEXT_PRIMARY,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(colors::BG_SURFACE)),
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: colors::TEXT_MUTED,
            ..base
        },
    }
}

/// Tab button - active/inactive states
pub fn button_tab(is_active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme: &Theme, status: button::Status| {
        if is_active {
            button::Style {
                background: Some(Background::Color(colors::CYAN_ALPHA)),
                text_color: colors::CYAN,
                border: Border {
                    color: colors::CYAN,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            }
        } else {
            let base = button::Style {
                background: Some(Background::Color(colors::BG_ELEVATED)),
                text_color: colors::TEXT_SECONDARY,
                border: Border {
                    color: colors::BORDER_SUBTLE,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            };

            match status {
                button::Status::Hovered => button::Style {
                    text_color: colors::TEXT_PRIMARY,
                    border: Border {
                        color: colors::BORDER_DEFAULT,
                        ..base.border
                    },
                    ..base
                },
                _ => base,
            }
        }
    }
}

// =============================================================================
// CONTAINER STYLES
// =============================================================================

/// Panel container with border
pub fn container_panel(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BG_SURFACE)),
        border: Border {
            color: colors::BORDER_SUBTLE,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    }
}

/// Card container
pub fn container_card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BG_ELEVATED)),
        border: Border {
            color: colors::BORDER_DEFAULT,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    }
}

/// Code/payload container
pub fn container_code(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(colors::BG_DARK)),
        border: Border {
            color: colors::BORDER_SUBTLE,
            width: 1.0,
            radius: 2.0.into(),
        },
        ..Default::default()
    }
}

// =============================================================================
// TEXT INPUT STYLES
// =============================================================================

/// Default text input
pub fn text_input_default(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Background::Color(colors::BG_DARK),
        border: Border {
            color: colors::BORDER_DEFAULT,
            width: 1.0,
            radius: 2.0.into(),
        },
        icon: colors::TEXT_MUTED,
        placeholder: colors::TEXT_MUTED,
        value: colors::TEXT_PRIMARY,
        selection: colors::CYAN_ALPHA,
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: colors::CYAN_DIM,
                ..base.border
            },
            ..base
        },
        text_input::Status::Focused => text_input::Style {
            border: Border {
                color: colors::CYAN,
                ..base.border
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(colors::BG_SURFACE),
            value: colors::TEXT_MUTED,
            ..base
        },
    }
}
