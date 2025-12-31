use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

use crate::config::ConnectionConfig;

use super::message::{ConnectionStatus, MqttMessage};

#[derive(Debug)]
#[allow(dead_code)]
pub enum MqttCommand {
    Connect,
    Disconnect,
    Subscribe(String, u8),
    Unsubscribe(String),
    Publish(String, Vec<u8>, u8, bool),
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MqttEvent {
    Connected,
    Disconnected,
    Message(MqttMessage),
    Error(String),
    Subscribed(String),
}

pub struct MqttConnection {
    pub config: ConnectionConfig,
    pub status: ConnectionStatus,
    pub messages: Vec<MqttMessage>,
    pub message_count: usize,
    command_tx: Option<Sender<MqttCommand>>,
    event_rx: Option<Receiver<MqttEvent>>,
    _handle: Option<thread::JoinHandle<()>>,
}

impl MqttConnection {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            config,
            status: ConnectionStatus::Disconnected,
            messages: Vec::new(),
            message_count: 0,
            command_tx: None,
            event_rx: None,
            _handle: None,
        }
    }

    pub fn connect(&mut self) {
        if self.command_tx.is_some() {
            return;
        }

        let (cmd_tx, cmd_rx) = mpsc::channel::<MqttCommand>();
        let (evt_tx, evt_rx) = mpsc::channel::<MqttEvent>();

        self.command_tx = Some(cmd_tx);
        self.event_rx = Some(evt_rx);
        self.status = ConnectionStatus::Connecting;

        let config = self.config.clone();

        let handle = thread::spawn(move || {
            Self::mqtt_worker(config, cmd_rx, evt_tx);
        });

        self._handle = Some(handle);

        // Send initial connect command
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(MqttCommand::Connect);
        }
    }

    fn mqtt_worker(
        config: ConnectionConfig,
        cmd_rx: Receiver<MqttCommand>,
        evt_tx: Sender<MqttEvent>,
    ) {
        let client_id = config.effective_client_id();
        let mut mqttoptions = MqttOptions::new(&client_id, &config.host, config.port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));

        if let (Some(username), password) = (&config.username, &config.password) {
            mqttoptions.set_credentials(username, password.clone().unwrap_or_default());
        }

        let (client, mut connection) = Client::new(mqttoptions, 100);

        // Spawn command handler
        let client_clone = client.clone();
        let subscriptions = config.subscriptions.clone();
        thread::spawn(move || {
            // Wait a bit for connection to establish
            thread::sleep(Duration::from_millis(500));

            // Subscribe to configured topics
            for sub in &subscriptions {
                let qos = match sub.qos {
                    0 => QoS::AtMostOnce,
                    1 => QoS::AtLeastOnce,
                    _ => QoS::ExactlyOnce,
                };
                let _ = client_clone.subscribe(&sub.topic, qos);
            }

            // Handle commands
            for cmd in cmd_rx {
                match cmd {
                    MqttCommand::Connect => {}
                    MqttCommand::Disconnect => {
                        let _ = client_clone.disconnect();
                        break;
                    }
                    MqttCommand::Subscribe(topic, qos) => {
                        let qos = match qos {
                            0 => QoS::AtMostOnce,
                            1 => QoS::AtLeastOnce,
                            _ => QoS::ExactlyOnce,
                        };
                        let _ = client_clone.subscribe(&topic, qos);
                    }
                    MqttCommand::Unsubscribe(topic) => {
                        let _ = client_clone.unsubscribe(&topic);
                    }
                    MqttCommand::Publish(topic, payload, qos, retain) => {
                        let qos = match qos {
                            0 => QoS::AtMostOnce,
                            1 => QoS::AtLeastOnce,
                            _ => QoS::ExactlyOnce,
                        };
                        let _ = client_clone.publish(&topic, qos, retain, payload);
                    }
                }
            }
        });

        // Event loop
        for notification in connection.iter() {
            match notification {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    let _ = evt_tx.send(MqttEvent::Connected);
                }
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    let msg = MqttMessage::new(
                        publish.topic.to_string(),
                        publish.payload.to_vec(),
                        publish.qos as u8,
                        publish.retain,
                    );
                    let _ = evt_tx.send(MqttEvent::Message(msg));
                }
                Ok(Event::Incoming(Packet::SubAck(suback))) => {
                    let _ = evt_tx.send(MqttEvent::Subscribed(format!(
                        "Subscribed (pkid: {})",
                        suback.pkid
                    )));
                }
                Ok(Event::Incoming(Packet::Disconnect)) => {
                    let _ = evt_tx.send(MqttEvent::Disconnected);
                    break;
                }
                Err(e) => {
                    let _ = evt_tx.send(MqttEvent::Error(e.to_string()));
                    break;
                }
                _ => {}
            }
        }

        let _ = evt_tx.send(MqttEvent::Disconnected);
    }

    pub fn disconnect(&mut self) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(MqttCommand::Disconnect);
        }
        self.command_tx = None;
        self.event_rx = None;
        self.status = ConnectionStatus::Disconnected;
    }

    pub fn publish(&mut self, topic: &str, payload: &[u8], qos: u8, retain: bool) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(MqttCommand::Publish(
                topic.to_string(),
                payload.to_vec(),
                qos,
                retain,
            ));
        }
    }

    #[allow(dead_code)]
    pub fn subscribe(&mut self, topic: &str, qos: u8) {
        if let Some(tx) = &self.command_tx {
            let _ = tx.send(MqttCommand::Subscribe(topic.to_string(), qos));
        }
    }

    pub fn poll_events(&mut self) {
        if let Some(rx) = &self.event_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    MqttEvent::Connected => {
                        self.status = ConnectionStatus::Connected;
                    }
                    MqttEvent::Disconnected => {
                        self.status = ConnectionStatus::Disconnected;
                    }
                    MqttEvent::Message(msg) => {
                        self.messages.push(msg);
                        self.message_count += 1;
                        // Keep last 10000 messages
                        if self.messages.len() > 10000 {
                            self.messages.remove(0);
                        }
                    }
                    MqttEvent::Error(e) => {
                        self.status = ConnectionStatus::Error(e);
                    }
                    MqttEvent::Subscribed(_) => {}
                }
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.status.is_connected()
    }
}
