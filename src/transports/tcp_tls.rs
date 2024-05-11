use std::fmt;
use std::sync::Arc;

use rustls::{pki_types::ServerName, ClientConfig, RootCertStore};
use tokio::net::TcpStream;
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

        let root_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));
        let stream = TcpStream::connect(addr).await?;
        let stream = connector.connect(domain, stream).await?;

        Ok(Self { fqdn, port, stream })
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
