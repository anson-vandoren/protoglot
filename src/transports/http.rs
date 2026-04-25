use std::fmt;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use super::Transport;

pub struct HttpTransport {
    client: reqwest::Client,
    url: String,
    hec_token: Option<String>,
}

impl HttpTransport {
    pub fn new(protocol: &str, fqdn: String, port: u16, hec_token: Option<String>) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().build()?;
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

        let mut transport = HttpTransport::new("http", "127.0.0.1".to_string(), port, Some("test-token".to_string())).unwrap();
        transport.send(b"{\"event\":\"hello\"}\n").await.unwrap();

        let request = server.await.unwrap();
        assert!(request.starts_with("POST /services/collector/event HTTP/1.1"));
        assert!(request.contains("authorization: Splunk test-token"));
        assert!(request.contains("content-type: application/json"));
        assert!(request.ends_with("{\"event\":\"hello\"}\n"));
    }
}
