use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
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

impl TryFrom<&str> for Protocol {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "tcp" => Ok(Protocol::Tcp),
            "udp" => Ok(Protocol::Udp),
            _ => Err(anyhow::anyhow!("Invalid protocol {value}")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Syslog3164,
    Syslog5424,
    NdJson,
}

impl TryFrom<&str> for MessageType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "syslog3164" => Ok(Self::Syslog3164),
            "syslog5424" => Ok(Self::Syslog5424),
            "ndjson" => Ok(Self::NdJson),
            _ => Err(anyhow::anyhow!("Unknown message type '{value}'")),
        }
    }
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
