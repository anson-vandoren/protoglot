use std::fmt;

use super::Transport;

pub struct UdpTransport {
    fqdn: String,
    port: u16,
    socket: tokio::net::UdpSocket,
}

impl UdpTransport {
    pub async fn new(fqdn: String, port: u16) -> tokio::io::Result<Self> {
        let addr = format!("{}:{}", fqdn, port);
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

        match socket.connect(&addr).await {
            Ok(_) => Ok(Self { fqdn, port, socket }),
            Err(e) => {
                log::error!("Failed to connect to {}: {}", addr, e);
                Err(e)
            }
        }
    }
}

impl Transport for UdpTransport {
    async fn send(&mut self, data: Vec<u8>) -> tokio::io::Result<()> {
        self.socket.send(&data).await?;
        Ok(())
    }
}

impl fmt::Display for UdpTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "udp/{}:{}", self.fqdn, self.port)
    }
}
