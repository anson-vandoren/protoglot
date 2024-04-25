use std::time::Duration;

use crate::{generators::EventGenerator, transports::Transport};

pub struct SenderConfig {
    pub rate: u64, // events per second
    pub num_batches: u64, // number of batches to send, 0 means run forever
    pub events_per_batch: u64, // number of events per batch
    pub batch_delay: u64, // delay between batches in milliseconds
}

pub struct Sender<T: Transport, G: EventGenerator> {
    pub transport: T,
    pub generator: G,
    pub config: SenderConfig,
    batches_sent: u64,
}


impl<T, G> Sender<T, G>
where
    T: Transport + Send + 'static,
    G: EventGenerator + Send + 'static,
{
    pub async fn run(&mut self) -> tokio::io::Result<()> {
        // convert rate (in events per second) to interval (in microseconds)
        let mut interval = tokio::time::interval(Duration::from_micros(1000000 / self.config.rate));

        while self.config.num_batches == 0 || self.batches_sent < self.config.num_batches {
            for _ in 0..self.config.events_per_batch {
                interval.tick().await;
                let event = self.generator.generate();
                let serialized = event.serialize();
                self.transport.send(serialized).await?;
            }
            self.batches_sent += 1;

            if self.config.batch_delay > 0  && self.batches_sent < self.config.num_batches {
                tokio::time::sleep(Duration::from_millis(self.config.batch_delay)).await;
            }
        }
        Ok(())
    }
    pub fn new(transport: T, generator: G, config: SenderConfig) -> Self {
        Sender {
            transport,
            generator,
            config,
            batches_sent: 0,
        }
    }
}
