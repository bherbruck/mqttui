use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use super::connection::ConnectionConfig;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub connections: Vec<ConnectionConfig>,
    pub last_opened_tabs: Vec<String>, // Connection IDs
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from {:?}", path))?;
            let config: AppConfig =
                serde_json::from_str(&content).with_context(|| "Failed to parse config JSON")?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }
        let content =
            serde_json::to_string_pretty(self).with_context(|| "Failed to serialize config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {:?}", path))?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "mqttui", "mqttui")
            .context("Failed to determine config directory")?;
        Ok(proj_dirs.config_dir().join("config.json"))
    }

    pub fn add_connection(&mut self, config: ConnectionConfig) {
        self.connections.push(config);
    }

    pub fn update_connection(&mut self, config: ConnectionConfig) {
        if let Some(existing) = self.connections.iter_mut().find(|c| c.id == config.id) {
            *existing = config;
        }
    }

    pub fn remove_connection(&mut self, id: &str) {
        self.connections.retain(|c| c.id != id);
        self.last_opened_tabs.retain(|tab_id| tab_id != id);
    }

    pub fn get_connection(&self, id: &str) -> Option<&ConnectionConfig> {
        self.connections.iter().find(|c| c.id == id)
    }

    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut ConnectionConfig> {
        self.connections.iter_mut().find(|c| c.id == id)
    }
}
