//! IPC to talk to the warrior4-appliance-display service

use std::{
    net::{SocketAddr, TcpStream},
    time::Duration,
};

use warrior4_appliance_display::IPCRequest;

pub struct DisplayIPC {
    address: SocketAddr,
}

impl DisplayIPC {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    pub fn send_info<S: Into<String>>(&self, text: S) -> anyhow::Result<()> {
        self.send_doc(IPCRequest::Info { text: text.into() })?;
        Ok(())
    }

    pub fn send_warning<S: Into<String>>(&self, text: S) -> anyhow::Result<()> {
        self.send_doc(IPCRequest::Warning { text: text.into() })?;
        Ok(())
    }

    pub fn send_error<S: Into<String>>(&self, text: S) -> anyhow::Result<()> {
        self.send_doc(IPCRequest::Warning { text: text.into() })?;
        Ok(())
    }

    pub fn send_progress<S: Into<String>>(&self, text: S, percent: u8) -> anyhow::Result<()> {
        self.send_doc(IPCRequest::ProgressInfo {
            text: text.into(),
            percent,
        })?;
        Ok(())
    }

    pub fn send_ready<S: Into<String>>(&self, text: S) -> anyhow::Result<()> {
        self.send_doc(IPCRequest::ReadyInfo { text: text.into() })?;
        Ok(())
    }

    fn connect(&self) -> anyhow::Result<TcpStream> {
        let stream = TcpStream::connect_timeout(&self.address, Duration::from_secs(1))?;

        stream.set_nodelay(true)?;

        Ok(stream)
    }

    fn send_doc(&self, api_doc: IPCRequest) -> anyhow::Result<()> {
        let stream = self.connect()?;
        serde_json::to_writer(stream, &api_doc)?;

        Ok(())
    }
}
