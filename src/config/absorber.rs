use clap::ValueEnum;
use log::{info, warn};
use serde::{Deserialize, Serialize};

use super::{cli::Commands, FullConfig, ListenAddress, MessageType};

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "camelCase")]
pub enum HttpAuth {
    /// No authentication required
    #[default]
    None,
    /// Basic HTTP authentication (base64 encoded username:password)
    Basic,
    /// Bearer token authentication (e.g., JWT)
    Token,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsorberConfig {
    pub listen_addresses: Vec<ListenAddress>,
    pub update_interval: u64,
    pub message_type: MessageType,
    /// Note that HTTP2 implies HTTPS
    pub http2: bool,
    pub https: bool,
    pub self_signed: bool,
    pub private_ca: bool,
    pub auth: HttpAuth,
    pub token: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialAbsorberConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_addresses: Option<Vec<ListenAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Note that HTTP2 implies HTTPS
    pub http2: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub https: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_signed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_ca: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<HttpAuth>,
    pub token: String,
}

impl Default for AbsorberConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec![],
            update_interval: 5000,
            message_type: MessageType::Syslog3164,
            http2: false,
            https: false,
            self_signed: false,
            private_ca: false,
            auth: HttpAuth::None,
            token: String::new(),
        }
    }
}

impl AbsorberConfig {
    pub fn merge(mut self, other: PartialAbsorberConfig) -> Self {
        let PartialAbsorberConfig {
            listen_addresses,
            update_interval,
            message_type,
            http2,
            https,
            self_signed,
            private_ca,
            auth,
            token,
        } = other;

        if let Some(listen_addresses) = listen_addresses {
            self.listen_addresses = listen_addresses;
        }
        if let Some(update_interval) = update_interval {
            self.update_interval = update_interval;
        }
        if let Some(message_type) = message_type {
            self.message_type = message_type;
        }
        if let Some(http2) = http2 {
            self.http2 = http2;
        }
        if let Some(https) = https {
            self.https = https;
        }
        if let Some(self_signed) = self_signed {
            self.self_signed = self_signed;
        }
        if let Some(private_ca) = private_ca {
            self.private_ca = private_ca;
        }
        if let Some(auth) = auth {
            self.auth = auth;
        }

        self.token = token;

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

const USERNAME: &str = "AzureDiamond";
const PASSWORD: &str = "hunter2";
const TOKEN: &str = "this_is_a_fucking_token";

fn token_for(auth: &HttpAuth) -> String {
    use base64::prelude::*;
    match auth {
        HttpAuth::None => String::new(),
        HttpAuth::Basic => {
            info!("Using Basic HTTP credentials:\n  Username: {}\n  Password: {}", USERNAME, PASSWORD);
            let credentials = format!("{}:{}", USERNAME, PASSWORD);
            let encoded = BASE64_STANDARD.encode(credentials);
            format!("Basic {}", encoded)
        }
        HttpAuth::Token => {
            info!("Using Token: {}", TOKEN);
            format!("Bearer {}", TOKEN)
        }
    }
}

impl From<Option<Commands>> for PartialAbsorberConfig {
    fn from(value: Option<Commands>) -> Self {
        if let Some(Commands::Absorber {
            update_interval,
            listen_addresses,
            message_type,
            http2,
            https,
            self_signed,
            private_ca,
            auth,
        }) = value
        {
            let listen_addresses = listen_addresses
                .iter()
                .flatten()
                .map(|addr| ListenAddress::try_from(addr.as_str()))
                .collect::<Result<Vec<_>, _>>()
                .ok();
            let auth_type = auth.clone().unwrap_or_default();

            return Self {
                update_interval,
                listen_addresses,
                message_type,
                http2,
                https,
                self_signed,
                private_ca,
                auth,
                token: token_for(&auth_type),
            };
        }
        warn!("Tried to get a PartialAbsorberConfig from non-Absorber command: {:?}", value);
        PartialAbsorberConfig::default()
    }
}

impl From<AbsorberConfig> for PartialAbsorberConfig {
    fn from(value: AbsorberConfig) -> Self {
        PartialAbsorberConfig {
            update_interval: Some(value.update_interval),
            listen_addresses: Some(value.listen_addresses),
            message_type: Some(value.message_type),
            http2: Some(value.http2),
            https: Some(value.https),
            self_signed: Some(value.self_signed),
            private_ca: Some(value.private_ca),
            auth: Some(value.auth),
            token: value.token,
        }
    }
}
