use std::time::Duration;

use crate::{generators::EventGenerator, transports::Transport};

pub struct Sender<T: Transport, G: EventGenerator> {
    pub transport: T,
    pub generator: G,
    pub rate: u64, // events per second
}

impl<T, G> Sender<T, G>
where
    T: Transport + Send + 'static,
    G: EventGenerator + Send + 'static,
{
    pub async fn run(&mut self) -> tokio::io::Result<()> {
        // convert rate (in events per second) to interval (in milliseconds)
        let mut interval = tokio::time::interval(Duration::from_millis(1000 / self.rate));

        loop {
            interval.tick().await;
            let event = self.generator.generate();
            let serialized = event.serialize();
            self.transport.send(serialized).await?;
        }
    }
}
