use std::fmt;

use anyhow::Context;
use log::error;

use super::Transport;

pub struct TcpTransport {
    fqdn: String,
    port: u16,
    stream: tokio::net::TcpStream,
}

impl TcpTransport {
    pub async fn new(fqdn: String, port: u16) -> anyhow::Result<Self> {
        let addr = format!("{}:{}", fqdn, port);
        let ip = tokio::net::lookup_host(addr)
            .await?
            .next()
            .context("Failed to resolve socket address")?;
        match tokio::net::TcpStream::connect(ip).await {
            Ok(stream) => Ok(Self { fqdn, port, stream }),
            Err(e) => {
                error!("Failed to connect to {}: {}", ip, e);
                Err(e.into())
            }
        }
    }
}

impl Transport for TcpTransport {
    async fn send(&mut self, data: Vec<u8>) -> tokio::io::Result<()> {
        tokio::io::AsyncWriteExt::write_all(&mut self.stream, &data).await
    }
}

impl fmt::Display for TcpTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tcp/{}:{}", self.fqdn, self.port)
    }
}
