use rustls::{pki_types::ServerName, ClientConfig, RootCertStore};
use serde_json::json;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio::time::interval;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let fqdn = "default.main.exciting-sharp-d7ds9oz.cribl-staging.cloud";
    let port = 10070;
    let addr = format!("{}:{}", fqdn, port);

    let domain = ServerName::try_from(fqdn).expect("Invalid DNS name");

    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let stream = TcpStream::connect(addr).await?;
    let mut stream = connector.connect(domain, stream).await?;

    let mut batch = Vec::new();
    let batch_size = 1000; // Number of messages per batch
    let mut interval = interval(Duration::from_millis(1)); // Time interval for sending messages if batch_size is not reached

    loop {
        if batch.len() < batch_size {
            let message = json!({
                "_time": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                "message": "Hello, world!"
            })
            .to_string();
            batch.push(message);
        }

        if batch.len() >= batch_size {
            let batched_message = batch.join("\n") + "\n"; // Join messages with newline and add one at the end
            if let Err(e) = stream.write_all(batched_message.as_bytes()).await {
                eprintln!("Failed to write to stream: {}", e);
                return Err(e);
            }
            batch.clear(); // Clear the batch after sending
        } else {
            interval.tick().await; // Wait for the next interval tick if the batch is not full
        }
    }
}
