use clap::Parser;
use eyre::{Result, Report};
use directories::ProjectDirs;
use figment::providers::{Env, Format, Serialized};
use log::{debug, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::cli::CliArgs;
use super::{MessageType, Protocol};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct EmitterSettings {
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

impl EmitterSettings {
    fn new() -> Result<Self> {
        let config_str = include_str!("../../config/default.json5");
        json5::from_str(config_str).map_err(Report::new)
    }
}
struct Json5;
impl figment::providers::Format for Json5 {
    type Error = json5::Error;

    const NAME: &'static str = "JSON5";

    fn from_str<'de, T: DeserializeOwned>(s: &'de str) -> Result<T, Self::Error> {
        json5::from_str(s)
    }
}

impl EmitterSettings {
    pub fn load() -> Result<Self> {
        // The settings are loaded in the following order:

        // 1. load defaults from file which should be baked into the binary
        let mut figment = figment::Figment::from(Serialized::defaults(EmitterSettings::new()?));

        // 2. load values from config directory, overlaying on defaults
        if let Some(proj_dirs) = ProjectDirs::from("com", "ansonvandoren", "bablfsh") {
            let config_file_path = proj_dirs.config_dir().join("config.json5");
            if config_file_path.exists() {
                figment = figment.merge(Json5::file(&config_file_path));
                info!(config_file_path:?; "Using config file");
            } else {
                info!(config_file_path:?; "No config file found, using defaults");
            }
        } else {
            info!("No config directory found, using defaults");
        }

        let args = CliArgs::parse();
        debug!(args:serde; "Parsed CLI args");

        // 3. if a file arg is provided, overlay that on defaults+configDir
        if let Some(ref file_path) = args.file {
            if file_path.exists() {
                figment = figment.merge(Json5::file(file_path));
                info!(file = file_path.to_str(); "Using specified file");
            } else {
                return Err(Report::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found: {}", file_path.display()),
                )));
            }
        }

        // 4. if env vars are provided, overlay those on defaults+configDir+file
        figment = figment.merge(Env::prefixed("BABL_").lowercase(false));

        // 5. if CLI args are provided, overlay those on defaults+configDir+file+env
        figment = figment.merge(Serialized::defaults(args));

        println!("{:?}", figment);

        figment.extract().map_err(Report::new)
    }
}
