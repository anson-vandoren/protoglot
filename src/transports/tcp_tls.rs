use std::{fmt, path::Path, sync::Arc};

use anyhow::Context as _;
use log::{debug, error};
use rustls::{
    ClientConfig, RootCertStore,
    pki_types::{CertificateDer, PrivateKeyDer, ServerName, pem::PemObject as _},
};
use tokio::{
    net::TcpStream,
    time::{self, Duration},
};
use tokio_rustls::TlsConnector;

use super::Transport;

pub struct TcpTlsTransport {
    fqdn: String,
    port: u16,
    stream: tokio_rustls::client::TlsStream<tokio::net::TcpStream>,
}

impl TcpTlsTransport {
    pub async fn new(
        fqdn: String,
        port: u16,
        client_cert: Option<&Path>,
        client_key: Option<&Path>,
        ca_cert: Option<&Path>,
    ) -> anyhow::Result<Self> {
        let addr = (fqdn.as_str(), port);
        let domain = ServerName::try_from(fqdn.to_string()).context("Invalid TLS server name")?;

        let mut root_store = RootCertStore::empty();
        let native_certs = rustls_native_certs::load_native_certs();
        if !native_certs.errors.is_empty() {
            anyhow::bail!("Failed to load native certificates: {:?}", native_certs.errors);
        }
        for cert in native_certs.certs {
            root_store.add(cert).context("Failed to add native root certificate")?;
        }
        if let Some(path) = ca_cert {
            let pem = std::fs::read(path).with_context(|| format!("Failed to read CA certificate '{}'", path.display()))?;
            let certs = CertificateDer::pem_slice_iter(&pem)
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| format!("Failed to parse CA certificate '{}'", path.display()))?;
            if certs.is_empty() {
                anyhow::bail!("No certificates found in CA certificate '{}'", path.display());
            }
            for cert in certs {
                root_store
                    .add(cert)
                    .with_context(|| format!("Failed to add CA certificate '{}'", path.display()))?;
            }
        }

        let builder = ClientConfig::builder().with_root_certificates(root_store);
        let config = match (client_cert, client_key) {
            (Some(cert_path), Some(key_path)) => {
                let cert_pem =
                    std::fs::read(cert_path).with_context(|| format!("Failed to read client certificate '{}'", cert_path.display()))?;
                let certs = CertificateDer::pem_slice_iter(&cert_pem)
                    .collect::<Result<Vec<_>, _>>()
                    .with_context(|| format!("Failed to parse client certificate '{}'", cert_path.display()))?;
                let key_pem = std::fs::read(key_path).with_context(|| format!("Failed to read client key '{}'", key_path.display()))?;
                let key = PrivateKeyDer::from_pem_slice(&key_pem)
                    .with_context(|| format!("Failed to parse client key '{}'", key_path.display()))?;
                builder.with_client_auth_cert(certs, key).with_context(|| {
                    format!(
                        "Failed to configure client certificate '{}' and key '{}'",
                        cert_path.display(),
                        key_path.display()
                    )
                })?
            }
            (None, None) => builder.with_no_client_auth(),
            _ => anyhow::bail!("Client certificate and key must be configured together"),
        };
        let connector = TlsConnector::from(Arc::new(config));

        let ip = tokio::net::lookup_host(addr)
            .await?
            .next()
            .context("Failed to resolve socket address")?;

        match TcpStream::connect(ip).await {
            Ok(tcp_stream) => {
                let handshake_duration = Duration::from_secs(5);
                let handshake_result = time::timeout(handshake_duration, connector.connect(domain, tcp_stream)).await;
                match handshake_result {
                    Ok(Ok(stream)) => {
                        debug!("TLS handshake succeeded to {ip}");
                        Ok(Self { fqdn, port, stream })
                    }
                    Ok(Err(e)) => {
                        error!("TLS handshake failed to {ip}: {e}");
                        Err(e.into())
                    }
                    Err(_) => {
                        error!("TLS handshake timed out to {ip}");
                        Err(anyhow::anyhow!("TLS handshake timed out"))
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to {ip}: {e}");
                Err(e.into())
            }
        }
    }
}

impl Transport for TcpTlsTransport {
    async fn send(&mut self, data: &[u8]) -> tokio::io::Result<()> {
        tokio::io::AsyncWriteExt::write_all(&mut self.stream, data).await
    }

    async fn shutdown(&mut self) -> tokio::io::Result<()> {
        tokio::io::AsyncWriteExt::shutdown(&mut self.stream).await
    }
}

impl fmt::Display for TcpTlsTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tcp_tls/{}:{}", self.fqdn, self.port)
    }
}
