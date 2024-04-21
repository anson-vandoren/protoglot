use std::time::Duration;

use crate::{generators::EventGenerator, serializers::Serializer, transports::Transport};

pub struct Sender<T: Transport, S: Serializer, G: EventGenerator> {
    pub transport: T,
    pub serializer: S,
    pub generator: G,
    pub rate: Duration,
}

impl<T, S, G> Sender<T, S, G>
where
    T: Transport + Send + 'static,
    S: Serializer + Send + 'static,
    G: EventGenerator + Send + 'static,
{
    pub fn new(transport: T, serializer: S, generator: G, rate: Duration) -> Self {
        Self {
            transport,
            serializer,
            generator,
            rate,
        }
    }

    pub async fn run(&mut self) -> tokio::io::Result<()> {
        let mut interval = tokio::time::interval(self.rate);
        loop {
            interval.tick().await;
            let event = self.generator.generate();
            let serialized = self.serializer.serialize(&event);
            self.transport.send(serialized).await?;
        }
    }
}
