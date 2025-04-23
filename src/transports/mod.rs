use std::fmt;

use log::error;

use crate::config::{EmitterConfig, Protocol};

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
    fn send(&mut self, data: Vec<u8>) -> impl std::future::Future<Output = tokio::io::Result<()>> + Send;
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

pub async fn create_transport(config: &EmitterConfig) -> tokio::io::Result<TransportType> {
    match config.protocol {
        Protocol::Tcp => {
            if config.tls {
                match tcp_tls::TcpTlsTransport::new(config.host.clone(), config.port).await {
                    Ok(transport) => Ok(TransportType::TcpTls(transport)),
                    Err(err) => {
                        error!("Failed to create TcpTlsTransport: {}", err);
                        Err(err)
                    }
                }
            } else {
                match tcp::TcpTransport::new(config.host.clone(), config.port).await {
                    Ok(transport) => Ok(TransportType::Tcp(transport)),
                    Err(err) => {
                        error!("Failed to create TcpTransport: {}", err);
                        Err(err)
                    }
                }
            }
        }
        Protocol::Udp => match udp::UdpTransport::new(config.host.clone(), config.port).await {
            Ok(transport) => Ok(TransportType::Udp(transport)),
            Err(err) => {
                error!("Failed to create UdpTransport: {}", err);
                Err(err)
            }
        },
        Protocol::Http => todo!(),
    }
}
