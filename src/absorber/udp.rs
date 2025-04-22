use std::sync::Arc;

use tokio::{net::UdpSocket, sync::Mutex};

use super::absorber::{validate_message, AbsorberInner, AbsorberStats};
use crate::config::MessageType;

pub struct UdpAbsorber {
    listener: UdpSocket,
    message_type: MessageType,
}

impl UdpAbsorber {
    pub async fn build(address: &str, port: u16, message_type: MessageType) -> Self {
        let listener = UdpSocket::bind(format!("{}:{}", address, port))
            .await
            .expect("Could not bind to UDP address & port.");
        Self { listener, message_type }
    }

    pub async fn run(self, stats: Arc<Mutex<AbsorberStats>>) -> anyhow::Result<()> {
        let mut buf = [0; 65535];
        loop {
            let (len, _) = self.listener.recv_from(&mut buf).await?;
            let message = &buf[..len];
            self.process_message(message, stats.clone()).await;
        }
    }

    async fn process_message(&self, message: &[u8], stats: Arc<Mutex<AbsorberStats>>) {
        // Validate and process the message
        if validate_message(message, &self.message_type) {
            let mut stats = stats.lock().await;
            stats.total_events += 1;
            stats.intv_events += 1;
            let message_len = message.len() as u64;
            stats.total_bytes += message_len;
            stats.intv_bytes += message_len;
        }
    }
}

impl From<UdpAbsorber> for AbsorberInner {
    fn from(value: UdpAbsorber) -> Self {
        AbsorberInner::Udp(value)
    }
}
