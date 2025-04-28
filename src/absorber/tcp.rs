use log::{debug, trace};
use tokio::{
    io::AsyncReadExt as _,
    net::{TcpListener, TcpStream},
};

use super::{extract_message, AbsorberInner, ConnOptions, StatsSvc};
use crate::{absorber::process_message, config::MessageType};

pub struct TcpAbsorber {
    address: String,
    port: u16,
    message_type: MessageType,
}

impl TcpAbsorber {
    pub async fn build(opts: ConnOptions, message_type: MessageType) -> Self {
        Self {
            message_type,
            address: opts.addr.host.to_string(),
            port: opts.addr.port,
        }
    }

    pub(super) async fn run(self, stats: StatsSvc) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port))
            .await
            .expect("Could not bind to TCP address & port");

        loop {
            let message_type = self.message_type.clone();
            let (socket, _) = listener.accept().await?;
            let stats = stats.clone();
            tokio::spawn(async move {
                debug!("Accepted TCP connection from: {}", socket.peer_addr().unwrap());
                if let Err(e) = handle_tcp_connection(socket, &stats, &message_type).await {
                    eprintln!("Error handling TCP connection: {}", e);
                }
            });
        }
    }
}
async fn handle_tcp_connection(mut socket: TcpStream, stats: &StatsSvc, message_type: &MessageType) -> tokio::io::Result<()> {
    let mut buf = Vec::new();
    loop {
        match socket.read_buf(&mut buf).await {
            Ok(0) => break, // connection closed
            Ok(_) => {
                if let Some(message) = extract_message(&mut buf, false) {
                    // convert message to string for logging
                    let message_str = String::from_utf8_lossy(&message);
                    trace!("Received message: {:?}", message_str);
                    process_message(&message, stats, message_type).await;
                }
            }
            Err(err) => {
                eprintln!("Error reading from socket: {}", err);
                break;
            }
        }
    }
    Ok(())
}

impl From<TcpAbsorber> for AbsorberInner {
    fn from(value: TcpAbsorber) -> Self {
        AbsorberInner::Tcp(value)
    }
}
