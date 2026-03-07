mod certs;
mod http;
mod stats_svc;
mod tcp;
mod udp;

#[cfg(test)]
mod integration_tests;

use std::{
    ops::Deref,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use certs::get_cert;
use http::HttpAbsorber;
use log::warn;
use stats_svc::StatsSvc;
use tcp::TcpAbsorber;
use tokio::io::{AsyncRead, ReadBuf};
use udp::UdpAbsorber;

use crate::config::{
    absorber::{AbsorberConfig, HttpAuth},
    ListenAddress, MessageType, Protocol,
};

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
    token: Option<String>,
    mtls: bool,
}

impl From<&AbsorberConfig> for Vec<ConnOptions> {
    fn from(config: &AbsorberConfig) -> Self {
        let http_version = match config.http2 {
            true => hyper::Version::HTTP_2,
            false => hyper::Version::HTTP_11,
        };
        let token = match config.auth {
            HttpAuth::None => None,
            _ => Some(config.token.clone()),
        };
        config
            .listen_addresses
            .iter()
            .map(|addr| {
                let addr_tls = match addr.protocol {
                    Protocol::Https | Protocol::Tcps => true,
                    _ => false,
                };
                let use_tls = config.https || config.http2 || addr_tls;

                let cert_type = if use_tls {
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

                ConnOptions {
                    http_version,
                    addr: addr.clone(),
                    cert_type,
                    protocol: addr.protocol.clone(),
                    token: token.clone(),
                    mtls: config.mtls,
                }
            })
            .collect()
    }
}

impl AbsorberInner {
    async fn build(opts: ConnOptions, message_type: MessageType) -> Self {
        match opts.protocol {
            Protocol::Tcp | Protocol::Tcps => TcpAbsorber::build(opts, message_type).await.into(),
            Protocol::Udp => UdpAbsorber::build(opts, message_type).await.into(),
            Protocol::Http | Protocol::Https => HttpAbsorber::build(opts, message_type).await.into(),
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
    // Ignore whitespace-only messages (e.g., trailing newlines)
    if message.iter().all(|b| b.is_ascii_whitespace()) {
        return;
    }

    // Validate and process the message
    if validate_message(message, message_type) {
        let message_len = message.len();
        stats.increment(1, 0, message_len).await;
    } else {
        warn!(
            "Failed to validate message of type {:?}: {:?}",
            message_type,
            String::from_utf8_lossy(message)
        );
    }
}

pub(super) fn extract_message(buf: &mut Vec<u8>, fin: bool) -> Option<Vec<u8>> {
    // Implement message extraction logic based on the message type
    // For now, assume newline-delimited messages
    if buf.len() == 1 && buf[0] == b'\n' {
        return None;
    }
    if let Some(pos) = buf.iter().position(|&x| x == b'\n') {
        let message = buf.drain(..=pos).collect();
        Some(message)
    } else if fin && !buf.is_empty() {
        let message = std::mem::take(buf);
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
        MessageType::Syslog5424Octet => validate_syslog5424(message),
        MessageType::NdJson => validate_ndjson(message),
    }
}

pub(super) struct CountingReader<R> {
    inner: R,
    stats: StatsSvc,
}

impl<R> CountingReader<R> {
    pub(super) fn new(inner: R, stats: StatsSvc) -> Self {
        Self { inner, stats }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for CountingReader<R> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let res = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &res {
            let after = buf.filled().len();
            let n = after - before;
            if n > 0 {
                self.stats.try_increment(0, n, 0);
            }
        }
        res
    }
}
