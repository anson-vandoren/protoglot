use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use eyre::Result;

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Protocol {
    Tcp,
    Udp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        };
        s.fmt(f)
    }
}

impl Protocol {
    pub fn from_str(input: &str) -> Result<Protocol, &'static str> {
        match input.to_lowercase().as_str() {
            "tcp" => Ok(Protocol::Tcp),
            "udp" => Ok(Protocol::Udp),
            _ => Err("Invalid protocol"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum MessageType {
    Syslog3164,
    Syslog5424,
    NdJson,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MessageType::Syslog3164 => "syslog3164",
            MessageType::Syslog5424 => "syslog5424",
            MessageType::NdJson => "ndjson",
        };
        s.fmt(f)
    }
}
