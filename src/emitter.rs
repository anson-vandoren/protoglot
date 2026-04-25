use std::time::{Duration, Instant};

use human_bytes::human_bytes;
use log::info;

use crate::{generators::EventGenerator, transports::Transport};

pub struct EmitterConfig {
    pub rate: u64,             // events per second
    pub num_cycles: u64,       // number of cycles to send, 0 means run forever
    pub events_per_cycle: u64, // number of events per cycles
    pub cycle_delay: u64,      // delay between cycles in milliseconds
    pub batch_size: u64,       // number of events per transport send
}

pub struct Emitter<T: Transport, G: EventGenerator> {
    pub transport: T,
    pub generator: G,
    pub config: EmitterConfig,
    cycles_sent: u64,
    pub total_events: u64,
    pub total_bytes: u64,
}

impl<T, G> Emitter<T, G>
where
    T: Transport + Send + 'static + std::fmt::Display,
    G: EventGenerator + Send + 'static,
{
    pub async fn run(&mut self) -> tokio::io::Result<()> {
        let start_time = Instant::now();
        let mut next_tick = Instant::now();
        let interval_nanos = 1_000_000_000 / self.config.rate;

        let mut buf = Vec::with_capacity(1024);
        let sleep_batch_size = if self.config.rate >= 100_000 {
            128
        } else if self.config.rate >= 10_000 {
            64
        } else if self.config.rate >= 1_000 {
            16
        } else if self.config.rate >= 100 {
            4
        } else {
            1
        };

        let batch_size = self.config.batch_size.max(1);

        while self.config.num_cycles == 0 || self.cycles_sent < self.config.num_cycles {
            let mut events_sent_this_cycle = 0;
            while events_sent_this_cycle < self.config.events_per_cycle {
                buf.clear();
                let events_in_batch = batch_size.min(self.config.events_per_cycle - events_sent_this_cycle);
                for _ in 0..events_in_batch {
                    self.generator.generate_into(&mut buf);
                }
                self.total_bytes += buf.len() as u64;
                self.total_events += events_in_batch;
                self.transport.send(&buf).await?;

                events_sent_this_cycle += events_in_batch;
                next_tick += Duration::from_nanos(interval_nanos.saturating_mul(events_in_batch));
                if events_sent_this_cycle % sleep_batch_size == 0 {
                    tokio::time::sleep_until(next_tick.into()).await;
                }
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

#[cfg(test)]
mod tests {
    use std::{
        fmt,
        sync::{Arc, Mutex},
    };

    use super::*;

    struct FakeGenerator {
        next: u64,
    }

    impl EventGenerator for FakeGenerator {
        fn generate_into(&mut self, buf: &mut Vec<u8>) {
            buf.extend_from_slice(format!("event-{}\n", self.next).as_bytes());
            self.next += 1;
        }
    }

    struct FakeTransport {
        sends: Arc<Mutex<Vec<Vec<u8>>>>,
    }

    impl Transport for FakeTransport {
        async fn send(&mut self, data: &[u8]) -> tokio::io::Result<()> {
            self.sends.lock().unwrap().push(data.to_vec());
            Ok(())
        }
    }

    impl fmt::Display for FakeTransport {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "fake")
        }
    }

    #[tokio::test]
    async fn batches_multiple_events_per_transport_send() {
        let sends = Arc::new(Mutex::new(Vec::new()));
        let transport = FakeTransport { sends: sends.clone() };
        let generator = FakeGenerator { next: 0 };
        let config = EmitterConfig {
            rate: 1_000_000,
            num_cycles: 1,
            events_per_cycle: 5,
            cycle_delay: 0,
            batch_size: 2,
        };
        let mut emitter = Emitter::new(transport, generator, config);

        emitter.run().await.unwrap();

        let sends = sends.lock().unwrap();
        assert_eq!(sends.len(), 3);
        assert_eq!(sends[0], b"event-0\nevent-1\n");
        assert_eq!(sends[1], b"event-2\nevent-3\n");
        assert_eq!(sends[2], b"event-4\n");
        assert_eq!(emitter.total_events, 5);
    }
}
