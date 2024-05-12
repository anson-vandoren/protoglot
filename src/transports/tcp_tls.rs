use std::fmt;
use std::sync::Arc;

use log::{debug, error};
use rustls::{pki_types::ServerName, ClientConfig, RootCertStore};
use tokio::net::TcpStream;
use tokio::time::{self, Duration};
use tokio_rustls::TlsConnector;

use super::Transport;

pub struct TcpTlsTransport {
    fqdn: String,
    port: u16,
    stream: tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
}

impl TcpTlsTransport {
    pub async fn new(fqdn: String, port: u16) -> tokio::io::Result<Self> {
        let addr = format!("{}:{}", fqdn, port);
        let domain = ServerName::try_from(fqdn.to_string()).expect("Invalid DNS name");

        let mut root_store = RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().expect("Failed to load native certs") {
            root_store.add(cert).unwrap();
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));

        match TcpStream::connect(&addr).await {
            Ok(tcp_stream) => {
                let handshake_duration = Duration::from_secs(5);
                let handshake_result =
                    time::timeout(handshake_duration, connector.connect(domain, tcp_stream)).await;
                match handshake_result {
                    Ok(Ok(stream)) => {
                        debug!("TLS handshake succeeded to {}", addr);
                        Ok(Self { fqdn, port, stream })
                    }
                    Ok(Err(e)) => {
                        error!("TLS handshake failed to {}: {}", addr, e);
                        Err(e)
                    }
                    Err(_) => {
                        error!("TLS handshake timed out to {}", addr);
                        Err(tokio::io::Error::new(
                            tokio::io::ErrorKind::TimedOut,
                            "TLS handshake timed out",
                        ))
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", addr, e);
                Err(e)
            }
        }
    }
}

impl Transport for TcpTlsTransport {
    async fn send(&mut self, data: Vec<u8>) -> tokio::io::Result<()> {
        tokio::io::AsyncWriteExt::write_all(&mut self.stream, &data).await
    }
}

impl fmt::Display for TcpTlsTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tcp_tls/{}:{}", self.fqdn, self.port)
    }
}
