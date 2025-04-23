use tokio::net::UdpSocket;

use super::{process_message, AbsorberInner, StatsSvc};
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

    pub(super) async fn run(self, stats: StatsSvc) -> anyhow::Result<()> {
        let mut buf = [0; 65535];
        loop {
            let (len, _) = self.listener.recv_from(&mut buf).await?;
            let message = &buf[..len];
            process_message(message, &stats.clone(), &self.message_type).await;
        }
    }
}

impl From<UdpAbsorber> for AbsorberInner {
    fn from(value: UdpAbsorber) -> Self {
        AbsorberInner::Udp(value)
    }
}
