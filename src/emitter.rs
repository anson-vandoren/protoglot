use std::time::Duration;

use crate::{generators::EventGenerator, transports::Transport};

pub struct EmitterConfig {
    pub rate: u64,              // events per second
    pub num_cycles: u64,        // number of cycles to send, 0 means run forever
    pub events_per_cycle: u64, // number of events per cycles
    pub cycle_delay: u64,       // delay between cycles in milliseconds
}

pub struct Emitter<T: Transport, G: EventGenerator> {
    pub transport: T,
    pub generator: G,
    pub config: EmitterConfig,
    cycles_sent: u64,
}

impl<T, G> Emitter<T, G>
where
    T: Transport + Send + 'static,
    G: EventGenerator + Send + 'static,
{
    pub async fn run(&mut self) -> tokio::io::Result<()> {
        // convert rate (in events per second) to interval (in microseconds)
        let mut interval = tokio::time::interval(Duration::from_micros(1000000 / self.config.rate));

        while self.config.num_cycles == 0 || self.cycles_sent < self.config.num_cycles {
            for _ in 0..self.config.events_per_cycle {
                interval.tick().await;
                let event = self.generator.generate();
                let serialized = event.serialize();
                self.transport.send(serialized).await?;
            }
            self.cycles_sent += 1;

            if self.config.cycle_delay > 0 && self.cycles_sent < self.config.num_cycles {
                tokio::time::sleep(Duration::from_millis(self.config.cycle_delay)).await;
            }
        }
        Ok(())
    }
    pub fn new(transport: T, generator: G, config: EmitterConfig) -> Self {
        Emitter {
            transport,
            generator,
            config,
            cycles_sent: 0,
        }
    }
}
