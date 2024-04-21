use serde::{Deserialize, Serialize};
use config;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub senders: Vec<SenderSettings>,
    pub message_file: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SenderSettings {
    pub host: String,
    pub port: u16,
    pub rate: u64,
    #[serde(default = "default_tls")]
    pub tls: bool,
    pub message_type: String,
}

fn default_tls() -> bool {
    true
}

// TODO: file watching for config changes?
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