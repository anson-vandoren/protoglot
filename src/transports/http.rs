use std::{fmt, path::Path};

use anyhow::Context as _;
use reqwest::{
    Certificate, Identity,
    header::{AUTHORIZATION, CONTENT_TYPE},
};

use super::Transport;

pub struct HttpTransport {
    client: reqwest::Client,
    url: String,
    hec_token: Option<String>,
}

impl HttpTransport {
    pub fn new(
        protocol: &str,
        fqdn: String,
        port: u16,
        hec_token: Option<String>,
        client_cert: Option<&Path>,
        client_key: Option<&Path>,
        ca_cert: Option<&Path>,
    ) -> anyhow::Result<Self> {
        let mut builder = reqwest::Client::builder();
        if let Some(path) = ca_cert {
            let pem = std::fs::read(path).with_context(|| format!("Failed to read CA certificate '{}'", path.display()))?;
            let certs =
                Certificate::from_pem_bundle(&pem).with_context(|| format!("Failed to parse CA certificate '{}'", path.display()))?;
            if certs.is_empty() {
                anyhow::bail!("No certificates found in CA certificate '{}'", path.display());
            }
            for cert in certs {
                builder = builder.add_root_certificate(cert);
            }
        }
        match (client_cert, client_key) {
            (Some(cert_path), Some(key_path)) => {
                let cert =
                    std::fs::read(cert_path).with_context(|| format!("Failed to read client certificate '{}'", cert_path.display()))?;
                let key = std::fs::read(key_path).with_context(|| format!("Failed to read client key '{}'", key_path.display()))?;
                let mut identity_pem = Vec::with_capacity(cert.len() + key.len() + 1);
                identity_pem.extend_from_slice(&cert);
                identity_pem.push(b'\n');
                identity_pem.extend_from_slice(&key);
                let identity = Identity::from_pem(&identity_pem).with_context(|| {
                    format!(
                        "Failed to parse client certificate '{}' and key '{}'",
                        cert_path.display(),
                        key_path.display()
                    )
                })?;
                builder = builder.identity(identity);
            }
            (None, None) => {}
            _ => anyhow::bail!("Client certificate and key must be configured together"),
        }
        let client = builder.build()?;
        let url = format!("{protocol}://{fqdn}:{port}/services/collector/event");

        Ok(Self { client, url, hec_token })
    }
}

impl Transport for HttpTransport {
    async fn send(&mut self, data: &[u8]) -> tokio::io::Result<()> {
        let mut request = self
            .client
            .post(&self.url)
            .header(CONTENT_TYPE, "application/json")
            .body(data.to_vec());

        if let Some(token) = &self.hec_token {
            request = request.header(AUTHORIZATION, format!("Splunk {token}"));
        }

        let response = request.send().await.map_err(tokio::io::Error::other)?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(tokio::io::Error::other(format!(
                "HTTP emitter received non-success status {} from {}",
                response.status(),
                self.url
            )))
        }
    }

    async fn shutdown(&mut self) -> tokio::io::Result<()> {
        Ok(())
    }
}

impl fmt::Display for HttpTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "http/{}", self.url)
    }
}

#[cfg(test)]
mod tests {
    use tokio::{
        io::{AsyncReadExt as _, AsyncWriteExt as _},
        net::TcpListener,
    };

    use super::*;

    #[tokio::test]
    async fn sends_json_post_with_splunk_authorization() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = vec![0; 4096];
            let len = socket.read(&mut buf).await.unwrap();
            socket.write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\nOK").await.unwrap();
            String::from_utf8(buf[..len].to_vec()).unwrap()
        });

        let mut transport = HttpTransport::new(
            "http",
            "127.0.0.1".to_string(),
            port,
            Some("test-token".to_string()),
            None,
            None,
            None,
        )
        .unwrap();
        transport.send(b"{\"event\":\"hello\"}\n").await.unwrap();

        let request = server.await.unwrap();
        assert!(request.starts_with("POST /services/collector/event HTTP/1.1"));
        assert!(request.contains("authorization: Splunk test-token"));
        assert!(request.contains("content-type: application/json"));
        assert!(request.ends_with("{\"event\":\"hello\"}\n"));
    }
}
