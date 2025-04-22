use std::sync::Arc;

use log::{debug, trace};
use tokio::{
    io::AsyncReadExt as _,
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use super::absorber::{extract_message, validate_message, AbsorberInner, AbsorberStats};
use crate::config::MessageType;

pub struct TcpAbsorber {
    address: String,
    port: u16,
    message_type: MessageType,
}

impl TcpAbsorber {
    pub async fn build(address: &str, port: u16, message_type: MessageType) -> Self {
        Self {
            message_type,
            address: address.to_string(),
            port,
        }
    }

    pub async fn run(self, stats: Arc<Mutex<AbsorberStats>>) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port))
            .await
            .expect("Could not bind to TCP address & port");

        loop {
            let message_type = self.message_type.clone();
            let (socket, _) = listener.accept().await?;
            let stats = Arc::clone(&stats);
            tokio::spawn(async move {
                debug!("Accepted TCP connection from: {}", socket.peer_addr().unwrap());
                if let Err(e) = handle_tcp_connection(socket, stats, &message_type).await {
                    eprintln!("Error handling TCP connection: {}", e);
                }
            });
        }
    }
}
async fn handle_tcp_connection(
    mut socket: TcpStream,
    stats: Arc<Mutex<AbsorberStats>>,
    message_type: &MessageType,
) -> tokio::io::Result<()> {
    let mut buf = Vec::new();
    loop {
        match socket.read_buf(&mut buf).await {
            Ok(0) => break, // connection closed
            Ok(_) => {
                if let Some(message) = extract_message(&mut buf) {
                    // convert message to string for logging
                    let message_str = String::from_utf8_lossy(&message);
                    trace!("Received message: {:?}", message_str);
                    process_message(&message, stats.clone(), message_type).await;
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

async fn process_message(message: &[u8], stats: Arc<Mutex<AbsorberStats>>, message_type: &MessageType) {
    // Validate and process the message
    if validate_message(message, message_type) {
        let mut stats = stats.lock().await;
        stats.total_events += 1;
        stats.intv_events += 1;
        let message_len = message.len() as u64;
        stats.total_bytes += message_len;
        stats.intv_bytes += message_len;
    }
}

impl From<TcpAbsorber> for AbsorberInner {
    fn from(value: TcpAbsorber) -> Self {
        AbsorberInner::Tcp(value)
    }
}
