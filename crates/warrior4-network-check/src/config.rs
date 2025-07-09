use std::{net::IpAddr, path::Path};

use serde::Deserialize;

/// target.json config object
#[derive(Deserialize)]
pub struct TargetConfig {
    pub bootstrap_dns: Vec<IpAddr>,
    pub nonexistent_url: String,
    pub cleartext_url: String,
    pub target_url: String,
    pub content: String,
}

/// Deserialize the config from the given path.
pub fn load_config(path: &Path) -> anyhow::Result<TargetConfig> {
    let config_text = std::fs::read_to_string(path)?;
    let config = serde_json::from_str::<TargetConfig>(&config_text)?;

    Ok(config)
}
