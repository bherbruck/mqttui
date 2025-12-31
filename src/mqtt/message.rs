use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: u8,
    pub retain: bool,
    pub timestamp: DateTime<Utc>,
}

impl MqttMessage {
    pub fn new(topic: String, payload: Vec<u8>, qos: u8, retain: bool) -> Self {
        Self {
            topic,
            payload,
            qos,
            retain,
            timestamp: Utc::now(),
        }
    }

    pub fn payload_as_string(&self) -> String {
        String::from_utf8_lossy(&self.payload).to_string()
    }

    pub fn payload_as_json(&self) -> Option<serde_json::Value> {
        serde_json::from_slice(&self.payload).ok()
    }

    pub fn payload_preview(&self, max_len: usize) -> String {
        let s = self.payload_as_string();
        if s.len() > max_len {
            format!("{}...", &s[..max_len])
        } else {
            s
        }
    }

    pub fn is_json(&self) -> bool {
        self.payload_as_json().is_some()
    }

    pub fn formatted_payload(&self) -> String {
        if let Some(json) = self.payload_as_json() {
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| self.payload_as_string())
        } else {
            self.payload_as_string()
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl ConnectionStatus {
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected)
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            ConnectionStatus::Disconnected => egui::Color32::GRAY,
            ConnectionStatus::Connecting => egui::Color32::YELLOW,
            ConnectionStatus::Connected => egui::Color32::GREEN,
            ConnectionStatus::Error(_) => egui::Color32::RED,
        }
    }

    pub fn text(&self) -> &str {
        match self {
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Error(_) => "Error",
        }
    }
}
