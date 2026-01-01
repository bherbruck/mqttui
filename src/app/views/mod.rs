//! View functions for the MQTT UI application
//!
//! Views are implemented as methods on MqttUi and are organized into separate files:
//! - tabs: Tab bar for connection tabs
//! - home: Home screen with connection cards
//! - connection_form: New/edit connection form
//! - connection: Active connection view with pane grid
//! - publish: Publish panel
//! - topic_tree: Topic tree panel
//! - message: Message panel

mod connection;
mod connection_form;
mod home;
mod message;
mod publish;
mod tabs;
pub mod topic_tree;
