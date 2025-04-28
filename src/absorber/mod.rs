mod certs;
mod http;
mod stats_svc;
mod tcp;
mod udp;

use std::{ops::Deref, sync::Arc};

use certs::get_cert;
use http::HttpAbsorber;
use log::warn;
use stats_svc::StatsSvc;
use tcp::TcpAbsorber;
use udp::UdpAbsorber;

use crate::config::{absorber::AbsorberConfig, ListenAddress, MessageType, Protocol};

#[derive(Clone)]
pub struct Absorber {
    config: Arc<AbsorberConfig>,
}

pub enum AbsorberInner {
    Tcp(TcpAbsorber),
    Udp(UdpAbsorber),
    Http(HttpAbsorber),
}

#[derive(Clone, Debug)]
enum CertType {
    None,
    SelfSigned,
    PrivateCA,
    PublicCA,
}

#[derive(Debug)]
pub struct ConnOptions {
    http_version: hyper::Version,
    addr: ListenAddress,
    cert_type: CertType,
    protocol: Protocol,
    token: String,
}

impl From<&AbsorberConfig> for Vec<ConnOptions> {
    fn from(config: &AbsorberConfig) -> Self {
        let http_version = match config.http2 {
            true => hyper::Version::HTTP_2,
            false => hyper::Version::HTTP_11,
        };
        let cert_type = if config.https || config.http2 {
            if config.self_signed {
                CertType::SelfSigned
            } else if config.private_ca {
                CertType::PrivateCA
            } else {
                CertType::PublicCA
            }
        } else {
            CertType::None
        };
        config
            .listen_addresses
            .iter()
            .map(|addr| ConnOptions {
                http_version,
                addr: addr.clone(),
                cert_type: cert_type.clone(),
                protocol: addr.protocol.clone(),
                token: config.token.clone(),
            })
            .collect()
    }
}

impl AbsorberInner {
    async fn build(opts: ConnOptions, message_type: MessageType) -> Self {
        match opts.protocol {
            Protocol::Tcp => TcpAbsorber::build(opts, message_type).await.into(),
            Protocol::Udp => UdpAbsorber::build(opts, message_type).await.into(),
            Protocol::Http => HttpAbsorber::build(opts, message_type).await.into(),
        }
    }

    async fn run(self, stats_svc: StatsSvc) -> anyhow::Result<()> {
        match self {
            Self::Tcp(absorber) => absorber.run(stats_svc).await,
            Self::Udp(absorber) => absorber.run(stats_svc).await,
            Self::Http(absorber) => absorber.run(stats_svc).await,
        }
    }
}

impl Absorber {
    pub fn new(config: AbsorberConfig) -> Self {
        Self { config: Arc::new(config) }
    }

    pub async fn run(&self) -> tokio::io::Result<()> {
        let mut handles = vec![];
        let update_interval = self.config.update_interval;
        let stats_svc = StatsSvc::run(update_interval);

        let conn_opts: Vec<ConnOptions> = self.config.deref().into();
        for conn_opt in conn_opts {
            let stats = stats_svc.clone();
            let absorber = AbsorberInner::build(conn_opt, self.config.message_type.clone()).await;
            let handle = tokio::spawn(async move { absorber.run(stats).await });
            handles.push(handle);
        }

        let input_handle = tokio::spawn(handle_user_input(stats_svc));
        handles.push(input_handle);

        for handle in handles {
            handle.await?.unwrap();
        }

        Ok(())
    }
}

async fn handle_user_input(stats: StatsSvc) -> anyhow::Result<()> {
    let mut input = String::new();
    loop {
        if std::io::stdin().read_line(&mut input).is_ok() {
            match input.trim() {
                "rs" => {
                    stats.reset().await;
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

async fn process_message(message: &[u8], stats: &StatsSvc, message_type: &MessageType) {
    // Validate and process the message
    if validate_message(message, message_type) {
        let message_len = message.len();
        stats.increment(1, message_len).await;
    } else {
        warn!("Failed to validate message");
    }
}

pub(super) fn extract_message(buf: &mut Vec<u8>, fin: bool) -> Option<Vec<u8>> {
    // Implement message extraction logic based on the message type
    // For now, assume newline-delimited messages
    if let Some(pos) = buf.iter().position(|&x| x == b'\n') {
        let message = buf.drain(..=pos).collect();
        Some(message)
    } else if fin && !buf.is_empty() {
        let message = buf.drain(..).collect();
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
