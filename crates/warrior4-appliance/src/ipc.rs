//! IPC to talk to the warrior4-appliance-display service

use std::{
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use serde::{Deserialize, Serialize};

/// The JSON object that gets serialized
#[derive(Debug, Clone, Serialize, Deserialize)]
struct APIDoc {
    request: String,
    text: Option<String>,
    percent: Option<u8>, // 0 to 100
}

pub struct DisplayIPC {
    address: SocketAddr,
}

impl DisplayIPC {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    pub fn send_progress<S: Into<String>>(&self, text: S, percent: u8) -> anyhow::Result<()> {
        self.send_doc(APIDoc {
            request: "progress".to_string(),
            text: Some(text.into()),
            percent: Some(percent),
        })?;
        Ok(())
    }

    pub fn send_ready<S: Into<String>>(&self, text: S) -> anyhow::Result<()> {
        self.send_doc(APIDoc {
            request: "ready".to_string(),
            text: Some(text.into()),
            percent: None,
        })?;
        Ok(())
    }

    fn connect(&self) -> anyhow::Result<TcpStream> {
        let stream = TcpStream::connect_timeout(&self.address, Duration::from_secs(1))?;

        stream.set_nodelay(true)?;

        Ok(stream)
    }

    fn send_doc(&self, api_doc: APIDoc) -> anyhow::Result<()> {
        let stream = self.connect()?;
        serde_json::to_writer(stream, &api_doc)?;

        Ok(())
    }
}
