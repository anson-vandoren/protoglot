use super::Transport;

pub struct UdpTransport {
    socket: tokio::net::UdpSocket,
}

impl UdpTransport {
    pub async fn new(fqdn: String, port: u16) -> tokio::io::Result<Self> {
        let addr = format!("{}:{}", fqdn, port);
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(addr).await?;
        Ok(Self { socket })
    }
}

impl Transport for UdpTransport {
    async fn send(&mut self, data: Vec<u8>) -> tokio::io::Result<()> {
        self.socket.send(&data).await?;
        Ok(())
    }
}
