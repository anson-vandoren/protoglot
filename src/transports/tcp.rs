use super::Transport;

pub struct TcpTransport {
    stream: tokio::net::TcpStream,
}

impl TcpTransport {
    pub async fn new(fqdn: String, port: u16) -> tokio::io::Result<Self> {
        let addr = format!("{}:{}", fqdn, port);
        let stream = tokio::net::TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }
}

impl Transport for TcpTransport {
    async fn send(&mut self, data: Vec<u8>) -> tokio::io::Result<()> {
        tokio::io::AsyncWriteExt::write_all(&mut self.stream, &data).await
    }
}
