use std::fmt;

pub mod http;
pub mod tcp;
pub mod tcp_tls;
pub mod udp;

pub enum TransportType {
    Tcp(tcp::TcpTransport),
    TcpTls(tcp_tls::TcpTlsTransport),
    Udp(udp::UdpTransport),
}

impl Transport for TransportType {
    async fn send(&mut self, data: Vec<u8>) -> tokio::io::Result<()> {
        match self {
            TransportType::Tcp(transport) => transport.send(data).await,
            TransportType::TcpTls(transport) => transport.send(data).await,
            TransportType::Udp(transport) => transport.send(data).await,
        }
    }
}

pub trait Transport: Send {
    fn send(
        &mut self,
        data: Vec<u8>,
    ) -> impl std::future::Future<Output = tokio::io::Result<()>> + Send;
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportType::Tcp(transport) => write!(f, "{}", transport),
            TransportType::TcpTls(transport) => write!(f, "{}", transport),
            TransportType::Udp(transport) => write!(f, "{}", transport),
        }
    }
}
