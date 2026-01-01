//! MQTT worker thread for handling broker connections

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::config::ConnectionConfig;
use crate::mqtt::MqttMessage;

use super::types::MqttEvent;

/// Run the MQTT worker - handles connection, subscriptions, and message routing
pub fn run_mqtt_worker(
    config: ConnectionConfig,
    cmd_rx: mpsc::Receiver<super::types::MqttCommand>,
    evt_tx: mpsc::SyncSender<MqttEvent>,
) {
    use rumqttc::{Client, Event, MqttOptions, Packet, QoS};

    let client_id = config.effective_client_id();

    // Try localhost fallback to 127.0.0.1 on Windows
    let hosts_to_try: Vec<&str> = if config.host.eq_ignore_ascii_case("localhost") {
        vec!["localhost", "127.0.0.1"]
    } else {
        vec![config.host.as_str()]
    };

    let mut last_error = None;
    let mut client_and_connection = None;

    for host in &hosts_to_try {
        let mut mqttoptions = MqttOptions::new(&client_id, *host, config.port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));
        // Increase internal buffer for high-volume brokers
        mqttoptions.set_max_packet_size(256 * 1024, 256 * 1024);

        if let (Some(username), password) = (&config.username, &config.password) {
            mqttoptions.set_credentials(username, password.clone().unwrap_or_default());
        }

        let (client, mut connection) = Client::new(mqttoptions, 500);

        // Try to get first event to verify connection works
        match connection.iter().next() {
            Some(Ok(event)) => {
                tracing::info!("Connected to MQTT broker at {}:{}", host, config.port);
                // Put the event back by processing it
                client_and_connection = Some((client, connection, Some(event)));
                break;
            }
            Some(Err(e)) => {
                tracing::warn!("Failed to connect to {}:{}: {}", host, config.port, e);
                last_error = Some(e.to_string());
            }
            None => {
                tracing::warn!("Connection to {}:{} closed immediately", host, config.port);
                last_error = Some("Connection closed".to_string());
            }
        }
    }

    let (client, mut connection, first_event) = match client_and_connection {
        Some(c) => c,
        None => {
            let _ = evt_tx.try_send(MqttEvent::Error(
                last_error.unwrap_or_else(|| "Failed to connect".to_string()),
            ));
            return;
        }
    };

    // Spawn command handler
    let client_clone = client.clone();
    let subscriptions = config.subscriptions.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(500));

        // Subscribe to configured topics
        for sub in &subscriptions {
            let qos = match sub.qos {
                0 => QoS::AtMostOnce,
                1 => QoS::AtLeastOnce,
                _ => QoS::ExactlyOnce,
            };
            if let Err(e) = client_clone.subscribe(&sub.topic, qos) {
                tracing::warn!("Failed to subscribe to {}: {}", sub.topic, e);
            }
        }

        // Don't subscribe to # if we already have subscriptions - it's redundant and causes message floods
        if subscriptions.is_empty() {
            let _ = client_clone.subscribe("#", QoS::AtMostOnce);
        }

        for cmd in cmd_rx {
            match cmd {
                super::types::MqttCommand::Connect => {}
                super::types::MqttCommand::Disconnect => {
                    let _ = client_clone.disconnect();
                    break;
                }
                super::types::MqttCommand::Publish(topic, payload, qos, retain) => {
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

    // Helper to process a single event
    let process_event = |event: &Event, evt_tx: &mpsc::SyncSender<MqttEvent>| -> bool {
        match event {
            Event::Incoming(Packet::ConnAck(_)) => {
                if evt_tx.try_send(MqttEvent::Connected).is_err() {
                    tracing::warn!("Event channel full, dropping connected event");
                }
                true
            }
            Event::Incoming(Packet::Publish(publish)) => {
                let msg = MqttMessage::new(
                    publish.topic.to_string(),
                    publish.payload.to_vec(),
                    publish.qos as u8,
                    publish.retain,
                );
                // Use try_send to avoid blocking if channel is full
                if evt_tx.try_send(MqttEvent::Message(msg)).is_err() {
                    // Channel full - drop message to prevent backpressure
                    tracing::trace!("Event channel full, dropping message");
                }
                true
            }
            Event::Incoming(Packet::Disconnect) => {
                let _ = evt_tx.try_send(MqttEvent::Disconnected);
                false // stop processing
            }
            _ => true,
        }
    };

    // Process the first event we got during connection testing
    if let Some(event) = first_event {
        if !process_event(&event, &evt_tx) {
            return;
        }
    }

    // Event loop with error recovery
    for notification in connection.iter() {
        match notification {
            Ok(event) => {
                if !process_event(&event, &evt_tx) {
                    break;
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                tracing::error!("MQTT connection error: {}", error_msg);
                let _ = evt_tx.try_send(MqttEvent::Error(error_msg));
                break;
            }
        }
    }

    let _ = evt_tx.try_send(MqttEvent::Disconnected);
}
