use egui::{Color32, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals};

pub struct MqttUiTheme;

impl MqttUiTheme {
    // Color palette - Modern dark theme
    pub const BG_DARK: Color32 = Color32::from_rgb(18, 18, 24);
    pub const BG_MEDIUM: Color32 = Color32::from_rgb(28, 28, 36);
    pub const BG_LIGHT: Color32 = Color32::from_rgb(38, 38, 48);
    pub const BG_HOVER: Color32 = Color32::from_rgb(48, 48, 60);
    pub const BG_ACTIVE: Color32 = Color32::from_rgb(58, 58, 72);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 240, 245);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 175);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 100, 115);

    pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(99, 102, 241); // Indigo
    #[allow(dead_code)]
    pub const ACCENT_SECONDARY: Color32 = Color32::from_rgb(139, 92, 246); // Purple
    pub const ACCENT_SUCCESS: Color32 = Color32::from_rgb(34, 197, 94); // Green
    pub const ACCENT_WARNING: Color32 = Color32::from_rgb(245, 158, 11); // Amber
    pub const ACCENT_ERROR: Color32 = Color32::from_rgb(239, 68, 68); // Red
    pub const ACCENT_INFO: Color32 = Color32::from_rgb(59, 130, 246); // Blue

    pub const BORDER: Color32 = Color32::from_rgb(55, 55, 70);
    #[allow(dead_code)]
    pub const BORDER_FOCUS: Color32 = Color32::from_rgb(99, 102, 241);

    #[allow(clippy::field_reassign_with_default)]
    pub fn apply(ctx: &egui::Context) {
        let mut style = Style::default();

        // Font sizes
        style.text_styles = [
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Heading,
                FontId::new(20.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
        ]
        .into();

        // Spacing
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(16.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.indent = 20.0;

        // Visuals
        let mut visuals = Visuals::dark();

        visuals.window_fill = Self::BG_MEDIUM;
        visuals.panel_fill = Self::BG_DARK;
        visuals.faint_bg_color = Self::BG_LIGHT;
        visuals.extreme_bg_color = Self::BG_DARK;

        visuals.window_rounding = Rounding::same(12.0);
        visuals.window_stroke = Stroke::new(1.0, Self::BORDER);

        // Widget styling
        visuals.widgets.noninteractive.bg_fill = Self::BG_MEDIUM;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_SECONDARY);
        visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

        visuals.widgets.inactive.bg_fill = Self::BG_LIGHT;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.inactive.rounding = Rounding::same(8.0);

        visuals.widgets.hovered.bg_fill = Self::BG_HOVER;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.hovered.rounding = Rounding::same(8.0);

        visuals.widgets.active.bg_fill = Self::ACCENT_PRIMARY;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.active.rounding = Rounding::same(8.0);

        visuals.widgets.open.bg_fill = Self::BG_ACTIVE;
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.open.rounding = Rounding::same(8.0);

        visuals.selection.bg_fill = Self::ACCENT_PRIMARY.gamma_multiply(0.4);
        visuals.selection.stroke = Stroke::new(1.0, Self::ACCENT_PRIMARY);

        visuals.hyperlink_color = Self::ACCENT_INFO;
        visuals.warn_fg_color = Self::ACCENT_WARNING;
        visuals.error_fg_color = Self::ACCENT_ERROR;

        style.visuals = visuals;
        ctx.set_style(style);
    }

    #[allow(dead_code)]
    pub fn status_color(connected: bool) -> Color32 {
        if connected {
            Self::ACCENT_SUCCESS
        } else {
            Self::TEXT_MUTED
        }
    }

    pub fn qos_color(qos: u8) -> Color32 {
        match qos {
            0 => Self::TEXT_MUTED,
            1 => Self::ACCENT_INFO,
            _ => Self::ACCENT_WARNING,
        }
    }
}

// Custom styled widgets
pub fn styled_button(ui: &mut egui::Ui, text: &str, primary: bool) -> egui::Response {
    let button = if primary {
        egui::Button::new(text).fill(MqttUiTheme::ACCENT_PRIMARY)
    } else {
        egui::Button::new(text).fill(MqttUiTheme::BG_LIGHT)
    };
    ui.add(button)
}

pub fn styled_text_edit(ui: &mut egui::Ui, text: &mut String, hint: &str) -> egui::Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .margin(egui::Margin::symmetric(12.0, 8.0)),
    )
}

#[allow(dead_code)]
pub fn styled_text_edit_multiline(
    ui: &mut egui::Ui,
    text: &mut String,
    hint: &str,
) -> egui::Response {
    ui.add(
        egui::TextEdit::multiline(text)
            .hint_text(hint)
            .margin(egui::Margin::symmetric(12.0, 8.0)),
    )
}

pub fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(text)
            .color(MqttUiTheme::TEXT_SECONDARY)
            .size(12.0),
    );
    ui.add_space(4.0);
}
