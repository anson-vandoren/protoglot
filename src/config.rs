use config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub emitters: Vec<EmitterSettings>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmitterSettings {
    pub host: String,
    pub port: u16,
    pub rate: u64,
    #[serde(default = "default_tls")]
    pub tls: bool,
    pub protocol: String,
    pub message_type: String,
    #[serde(default = "default_num_emitters")]
    pub num_emitters: u64,
    #[serde(default = "default_events_per_batch")]
    pub events_per_batch: u64,
    #[serde(default = "default_num_batches")]
    pub num_batches: u64,
    #[serde(default = "default_batch_delay")]
    pub batch_delay: u64,
}

fn default_tls() -> bool {
    false
}

fn default_batch_delay() -> u64 {
    0
}

fn default_num_batches() -> u64 {
    1
}

fn default_events_per_batch() -> u64 {
    10000
}

fn default_num_emitters() -> u64 {
    1
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("BABL"));
        let settings: Settings = builder.build()?.try_deserialize()?;
        Ok(settings)
    }
}
