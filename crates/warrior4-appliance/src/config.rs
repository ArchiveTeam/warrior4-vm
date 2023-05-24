//! Editable configuration file loading

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use serde::Deserialize;

/// The config that gets loaded from the toml config file
#[derive(Deserialize)]
pub struct AppConfig {
    pub log_path: PathBuf,
    pub state_path: PathBuf,
    pub display_ipc_address: SocketAddr,
    pub patch_script_url: Option<String>,

    // Watchtower container
    pub watchtower_name: String,
    pub watchtower_creator: PathBuf,

    // Watchtower container but with --run-once
    pub watchtower_run_once_name: String,
    pub watchtower_run_once_creator: PathBuf,

    // The warrior project management container
    pub payload_name: String,
    pub payload_creator: PathBuf,
    pub payload_pre_start: PathBuf,
    pub payload_post_start: PathBuf,
    pub payload_wait_ready: PathBuf,
    pub payload_reboot_check: PathBuf,
    pub payload_poweroff_check: PathBuf,
    pub payload_ready_message: String,
}

/// Deserialize the config from the given path
pub fn load_config(path: &Path) -> anyhow::Result<AppConfig> {
    tracing::info!("reading configuration");

    let config_text = std::fs::read_to_string(path)?;
    let config = toml::from_str::<AppConfig>(&config_text)?;

    tracing::info!("loaded configuration");

    Ok(config)
}
