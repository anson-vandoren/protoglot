use log::warn;
use serde::{Deserialize, Serialize};

use super::{cli::Commands, config::FullConfig, ListenAddress, MessageType};

#[derive(Serialize, Clone, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AbsorberConfig {
    pub listen_addresses: Vec<ListenAddress>,
    pub update_interval: u64,
    pub message_type: MessageType,
}

#[derive(Serialize, Clone, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PartialAbsorberConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_addresses: Option<Vec<ListenAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,
}

impl Default for AbsorberConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec![],
            update_interval: 5000,
            message_type: MessageType::Syslog3164,
        }
    }
}

impl AbsorberConfig {
    pub fn merge(mut self, other: PartialAbsorberConfig) -> Self {
        if let Some(other) = other.listen_addresses {
            self.listen_addresses = other;
        }
        if let Some(other) = other.update_interval {
            self.update_interval = other;
        }
        if let Some(other) = other.message_type {
            self.message_type = other
        }
        self
    }

    pub fn merge_from(self, other: Option<FullConfig>) -> Self {
        if let Some(full_config) = other {
            if let Some(absorber_config) = full_config.absorber {
                return self.merge(absorber_config);
            }
        }
        self
    }
}

impl From<Option<Commands>> for PartialAbsorberConfig {
    fn from(value: Option<Commands>) -> Self {
        if let Some(Commands::Absorber {
            update_interval,
            listen_addresses,
            message_type,
        }) = value
        {
            let listen_addresses = listen_addresses
                .iter()
                .flatten()
                .map(|addr| ListenAddress::try_from(addr.as_str()))
                .collect::<Result<Vec<_>, _>>()
                .ok();
            return Self {
                update_interval,
                listen_addresses,
                message_type,
            };
        }
        warn!("Tried to get a PartialAbsorberConfig from non-Absorber command: {:?}", value);
        PartialAbsorberConfig {
            update_interval: None,
            listen_addresses: None,
            message_type: None,
        }
    }
}

impl From<AbsorberConfig> for PartialAbsorberConfig {
    fn from(value: AbsorberConfig) -> Self {
        PartialAbsorberConfig {
            update_interval: Some(value.update_interval),
            listen_addresses: Some(value.listen_addresses),
            message_type: Some(value.message_type),
        }
    }
}
