use std::path::Path;
use std::str::FromStr;

use clap::Parser;
use directories::ProjectDirs;
use eyre::{Report, Result};
use figment::providers::{Env, Format, Serialized};
use figment::Figment;
use log::{debug, info, trace};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::cli::{CliArgs, Commands};
use super::{MessageType, Protocol};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub mode: AppMode,
    pub emitter: Option<EmitterConfig>,
    pub absorber: Option<AbsorberConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AppMode {
    Emitter,
    Absorber,
}

#[derive(Serialize, Clone, Deserialize, Debug)]
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

impl EmitterConfig {
    fn default() -> Result<Self> {
        let config_str = include_str!("../../config/default.json5");
        let figment = Figment::from(Json5::string(config_str).nested());
        let res = figment.select("emitter").extract().map_err(Report::new);
        info!(res:?; "Default emitter config");
        res
    }
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AbsorberConfig {
    pub listen_addresses: Vec<ListenAddress>,
    pub update_interval: u64,
    pub message_type: MessageType,
}

impl Default for AbsorberConfig {
    fn default() -> Self {
        Self {
            listen_addresses: vec![ListenAddress::default()],
            update_interval: 60,
            message_type: MessageType::NdJson,
        }
    }
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListenAddress {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
}

impl Default for ListenAddress {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 4242,
            protocol: Protocol::Tcp,
        }
    }
}

impl ListenAddress {
    pub fn from_str(s: &str) -> Result<Self> {
        // ex: "tcp://127.0.0.1:4242"
        let parts: Vec<&str> = s.split("://").collect();
        if parts.len() != 2 {
            return Err(Report::msg("Invalid listen address format"));
        }
        let protocol = Protocol::from_str(parts[0]).map_err(|e| Report::msg(e))?;
        let parts: Vec<&str> = parts[1].split(':').collect();
        if parts.len() != 2 {
            return Err(Report::msg("Invalid listen address format"));
        }
        let host = parts[0].to_string();
        let port = parts[1].parse().map_err(Report::new)?;
        Ok(Self {
            host,
            port,
            protocol,
        })
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

impl AppSettings {
    pub fn load() -> Result<Self> {
        let args = CliArgs::parse();
        Self::setup_logging(args.verbose);

        let (mode, config) = match &args.command {
            Some(Commands::Absorber { .. }) => {
                trace!("Starting absorber");
                (AppMode::Absorber, Self::load_absorber_config(args)?)
            }
            None => {
                info!("No command specified, starting emitter");
                (AppMode::Emitter, Self::load_emitter_config(args)?)
            }
        };

        Ok(AppSettings {
            emitter: if matches!(mode, AppMode::Emitter) { Some(config.emitter.unwrap()) } else { None },
            absorber: if matches!(mode, AppMode::Absorber) { Some(config.absorber.unwrap()) } else { None },
            mode,
        })
    }

    fn setup_logging(verbose: u8) {
        let level = match verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            3 => "trace",
            _ => "warn",
        };
        env_logger::builder()
            .filter_level(log::LevelFilter::from_str(level).unwrap())
            .init();
    }

    fn load_config_file(file_path: &Path) -> Result<figment::providers::Data<Json5>, Report> {
        if file_path.exists() {
            info!(file = file_path.to_str(); "Using specified file");
            Ok(Json5::file(file_path))
        } else {
            Err(Report::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", file_path.display()),
            )))
        }
    }

    fn load_emitter_config(args: CliArgs) -> Result<Self> {
        // The settings are loaded in the following order:
        // 1. load defaults from file which will be baked into the binary
        let mut figment = Figment::from(Serialized::defaults(EmitterConfig::default()?));
        figment = figment.select("emitter");

        // 2. load values from config directory, overlaying on defaults
        if let Some(proj_dirs) = ProjectDirs::from("com", "ansonvandoren", "protoglot") {
            let config_file_path = proj_dirs.config_dir().join("config.json5");
            if config_file_path.exists() {
                let mut file_figment = Figment::from(Json5::file(&config_file_path).nested());
                file_figment = file_figment.select("emitter");
                figment = figment.merge(file_figment);
                info!("Using config file found at {:?}", config_file_path);
            } else {
                info!(
                    "No config file found at {:?}, using defaults",
                    config_file_path
                );
            }
        } else {
            info!("No config directory found, using defaults");
        }

        debug!(args:serde; "Parsed CLI args");

        // 3. if a file arg is provided, overlay that on defaults+configDir
        if let Some(ref file_path) = args.file {
            figment = figment.merge(Self::load_config_file(&file_path)?);
        }

        // 4. if env vars are provided, overlay those on defaults+configDir+file
        figment = figment.merge(Env::prefixed("GLOT_").lowercase(false).split("__"));

        // 5. if CLI args are provided, overlay those on defaults+configDir+file+env
        figment = figment.merge(Serialized::defaults(args));

        trace!(figment:?; "Final configuration");
        let emitter_config = figment.extract().map_err(Report::new);
        Ok(AppSettings {
            mode: AppMode::Emitter,
            emitter: Some(emitter_config?),
            absorber: None,
        })
    }

    fn load_absorber_config(args: CliArgs) -> Result<Self> {
        let mut figment = Figment::from(Serialized::defaults(AbsorberConfig::default()));
        let absorber_args = args.command.unwrap(); // TODO: how to do this correctly?
        figment = figment.merge(Serialized::defaults(absorber_args));
        figment.extract().map_err(Report::new)
    }
}
