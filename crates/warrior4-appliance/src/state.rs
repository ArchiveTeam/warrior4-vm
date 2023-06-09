//! Data serialized to disk for state management

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub uuid: Uuid,
    pub created: DateTime<Utc>,
    pub last_forced_reboot: DateTime<Utc>,
}

impl State {
    pub fn new() -> Self {
        Self {
            uuid: uuid::Uuid::new_v4(),
            created: Utc::now(),
            last_forced_reboot: Default::default(),
        }
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let buf = std::fs::read_to_string(path)?;
        let doc = serde_json::from_str::<State>(&buf)?;

        Ok(doc)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }

        let buf = serde_json::to_string_pretty(self)?;
        std::fs::write(path, buf.as_bytes())?;

        Ok(())
    }
}
