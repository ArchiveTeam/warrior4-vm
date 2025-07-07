//! IPC socket to allow the appliance management program to talk to us

use std::{
    io::{BufRead, BufReader, BufWriter},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::Sender,
    time::Duration,
};

use crate::api::Request;

pub fn run(channel: Sender<Request>, address: SocketAddr) -> anyhow::Result<()> {
    let listener = TcpListener::bind(address)?;

    loop {
        match listener.accept() {
            Ok((stream, _addr)) => {
                let channel = channel.clone();
                std::thread::spawn(|| match handle_client(stream, channel) {
                    Ok(_) => {}
                    Err(error) => eprintln!("{error}"),
                });
            }
            Err(_) => {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn handle_client(stream: TcpStream, channel: Sender<Request>) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut _writer = BufWriter::new(stream);
    let mut buf = String::new();

    loop {
        buf.clear();
        let amount = reader.read_line(&mut buf)?;

        if amount == 0 {
            break;
        }

        let doc = serde_json::from_str::<Request>(&buf)?;
        channel.send(doc)?;
    }

    Ok(())
}
