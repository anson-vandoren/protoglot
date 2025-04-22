use std::sync::Arc;

use human_bytes::human_bytes;
use tokio::sync::Mutex;

use super::{tcp::TcpAbsorber, udp::UdpAbsorber};
use crate::config::{absorber::AbsorberConfig, MessageType, Protocol};

#[derive(Clone)]
pub struct Absorber {
    config: Arc<AbsorberConfig>,
    stats: Arc<Mutex<AbsorberStats>>,
}

pub enum AbsorberInner {
    Tcp(TcpAbsorber),
    Udp(UdpAbsorber),
}

impl AbsorberInner {
    async fn build(protocol: Protocol, address: &str, port: u16, message_type: MessageType) -> Self {
        match protocol {
            Protocol::Tcp => TcpAbsorber::build(address, port, message_type).await.into(),
            Protocol::Udp => UdpAbsorber::build(address, port, message_type).await.into(),
        }
    }

    async fn run(self, stats: Arc<Mutex<AbsorberStats>>) -> anyhow::Result<()> {
        match self {
            Self::Tcp(absorber) => absorber.run(stats).await,
            Self::Udp(absorber) => absorber.run(stats).await,
        }
    }
}

pub(super) struct AbsorberStats {
    pub(super) total_events: u64,
    pub(super) intv_events: u64,
    pub(super) intv_bytes: u64,
    pub(super) total_bytes: u64,
    pub(super) start_time: std::time::Instant,
}

impl Absorber {
    pub fn new(config: AbsorberConfig) -> Self {
        Self {
            config: Arc::new(config),
            stats: Arc::new(Mutex::new(AbsorberStats {
                total_events: 0,
                total_bytes: 0,
                intv_events: 0,
                intv_bytes: 0,
                start_time: std::time::Instant::now(),
            })),
        }
    }

    pub async fn run(&self) -> tokio::io::Result<()> {
        let mut handles = vec![];

        for address in &self.config.listen_addresses {
            let address = address.clone();
            let absorber = AbsorberInner::build(address.protocol, &address.host, address.port, self.config.message_type.clone()).await;
            let stats = Arc::clone(&self.stats);
            let handle = tokio::spawn(async move { absorber.run(stats).await });
            handles.push(handle);
        }

        let update_interval = self.config.update_interval;
        let stats_handle = tokio::spawn(update_stats(Arc::clone(&self.stats), update_interval));
        handles.push(stats_handle);

        let input_handle = tokio::spawn(handle_user_input(Arc::clone(&self.stats)));
        handles.push(input_handle);

        for handle in handles {
            handle.await?.unwrap();
        }

        Ok(())
    }
}

async fn update_stats(stats: Arc<Mutex<AbsorberStats>>, update_interval: u64) -> anyhow::Result<()> {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(update_interval)).await;
        let mut stats = stats.lock().await;
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
        stats.start_time = std::time::Instant::now();
        stats.intv_bytes = 0;
        stats.intv_events = 0;
    }
}

async fn handle_user_input(stats: Arc<Mutex<AbsorberStats>>) -> anyhow::Result<()> {
    let mut input = String::new();
    loop {
        if let Ok(_) = std::io::stdin().read_line(&mut input) {
            match input.trim() {
                "rs" => {
                    let mut stats = stats.lock().await;
                    *stats = AbsorberStats {
                        total_events: 0,
                        total_bytes: 0,
                        intv_events: 0,
                        intv_bytes: 0,
                        start_time: std::time::Instant::now(),
                    };
                    println!("Stats reset")
                }
                "q" => {
                    println!("Exiting...");
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        input.clear();
    }
}

pub(super) fn extract_message(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
    // Implement message extraction logic based on the message type
    // For now, assume newline-delimited messages
    if let Some(pos) = buf.iter().position(|&x| x == b'\n') {
        let message = buf.drain(..=pos).collect();
        Some(message)
    } else {
        None
    }
}

fn human_events(events: f64) -> String {
    if events < 1_000.0 {
        events.to_string()
    } else if events < 1_000_000.0 {
        format!("{:.1}k", events / 1_000.0)
    } else if events < 1_000_000_000.0 {
        format!("{:.1}M", events / 1_000_000.0)
    } else {
        format!("{:.1}B", events / 1_000_000_000.0)
    }
}

fn validate_syslog3164(message: &[u8]) -> bool {
    // TODO: Implement full Syslog 3164 message validation
    message.starts_with(b"<") && message.contains(&b'>')
}

fn validate_syslog5424(message: &[u8]) -> bool {
    // TODO: Implement full Syslog 5424 message validation
    let s = String::from_utf8_lossy(message);
    s.starts_with("<") && s.contains(">1 ") && s.split_whitespace().count() >= 7
}

fn validate_ndjson(message: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(message).is_ok()
}

pub(super) fn validate_message(message: &[u8], typ: &MessageType) -> bool {
    match typ {
        MessageType::Syslog3164 => validate_syslog3164(message),
        MessageType::Syslog5424 => validate_syslog5424(message),
        MessageType::NdJson => validate_ndjson(message),
    }
}
