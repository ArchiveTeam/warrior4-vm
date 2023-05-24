//! IPC socket to allow the appliance management program to talk to us

use std::{
    io::{BufRead, BufReader, BufWriter},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::Sender,
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

pub enum IPCEvent {
    Progress { text: String, percent: u8 },
    Ready { text: String },
}

pub fn run(channel: Sender<IPCEvent>, address: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address)?;

    loop {
        match listener.accept() {
            Ok((stream, _addr)) => {
                let channel = channel.clone();
                std::thread::spawn(|| match handle_client(stream, channel) {
                    Ok(_) => {}
                    Err(error) => eprintln!("{}", error),
                });
            }
            Err(_) => {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn handle_client(stream: TcpStream, channel: Sender<IPCEvent>) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut _writer = BufWriter::new(stream);
    let mut buf = String::new();

    loop {
        buf.clear();
        let amount = reader.read_line(&mut buf)?;

        if amount == 0 {
            break;
        }

        let doc = serde_json::from_str::<APIDoc>(&buf)?;

        match doc.request.as_str() {
            "progress" => {
                channel.send(IPCEvent::Progress {
                    text: doc.text.unwrap_or_default(),
                    percent: doc.percent.unwrap_or_default(),
                })?;
            }
            "ready" => {
                channel.send(IPCEvent::Ready {
                    text: doc.text.unwrap_or_default(),
                })?;
            }
            _ => {}
        }
    }

    Ok(())
}
