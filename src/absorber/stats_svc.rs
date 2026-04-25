use std::time::Duration;

use human_bytes::human_bytes;
use log::info;
#[cfg(test)]
use tokio::sync::oneshot;
use tokio::{sync::mpsc, time::Instant};

use super::human_events;

pub(crate) struct AbsorberStats {
    pub(crate) total_events: usize,
    pub(crate) intv_events: usize,
    pub(crate) total_raw_bytes: usize,
    pub(crate) intv_raw_bytes: usize,
    pub(crate) total_decomp_bytes: usize,
    pub(crate) intv_decomp_bytes: usize,
    pub(crate) start_time: Instant,
}

#[derive(Clone)]
pub(crate) struct StatsSvc {
    tx: mpsc::Sender<StatsMessage>,
}

impl StatsSvc {
    pub fn run(update_intv_millis: u64) -> Self {
        let mut stats = AbsorberStats::new();
        let (tx, mut rx) = mpsc::channel(100);

        let mut interval = tokio::time::interval(Duration::from_millis(update_intv_millis));
        let task = async move {
            loop {
                tokio::select! {
                    update = rx.recv() => {
                        match update {
                            None => {
                                info!("StatsIncrement channel is closed.");
                                break;
                            }
                            Some(update) => {
                                match update {
                                    StatsMessage::Reset => {
                                        stats.total_events = 0;
                                        stats.intv_events = 0;
                                        stats.total_raw_bytes = 0;
                                        stats.intv_raw_bytes = 0;
                                        stats.total_decomp_bytes = 0;
                                        stats.intv_decomp_bytes = 0;
                                        stats.start_time = Instant::now();
                                    },
                                    StatsMessage::Increment { events, raw_bytes, decomp_bytes } => {
                                        stats.total_events += events;
                                        stats.intv_events += events;
                                        stats.total_raw_bytes += raw_bytes;
                                        stats.intv_raw_bytes += raw_bytes;
                                        stats.total_decomp_bytes += decomp_bytes;
                                        stats.intv_decomp_bytes += decomp_bytes;
                                    },
                                    #[cfg(test)]
                                    StatsMessage::GetStats(tx) => {
                                        let _ = tx.send((stats.total_events, stats.total_raw_bytes, stats.total_decomp_bytes));
                                    }
                                }
                            }
                        }

                    }

                    _ = interval.tick() => {
                        let elapsed = stats.start_time.elapsed().as_secs_f64();
                        if stats.intv_events > 0 {
                            let events_per_sec = stats.intv_events as f64 / elapsed;
                            let fmt_eps = human_events(events_per_sec);
                            let raw_bytes_per_sec = stats.intv_raw_bytes as f64 / elapsed;
                            let fmt_raw_bps = human_bytes(raw_bytes_per_sec);
                            let decomp_bytes_per_sec = stats.intv_decomp_bytes as f64 / elapsed;
                            let fmt_decomp_bps = human_bytes(decomp_bytes_per_sec);

                            if stats.intv_raw_bytes != stats.intv_decomp_bytes {
                                let ratio = stats.intv_decomp_bytes as f64 / stats.intv_raw_bytes as f64;
                                let fmt_total_raw = human_bytes(stats.total_raw_bytes as f64);
                                let fmt_total_decomp = human_bytes(stats.total_decomp_bytes as f64);
                                println!(
                                    "Total events: {}, Total raw: {}, Total decomp: {} | {} EPS, {}/s raw, {}/s decomp ({:.1}x ratio)",
                                    stats.total_events, fmt_total_raw, fmt_total_decomp, fmt_eps, fmt_raw_bps, fmt_decomp_bps, ratio
                                );
                            } else {
                                let fmt_total_bytes = human_bytes(stats.total_raw_bytes as f64);
                                println!(
                                    "Total events: {}, Total bytes: {} | {} EPS, {}/s average",
                                    stats.total_events, fmt_total_bytes, fmt_eps, fmt_raw_bps
                                );
                            }
                        }
                        // reset interval start time
                        stats.start_time = Instant::now();
                        stats.intv_raw_bytes = 0;
                        stats.intv_decomp_bytes = 0;
                        stats.intv_events = 0;
                    }
                }
            }
        };
        tokio::spawn(task);
        Self { tx }
    }

    pub async fn increment(&self, events: usize, raw_bytes: usize, decomp_bytes: usize) {
        self.tx
            .send(StatsMessage::Increment {
                events,
                raw_bytes,
                decomp_bytes,
            })
            .await
            .unwrap();
    }

    pub fn try_increment(&self, events: usize, raw_bytes: usize, decomp_bytes: usize) {
        let _ = self.tx.try_send(StatsMessage::Increment {
            events,
            raw_bytes,
            decomp_bytes,
        });
    }

    pub async fn reset(&self) {
        self.tx.send(StatsMessage::Reset).await.unwrap();
    }

    #[cfg(test)]
    pub async fn get_stats(&self) -> (usize, usize, usize) {
        let (tx, rx) = oneshot::channel();
        self.tx.send(StatsMessage::GetStats(tx)).await.unwrap();
        rx.await.unwrap()
    }
}

#[derive(Debug)]
enum StatsMessage {
    Increment {
        events: usize,
        raw_bytes: usize,
        decomp_bytes: usize,
    },
    Reset,
    #[cfg(test)]
    GetStats(oneshot::Sender<(usize, usize, usize)>),
}

impl AbsorberStats {
    pub fn new() -> Self {
        Self {
            total_events: 0,
            intv_events: 0,
            total_raw_bytes: 0,
            intv_raw_bytes: 0,
            total_decomp_bytes: 0,
            intv_decomp_bytes: 0,
            start_time: Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absorber_stats_new() {
        let stats = AbsorberStats::new();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.total_raw_bytes, 0);
        assert_eq!(stats.total_decomp_bytes, 0);
    }

    #[test]
    fn test_absorber_stats_increment() {
        let mut stats = AbsorberStats::new();

        // Uncompressed increment
        stats.total_events += 1;
        stats.total_raw_bytes += 100;
        stats.total_decomp_bytes += 100;

        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.total_raw_bytes, 100);
        assert_eq!(stats.total_decomp_bytes, 100);

        // Compressed increment
        stats.total_events += 1;
        stats.total_raw_bytes += 50;
        stats.total_decomp_bytes += 200;

        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.total_raw_bytes, 150);
        assert_eq!(stats.total_decomp_bytes, 300);
    }
}
