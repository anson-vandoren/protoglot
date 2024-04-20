use std::time::{Duration, SystemTime};

use crate::{event::Event, serializers::Serializer, transports::Transport};

pub struct Sender<T: Transport, S: Serializer> {
    pub transport: T,
    pub serializer: S,
    pub rate: Duration,
}

impl<T, S> Sender<T, S>
where
    T: Transport + Send + 'static,
    S: Serializer + Send + 'static,
{
    pub fn new(transport: T, serializer: S, rate: Duration) -> Self {
        Self {
            transport,
            serializer,
            rate,
        }
    }

    pub async fn run(&mut self) -> tokio::io::Result<()> {
        let mut interval = tokio::time::interval(self.rate);
        loop {
            interval.tick().await;
            let event = Event::new(
                SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
                "Test message".to_string(),
            );
            let serialized = self.serializer.serialize(&event);
            self.transport.send(serialized).await?;
        }
    }
}
