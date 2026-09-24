use std::{net::IpAddr, path::Path};

use serde::Deserialize;

/// target.json config object.
#[derive(Deserialize)]
pub struct TargetConfig {
    /// Popular public DNS resolvers around the world for basic test.
    pub public_dns_resolver: Vec<IpAddr>,
    /// IP addresses of DNS resolvers that ArchiveTeam uses.
    pub cleartext_dns_resolver: Vec<IpAddr>,
    /// The second item is the domain name for DNS-over-TLS.
    pub encrypted_dns_resolver: Vec<(IpAddr, String)>,
    /// A valid domain name.
    pub domain: String,
    /// An invalid domain name template.
    pub nonexistent_domain: String,
    /// The URL template to a HTML download.
    pub url: String,
    /// The contents of the HTML file.
    pub url_content: String,
}

/// Deserialize the config from the given path.
pub fn load_config(path: &Path) -> anyhow::Result<TargetConfig> {
    let config_text = std::fs::read_to_string(path)?;
    let config = serde_json::from_str::<TargetConfig>(&config_text)?;

    Ok(config)
}
