use std::time::Duration;

use human_bytes::human_bytes;
use log::info;
use tokio::{sync::mpsc, time::Instant};

use super::human_events;

pub(super) struct AbsorberStats {
    pub(super) total_events: usize,
    pub(super) intv_events: usize,
    pub(super) intv_bytes: usize,
    pub(super) total_bytes: usize,
    pub(super) start_time: Instant,
}

#[derive(Clone)]
pub(super) struct StatsSvc {
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
                                        stats.total_bytes = 0;
                                        stats.intv_bytes = 0;
                                        stats.start_time = Instant::now();
                                    },
                                    StatsMessage::Increment { events, bytes } => {
                                        stats.total_events += events;
                                        stats.intv_events += events;
                                        stats.total_bytes += bytes;
                                        stats.intv_bytes += bytes;
                                    },
                                }
                            }
                        }

                    }

                    _ = interval.tick() => {
                        let elapsed = stats.start_time.elapsed().as_secs_f64();
                        if stats.intv_events > 0 {
                            let events_per_sec = stats.intv_events as f64 / elapsed;
                            let fmt_eps = human_events(events_per_sec);
                            let bytes_per_sec = stats.intv_bytes as f64 / elapsed;
                            let fmt_bps = human_bytes(bytes_per_sec);
                            println!(
                                "Total events: {}, {} EPS average, {}/s average",
                                stats.total_events, fmt_eps, fmt_bps
                            );
                        }
                        // reset interval start time
                        stats.start_time = Instant::now();
                        stats.intv_bytes = 0;
                        stats.intv_events = 0;
                    }
                }
            }
        };
        tokio::spawn(task);
        Self { tx }
    }

    pub async fn increment(&self, events: usize, bytes: usize) {
        self.tx.send(StatsMessage::Increment { events, bytes }).await.unwrap();
    }

    pub async fn reset(&self) {
        self.tx.send(StatsMessage::Reset).await.unwrap();
    }
}

#[derive(Debug)]
enum StatsMessage {
    Increment { events: usize, bytes: usize },
    Reset,
}

impl AbsorberStats {
    pub fn new() -> Self {
        Self {
            total_events: 0,
            intv_events: 0,
            total_bytes: 0,
            intv_bytes: 0,
            start_time: Instant::now(),
        }
    }
}
