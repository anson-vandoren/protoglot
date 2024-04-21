pub mod http;
pub mod tcp;
pub mod tcp_tls;
pub mod udp;

pub trait Transport {
    fn send(&mut self, data: Vec<u8>) -> impl std::future::Future<Output = tokio::io::Result<()>> + Send;
}
