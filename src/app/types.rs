//! Internal types for the MQTT UI application

use std::sync::mpsc;

use iced::widget::pane_grid;

use crate::config::ConnectionConfig;
use crate::mqtt::{ConnectionStatus, MqttMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Publish,
    Topics,
    Message,
}

#[derive(Default, PartialEq, Clone)]
pub enum View {
    #[default]
    Home,
    ConnectionForm {
        editing_id: Option<String>,
    },
    Connection(String),
}

#[allow(dead_code)]
pub struct ConnectionState {
    pub config: ConnectionConfig,
    pub status: ConnectionStatus,
    pub messages: Vec<MqttMessage>,
    pub command_tx: Option<mpsc::Sender<MqttCommand>>,
    pub event_rx: Option<mpsc::Receiver<MqttEvent>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MqttCommand {
    Connect,
    Disconnect,
    Publish(String, Vec<u8>, u8, bool),
}

#[derive(Debug)]
pub enum MqttEvent {
    Connected,
    Disconnected,
    Message(MqttMessage),
    Error(String),
}

/// Info about a tree node for rendering
#[derive(Clone)]
pub struct TreeNodeInfo {
    pub name: String,
    pub full_path: String,
    pub depth: usize,
    pub has_children: bool,
    pub has_messages: bool,
    pub message_count: usize,
    pub is_expanded: bool,
}

/// Create the initial 3-pane layout
pub fn create_pane_layout() -> pane_grid::State<Pane> {
    let (mut panes, publish_pane) = pane_grid::State::new(Pane::Publish);
    let (topics_pane, split1) = panes
        .split(pane_grid::Axis::Vertical, publish_pane, Pane::Topics)
        .unwrap();
    let (_, split2) = panes
        .split(pane_grid::Axis::Vertical, topics_pane, Pane::Message)
        .unwrap();
    // Resize to approximate 20% | 35% | 45%
    panes.resize(split1, 0.2);
    panes.resize(split2, 0.55);
    panes
}
