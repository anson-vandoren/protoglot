use serde::{Deserialize, Serialize};

use super::{cli::CliArgs, FullConfig, MessageType, Protocol};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmitterConfig {
    pub host: String,
    pub port: u16,
    pub rate: u64,
    pub tls: bool,
    pub protocol: Protocol,
    pub message_type: MessageType,
    pub num_emitters: u64,
    pub events_per_cycle: u64,
    pub num_cycles: u64,
    pub cycle_delay: u64,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            host: "localhost".into(),
            port: 9514,
            tls: false,
            protocol: Protocol::Tcp,
            rate: 1000,
            message_type: MessageType::Syslog3164,
            num_emitters: 1,
            events_per_cycle: 10000,
            num_cycles: 1,
            cycle_delay: 10000,
        }
    }
}

impl EmitterConfig {
    pub fn merge(mut self, other: PartialEmitterConfig) -> Self {
        if let Some(other) = other.host {
            self.host = other
        }
        if let Some(other) = other.port {
            self.port = other;
        }
        if let Some(other) = other.rate {
            self.rate = other;
        }
        if let Some(other) = other.tls {
            self.tls = other;
        }
        if let Some(other) = other.protocol {
            self.protocol = other;
        }
        if let Some(other) = other.message_type {
            self.message_type = other;
        }
        if let Some(other) = other.num_emitters {
            self.num_emitters = other;
        }
        if let Some(other) = other.events_per_cycle {
            self.events_per_cycle = other;
        }
        if let Some(other) = other.num_cycles {
            self.num_cycles = other;
        }
        if let Some(other) = other.cycle_delay {
            self.cycle_delay = other;
        }
        self
    }

    pub fn merge_from(self, other: Option<FullConfig>) -> Self {
        if let Some(full_config) = other {
            if let Some(emitter_config) = full_config.emitter {
                return self.merge(emitter_config);
            }
        }
        self
    }
}

impl From<CliArgs> for PartialEmitterConfig {
    fn from(value: CliArgs) -> Self {
        PartialEmitterConfig {
            host: value.host,
            port: value.port,
            rate: value.rate,
            tls: value.tls,
            protocol: value.protocol,
            message_type: value.message_type,
            num_emitters: value.num_emitters,
            events_per_cycle: value.events_per_cycle,
            num_cycles: value.num_cycles,
            cycle_delay: value.cycle_delay,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialEmitterConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Protocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_emitters: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_per_cycle: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_cycles: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_delay: Option<u64>,
}

impl From<EmitterConfig> for PartialEmitterConfig {
    fn from(value: EmitterConfig) -> Self {
        Self {
            host: Some(value.host),
            port: Some(value.port),
            rate: Some(value.rate),
            tls: Some(value.tls),
            protocol: Some(value.protocol),
            message_type: Some(value.message_type),
            num_emitters: Some(value.num_emitters),
            events_per_cycle: Some(value.events_per_cycle),
            num_cycles: Some(value.num_cycles),
            cycle_delay: Some(value.cycle_delay),
        }
    }
}
