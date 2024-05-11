use std::fmt;

use super::Transport;

pub struct TcpTransport {
    fqdn: String,
    port: u16,
    stream: tokio::net::TcpStream,
}

impl TcpTransport {
    pub async fn new(fqdn: String, port: u16) -> tokio::io::Result<Self> {
        let addr = format!("{}:{}", fqdn, port);
        let stream = tokio::net::TcpStream::connect(addr).await?;
        Ok(Self { fqdn, port, stream })
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