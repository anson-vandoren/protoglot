use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Tcps,
    Udp,
    Http,
    Https,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Protocol::Tcp => "tcp",
            Protocol::Tcps => "tcps",
            Protocol::Udp => "udp",
            Protocol::Http => "http",
            Protocol::Https => "https",
        };
        s.fmt(f)
    }
}

impl TryFrom<&str> for Protocol {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "tcp" => Ok(Protocol::Tcp),
            "tcps" => Ok(Protocol::Tcps),
            "udp" => Ok(Protocol::Udp),
            "http" => Ok(Protocol::Http),
            "https" => Ok(Protocol::Https),
            _ => Err(anyhow::anyhow!("Invalid protocol {value}")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    SplunkHec,
    TcpSyslog3164,
    TcpSyslog5424,
    UdpSyslog3164,
    HttpNdjson,
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Profile::SplunkHec => "splunk-hec",
            Profile::TcpSyslog3164 => "tcp-syslog3164",
            Profile::TcpSyslog5424 => "tcp-syslog5424",
            Profile::UdpSyslog3164 => "udp-syslog3164",
            Profile::HttpNdjson => "http-ndjson",
        };
        s.fmt(f)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Syslog3164,
    Syslog5424,
    Syslog5424Octet,
    NdJson,
    #[serde(rename = "splunk-hec", alias = "splunkhec", alias = "splunkHec")]
    SplunkHec,
}

impl TryFrom<&str> for MessageType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "syslog3164" => Ok(Self::Syslog3164),
            "syslog5424" => Ok(Self::Syslog5424),
            "syslog5424-octet" => Ok(Self::Syslog5424Octet),
            "ndjson" => Ok(Self::NdJson),
            "splunk-hec" | "splunkhec" | "splunkHec" => Ok(Self::SplunkHec),
            _ => Err(anyhow::anyhow!("Unknown message type '{value}'")),
        }
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MessageType::Syslog3164 => "syslog3164",
            MessageType::Syslog5424 => "syslog5424",
            MessageType::Syslog5424Octet => "syslog5424-octet",
            MessageType::NdJson => "ndjson",
            MessageType::SplunkHec => "splunk-hec",
        };
        s.fmt(f)
    }
}
