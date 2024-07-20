use std::path::Path;
use std::str::FromStr;

use clap::Parser;
use directories::ProjectDirs;
use eyre::{Report, Result};
use log::{debug, info, trace};
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

#[derive(Serialize, Clone, Deserialize, Debug)]
struct FullConfig {
    emitter: Option<EmitterConfig>,
    absorber: Option<AbsorberConfig>,
}

impl EmitterConfig {
    fn default() -> Result<Self> {
        let config_str = include_str!("../../config/default.json5");
        let full_config: FullConfig = serde_json5::from_str(config_str).map_err(Report::new)?;
        Ok(full_config.emitter.unwrap())
    }
}

#[derive(Serialize, Clone, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AbsorberConfig {
    pub listen_addresses: Vec<ListenAddress>,
    pub update_interval: u64,
    pub message_type: MessageType,
}

impl AbsorberConfig {
    fn default() -> Result<Self> {
        let config_str = include_str!("../../config/default.json5");
        let full_config: FullConfig = serde_json5::from_str(config_str).map_err(Report::new)?;
        Ok(full_config.absorber.unwrap())
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
            emitter: if matches!(mode, AppMode::Emitter) {
                Some(config.emitter.unwrap())
            } else {
                None
            },
            absorber: if matches!(mode, AppMode::Absorber) {
                Some(config.absorber.unwrap())
            } else {
                None
            },
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

    fn load_config_file(file_path: &Path) -> Result<FullConfig> {
        if file_path.exists() {
            info!(file = file_path.to_str(); "Using specified file");
            serde_json5::from_str(&std::fs::read_to_string(file_path).map_err(Report::new)?)
                .map_err(Report::new)
        } else {
            Err(Report::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", file_path.display()),
            )))
        }
    }

    fn load_emitter_config(args: CliArgs) -> Result<Self> {
        // start with the baked-in defaults
        let mut config = EmitterConfig::default()?;

        // overwrite with config file from standard location, if present
        if let Some(proj_dirs) = ProjectDirs::from("com", "ansonvandoren", "protoglot") {
            let config_file_path = proj_dirs.config_dir().join("config.json5");
            if config_file_path.exists() {
                let file_config: FullConfig = Self::load_config_file(&config_file_path)?;
                if let Some(emitter) = file_config.emitter {
                    config = emitter;
                }
            }
        }

        // overwrite with passed-in file, if exists
        if let Some(ref file_path) = args.file {
            let file_config: FullConfig = Self::load_config_file(&file_path)?;
            if let Some(emitter) = file_config.emitter {
                config = emitter;
            }
        }

        // overwrite with cli args that are present
        if let Some(host) = args.host {
            config.host = host;
        }
        if let Some(port) = args.port {
            config.port = port;
        }
        if let Some(rate) = args.rate {
            config.rate = rate;
        }
        if let Some(tls) = args.tls {
            config.tls = tls;
        }
        if let Some(protocol) = args.protocol {
            config.protocol = protocol;
        }
        if let Some(message_type) = args.message_type {
            config.message_type = message_type;
        }
        if let Some(num_emitters) = args.num_emitters {
            config.num_emitters = num_emitters;
        }
        if let Some(events_per_cycle) = args.events_per_cycle {
            config.events_per_cycle = events_per_cycle;
        }
        if let Some(num_cycles) = args.num_cycles {
            config.num_cycles = num_cycles;
        }
        if let Some(cycle_delay) = args.cycle_delay {
            config.cycle_delay = cycle_delay;
        }

        Ok(AppSettings {
            mode: AppMode::Emitter,
            emitter: Some(config),
            absorber: None,
        })
    }

    fn load_absorber_config(args: CliArgs) -> Result<Self> {
        // start with the baked-in defaults
        let mut config = AbsorberConfig::default()?;

        // overwrite with config file from standard location, if present
        if let Some(proj_dirs) = ProjectDirs::from("com", "ansonvandoren", "protoglot") {
            let config_file_path = proj_dirs.config_dir().join("config.json5");
            if config_file_path.exists() {
                debug!("Checking for config file at {:?}", config_file_path);
                let file_config: FullConfig = Self::load_config_file(&config_file_path)?;
                if let Some(absorber) = file_config.absorber {
                    debug!("Overwriting config with file");
                    config = absorber;
                } else {
                    debug!("No absorber config in file");
                }
            }
        }

        // overwrite with passed-in file, if exists
        if let Some(ref file_path) = args.file {
            debug!("Checking for config file at {:?}", file_path);
            let file_config: FullConfig = Self::load_config_file(&file_path)?;
            if let Some(absorber) = file_config.absorber {
                debug!("Overwriting config with file");
                config = absorber;
            }
        }

        // overwrite with cli args that are present
        if let Some(Commands::Absorber {
            update_interval,
            listen_addresses,
            message_type,
        }) = &args.command
        {
            debug!("Overwriting config with CLI args");
            if let Some(interval) = update_interval {
                debug!("Setting update interval to {}", interval);
                config.update_interval = *interval;
            }
            if let Some(addrs) = listen_addresses {
                config.listen_addresses = addrs
                    .iter()
                    .map(|addr| ListenAddress::from_str(addr).unwrap())
                    .collect();
                debug!("Setting listen addresses to {:?}", config.listen_addresses);
            }
            if let Some(msg_type) = message_type {
                config.message_type = *msg_type;
            }
        }

        Ok(AppSettings {
            mode: AppMode::Absorber,
            emitter: None,
            absorber: Some(config),
        })
    }
}
