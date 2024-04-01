pub mod http;
pub mod tcp;
pub mod tcp_tls;
pub mod udp;

pub trait Transport {
    async fn send(&mut self, data: Vec<u8>) -> tokio::io::Result<()>;
}
