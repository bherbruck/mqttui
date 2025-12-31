mod app;
mod config;
mod mqtt;
mod ui;

use app::MqttUiApp;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("mqttui=info".parse().unwrap()))
        .init();

    // Window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("MQTT UI")
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "MQTT UI",
        options,
        Box::new(|cc| Ok(Box::new(MqttUiApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    // Simple programmatic icon - a message bubble shape
    let size = 64;
    let mut rgba = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;

            // Create a simple MQTT-themed icon
            let cx = x as f32 - size as f32 / 2.0;
            let cy = y as f32 - size as f32 / 2.0;
            let dist = (cx * cx + cy * cy).sqrt();

            // Rounded square with message bubble
            let in_shape = dist < 24.0 || (y > size / 2 && x > size / 3 && x < size * 2 / 3);

            if in_shape && dist < 28.0 {
                // Indigo color from theme
                rgba[idx] = 99;     // R
                rgba[idx + 1] = 102; // G
                rgba[idx + 2] = 241; // B
                rgba[idx + 3] = 255; // A
            }
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}
