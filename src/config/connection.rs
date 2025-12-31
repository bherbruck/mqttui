use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MqttProtocol {
    #[default]
    Mqtt,
    MqttWs,
    Mqtts,
    MqttsWs,
}

impl MqttProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            MqttProtocol::Mqtt => "mqtt",
            MqttProtocol::MqttWs => "ws",
            MqttProtocol::Mqtts => "mqtts",
            MqttProtocol::MqttsWs => "wss",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            MqttProtocol::Mqtt => 1883,
            MqttProtocol::MqttWs => 8083,
            MqttProtocol::Mqtts => 8883,
            MqttProtocol::MqttsWs => 8084,
        }
    }

    pub fn all() -> &'static [MqttProtocol] {
        &[
            MqttProtocol::Mqtt,
            MqttProtocol::MqttWs,
            MqttProtocol::Mqtts,
            MqttProtocol::MqttsWs,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MqttVersion {
    #[default]
    V311,
    V5,
}

impl MqttVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            MqttVersion::V311 => "3.1.1",
            MqttVersion::V5 => "5.0",
        }
    }

    pub fn all() -> &'static [MqttVersion] {
        &[MqttVersion::V311, MqttVersion::V5]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub topic: String,
    pub qos: u8,
}

impl Default for Subscription {
    fn default() -> Self {
        Self {
            topic: "#".to_string(),
            qos: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub protocol: MqttProtocol,
    pub host: String,
    pub port: u16,
    pub version: MqttVersion,
    pub username: Option<String>,
    pub password: Option<String>,
    pub client_id: Option<String>,
    pub use_custom_client_id: bool,
    pub subscriptions: Vec<Subscription>,
    pub created_at: DateTime<Utc>,
    pub last_connected: Option<DateTime<Utc>>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "New Connection".to_string(),
            protocol: MqttProtocol::default(),
            host: "localhost".to_string(),
            port: 1883,
            version: MqttVersion::default(),
            username: None,
            password: None,
            client_id: None,
            use_custom_client_id: false,
            subscriptions: vec![Subscription::default()],
            created_at: Utc::now(),
            last_connected: None,
        }
    }
}

impl ConnectionConfig {
    #[allow(dead_code)]
    pub fn new(name: &str, host: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            host: host.to_string(),
            port,
            ..Default::default()
        }
    }

    pub fn uri(&self) -> String {
        let auth = match (&self.username, &self.password) {
            (Some(u), Some(_)) => format!("{}@", u),
            (Some(u), None) => format!("{}@", u),
            _ => String::new(),
        };
        format!(
            "{}://{}{}:{}",
            self.protocol.as_str(),
            auth,
            self.host,
            self.port
        )
    }

    pub fn effective_client_id(&self) -> String {
        if self.use_custom_client_id {
            self.client_id
                .clone()
                .unwrap_or_else(|| format!("mqttui-{}", &self.id[..8]))
        } else {
            format!("mqttui-{}", &Uuid::new_v4().to_string()[..8])
        }
    }
}
