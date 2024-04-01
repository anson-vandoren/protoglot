pub mod event;
mod serializers;
mod transports;

use event::Event;
use serializers::ndjson::NdJsonSerializer;
use serializers::Serializer;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::interval;
use transports::tcp_tls::TcpTlsTransport;
use transports::Transport;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let fqdn = "default.main.exciting-sharp-d7ds9oz.cribl-staging.cloud";
    let port = 10070;

    let serializer = NdJsonSerializer;
    let mut transport = TcpTlsTransport::new(fqdn.to_string(), port).await?;

    let batch_size = 1000; // Number of messages per batch
    let mut batch = Vec::with_capacity(batch_size); // Vector to store messages before sending
    let mut interval = interval(Duration::from_millis(1)); // Time interval for sending messages if batch_size is not reached

    loop {
        if batch.len() < batch_size {
            let event = Event::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
                "Test message".to_string(),
            );
            let serialized = serializer.serialize(&event);
            batch.push(serialized);
        }

        if batch.len() >= batch_size {
            let batched_message = batch.concat();
            transport.send(batched_message).await?;
            batch.clear();
        } else {
            interval.tick().await;
        }
    }
}
