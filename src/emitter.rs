use std::time::{Duration, Instant};

use human_bytes::human_bytes;
use log::info;

use crate::{generators::EventGenerator, transports::Transport};

pub struct EmitterConfig {
    pub rate: u64,             // events per second
    pub num_cycles: u64,       // number of cycles to send, 0 means run forever
    pub events_per_cycle: u64, // number of events per cycles
    pub cycle_delay: u64,      // delay between cycles in milliseconds
}

pub struct Emitter<T: Transport, G: EventGenerator> {
    pub transport: T,
    pub generator: G,
    pub config: EmitterConfig,
    cycles_sent: u64,
    total_events: u64,
    total_bytes: u64,
}

impl<T, G> Emitter<T, G>
where
    T: Transport + Send + 'static + std::fmt::Display,
    G: EventGenerator + Send + 'static,
{
    pub async fn run(&mut self) -> tokio::io::Result<()> {
        let start_time = Instant::now();
        let mut next_tick = Instant::now();
        let interval = Duration::from_nanos(1_000_000_000 / self.config.rate);

        while self.config.num_cycles == 0 || self.cycles_sent < self.config.num_cycles {
            for _ in 0..self.config.events_per_cycle {
                let serialized = self.generator.generate_bytes();
                self.total_bytes += serialized.len() as u64;
                self.total_events += 1;
                self.transport.send(serialized).await?;

                next_tick += interval;
                tokio::time::sleep_until(next_tick.into()).await;
            }
            self.cycles_sent += 1;

            if self.config.cycle_delay > 0 && self.cycles_sent < self.config.num_cycles {
                tokio::time::sleep(Duration::from_millis(self.config.cycle_delay)).await;
            }
        }

        let duration = start_time.elapsed();
        let duration_secs = duration.as_secs_f64();
        let events_per_sec = self.total_events as f64 / duration_secs;
        info!(emitter=self.transport.to_string(); "{:.0} events/s average", events_per_sec);
        let bytes_per_sec = self.total_bytes as f64 / duration_secs;
        let formatted_bytes = human_bytes(bytes_per_sec);
        info!(emitter=self.transport.to_string(); "{}/s average", formatted_bytes);
        Ok(())
    }

    pub fn new(transport: T, generator: G, config: EmitterConfig) -> Self {
        Emitter {
            transport,
            generator,
            config,
            cycles_sent: 0,
            total_events: 0,
            total_bytes: 0,
        }
    }
}
