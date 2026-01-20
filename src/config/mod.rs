pub mod absorber;
pub mod cli;
pub mod emitter;
mod types;

use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
    str::FromStr,
};

use absorber::{AbsorberConfig, PartialAbsorberConfig};
use cli::{CliArgs, Commands};
use directories::ProjectDirs;
pub use emitter::EmitterConfig;
use emitter::PartialEmitterConfig;
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
pub use types::{MessageType, Protocol};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub mode: AppMode,
    pub emitter: Option<EmitterConfig>,
    pub absorber: Option<AbsorberConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum AppMode {
    Emitter,
    Absorber,
    Config,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FullConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emitter: Option<PartialEmitterConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absorber: Option<PartialAbsorberConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenAddress {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
}

impl ToSocketAddrs for ListenAddress {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (self.host.as_str(), self.port).to_socket_addrs()
    }
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

impl TryFrom<&str> for ListenAddress {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // ex: "tcp://127.0.0.1:4242"
        let parts: Vec<&str> = value.split("://").collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid listen address format"));
        }
        let protocol = Protocol::try_from(parts[0])?;
        let parts: Vec<&str> = parts[1].split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid listen address format"));
        }
        let host = parts[0].to_string();
        let port = parts[1].parse()?;
        Ok(Self { host, port, protocol })
    }
}

impl AppSettings {
    pub fn load(args: CliArgs) -> anyhow::Result<Self> {
        Self::setup_logging(args.verbose);

        Ok(match &args.command {
            Some(Commands::Absorber { .. }) => {
                trace!("Starting absorber");
                Self {
                    emitter: None,
                    absorber: Self::load_absorber_config(args)?.absorber,
                    mode: AppMode::Absorber,
                }
            }
            Some(Commands::Config { overwrite }) => {
                write_default_config(overwrite.unwrap_or(false))?;
                Self {
                    emitter: None,
                    absorber: None,
                    mode: AppMode::Config,
                }
            }
            None => {
                info!("No command specified, starting emitter");
                Self {
                    absorber: None,
                    emitter: Self::load_emitter_config(args)?.emitter,
                    mode: AppMode::Emitter,
                }
            }
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

    fn load_emitter_config(args: CliArgs) -> anyhow::Result<Self> {
        // start with the baked-in defaults
        let mut config = EmitterConfig::default();

        // overwrite with config file from standard location, if present
        config = config.merge_from(load_default_config_file());

        // overwrite with passed-in file, if exists
        if let Some(ref file_path) = args.file {
            let file_config: FullConfig = load_config_file(file_path)?;
            debug!("Overwriting config with file {}", file_path.display());
            config = config.merge_from(Some(file_config));
        }

        // overwrite with cli args that are present
        let cli_args: PartialEmitterConfig = args.into();
        let config = config.merge(cli_args);

        Ok(AppSettings {
            mode: AppMode::Emitter,
            emitter: Some(config),
            absorber: None,
        })
    }

    fn load_absorber_config(args: CliArgs) -> anyhow::Result<Self> {
        // start with the baked-in defaults
        let mut config = AbsorberConfig::default();

        // overwrite with config file from standard location, if present
        config = config.merge_from(load_default_config_file());

        // overwrite with passed-in file, if exists
        if let Some(ref file_path) = args.file {
            let file_config = load_config_file(file_path)?;
            debug!("Overwriting config with file {}", file_path.display());
            config = config.merge_from(Some(file_config));
        }

        // overwrite with cli args that are present
        let cli_args: PartialAbsorberConfig = args.command.into();
        debug!("CLI args: {:?}", cli_args);
        let config = config.merge(cli_args);
        debug!("Config after CLI args: {:?}", config);

        Ok(AppSettings {
            mode: AppMode::Absorber,
            emitter: None,
            absorber: Some(config),
        })
    }
}

fn load_config_file(file_path: &Path) -> anyhow::Result<FullConfig> {
    if file_path.exists() {
        info!(file = file_path.to_str(); "Using specified file");
        Ok(serde_json5::from_str(&std::fs::read_to_string(file_path)?)?)
    } else {
        anyhow::bail!("File not found: '{}", file_path.display());
    }
}

fn load_default_config_file() -> Option<FullConfig> {
    let config_file = default_config_path();
    if config_file.exists() {
        debug!("Found config file at {:?}", config_file);
        return load_config_file(&config_file).ok();
    }
    debug!("No config file at {:?}", config_file);
    None
}

fn default_config_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "ansonvandoren", "protoglot").expect("$HOME directory not found.");
    proj_dirs.config_dir().join("config.json5")
}

fn write_default_config(overwrite: bool) -> anyhow::Result<()> {
    trace!("Writing out config file");
    let config_file = default_config_path();
    let fname = config_file.to_string_lossy();
    let should_write = match (config_file.exists(), overwrite) {
        (true, false) => {
            println!("Config file already exists at {fname}. Use '--overwrite true' to replace it.");
            false
        }
        (false, _) => {
            trace!("No file at '{}', writing out default.", fname);
            true
        }
        (true, true) => {
            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
            let mut backup_fname = config_file.clone();
            backup_fname.set_file_name(format!("config.{ts}.json5.bak"));
            println!(
                "Backing up existing config to '{}' before writing defaults.",
                backup_fname.display()
            );
            let old = std::fs::read_to_string(&config_file)?;
            std::fs::write(&backup_fname, old)?;
            true
        }
    };

    if should_write {
        let default_emitter = EmitterConfig::default();
        let default_absorber = AbsorberConfig::default();
        let cfg = FullConfig {
            emitter: Some(default_emitter.into()),
            absorber: Some(default_absorber.into()),
        };
        let parent = config_file.parent().unwrap();
        std::fs::create_dir_all(parent)?;
        std::fs::write(&config_file, serde_json::to_string_pretty(&cfg)?)?;
        println!("Wrote default configuration to {}", config_file.display())
    }

    Ok(())
}

#[cfg(test)]
mod test_config_subcommand {
    use clap::Parser as _;
    use pretty_assertions::{assert_eq, assert_ne};
    use sealed_test::prelude::*;

    use super::*;
    use crate::config::cli::CliArgs;

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn writes_config_file_with_config_subcommand() {
        let args = ["protoglot", "config"];
        let args = CliArgs::parse_from(args.iter());

        AppSettings::load(args).unwrap();

        let dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let expected_path = dir.config_dir().join("config.json5");
        assert!(expected_path.exists());
    }

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn does_not_overwrite() {
        let args = ["protoglot", "config"];
        let args = CliArgs::parse_from(args.iter());
        let dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let expected_path = dir.config_dir().join("config.json5");

        std::fs::create_dir_all(&expected_path.parent().unwrap()).unwrap();
        std::fs::write(&expected_path, "test").unwrap();

        AppSettings::load(args).unwrap();

        assert!(expected_path.exists());
        let read_back = std::fs::read_to_string(&expected_path).unwrap();
        assert_eq!(read_back, "test");
    }

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn does_overwrite() {
        let args = ["protoglot", "config", "--overwrite", "true"];
        let args = CliArgs::parse_from(args.iter());
        let dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let expected_path = dir.config_dir().join("config.json5");

        std::fs::create_dir_all(&expected_path.parent().unwrap()).unwrap();
        std::fs::write(&expected_path, "test").unwrap();

        AppSettings::load(args).unwrap();

        assert!(expected_path.exists());
        let read_back = std::fs::read_to_string(&expected_path).unwrap();
        assert_ne!(read_back, "test");
        let value: serde_json::Value = serde_json5::from_str(&read_back).expect("Should have been JSON.");
        let emitter = value.get("emitter");
        assert!(emitter.is_some());
        let absorber = value.get("absorber");
        assert!(absorber.is_some());

        let files = std::fs::read_dir(dir.config_dir()).unwrap();
        let files = files.map(Result::unwrap).collect::<Vec<_>>();
        assert!(files.len() > 1);
        for file in files {
            let f = file.path();
            if f != expected_path {
                // This should be our backup
                let backup_str = std::fs::read_to_string(f).unwrap();
                assert_eq!(backup_str, "test");
            }
        }
    }
}

#[cfg(test)]
mod test_load_absorber {
    use std::path::PathBuf;

    use clap::Parser as _;
    use pretty_assertions::{assert_eq, assert_matches};
    use sealed_test::prelude::*;

    use super::*;
    use crate::config::{cli::CliArgs, MessageType};

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn uses_defaults_when_nothing_else() {
        let args = ["protoglot", "absorber"];
        let args = CliArgs::parse_from(args.iter());

        let config = AppSettings::load_absorber_config(args).unwrap();

        assert_matches!(config.mode, AppMode::Absorber);
        assert!(config.emitter.is_none());
        let found: AbsorberConfig = config.absorber.unwrap().try_into().unwrap();
        assert_eq!(found, AbsorberConfig::default());
    }

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn uses_base_config_file() {
        // write a base config file
        let config = PartialAbsorberConfig {
            update_interval: Some(4242),
            listen_addresses: None,
            message_type: None,
            ..Default::default()
        };
        let config = FullConfig {
            emitter: None,
            absorber: Some(config),
        };
        let config_dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let config_path = config_dir.config_dir().join("config.json5");

        std::fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
        let config_as_str = serde_json::to_string(&config).unwrap();
        std::fs::write(&config_path, config_as_str).unwrap();

        let args = ["protoglot", "absorber"];
        let args = CliArgs::parse_from(args.iter());

        let config = AppSettings::load_absorber_config(args).unwrap();

        assert_matches!(config.mode, AppMode::Absorber);
        assert!(config.emitter.is_none());
        let found: AbsorberConfig = config.absorber.unwrap().try_into().unwrap();
        assert_matches!(
            found,
            AbsorberConfig {
                update_interval: 4242,
                message_type,
                listen_addresses,
                ..
            } if message_type == MessageType::Syslog3164 && listen_addresses.is_empty()
        );
    }

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn merges_other_file_with_base_config() {
        // write a base config file
        let config = PartialAbsorberConfig {
            update_interval: Some(4242),
            listen_addresses: None,
            message_type: Some(MessageType::Syslog5424),
            ..Default::default()
        };
        let config = FullConfig {
            emitter: None,
            absorber: Some(config),
        };
        let config_dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let config_path = config_dir.config_dir().join("config.json5");
        std::fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
        let config_as_str = serde_json::to_string(&config).unwrap();
        std::fs::write(&config_path, config_as_str).unwrap();

        // write a 'custom' config file
        let other_config = PartialAbsorberConfig {
            update_interval: Some(4243),
            listen_addresses: None,
            message_type: None,
            ..Default::default()
        };
        let other_config = FullConfig {
            emitter: None,
            absorber: Some(other_config),
        };

        let config_as_str = serde_json::to_string(&other_config).unwrap();
        std::fs::write(PathBuf::from("./my_config.json5"), config_as_str).unwrap();

        let args = ["protoglot", "--file", "./my_config.json5", "absorber"];
        let args = CliArgs::parse_from(args.iter());

        let config = AppSettings::load_absorber_config(args).unwrap();

        assert_matches!(config.mode, AppMode::Absorber);
        assert!(config.emitter.is_none());
        let found: AbsorberConfig = config.absorber.unwrap().try_into().unwrap();
        assert_matches!(
            found,
            AbsorberConfig {
                update_interval: 4243,
                message_type,
                listen_addresses,
                ..
            } if message_type == MessageType::Syslog5424 && listen_addresses.is_empty()
        );
    }

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn merges_cli_opts_on_top() {
        // write a base config file
        let config = PartialAbsorberConfig {
            update_interval: Some(4242),
            listen_addresses: None,
            message_type: Some(MessageType::Syslog5424),
            ..Default::default()
        };
        let config = FullConfig {
            emitter: None,
            absorber: Some(config),
        };
        let config_dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let config_path = config_dir.config_dir().join("config.json5");
        std::fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
        let config_as_str = serde_json::to_string(&config).unwrap();
        std::fs::write(&config_path, config_as_str).unwrap();

        // write a 'custom' config file
        let other_config = PartialAbsorberConfig {
            update_interval: Some(4243),
            listen_addresses: None,
            message_type: None,
            ..Default::default()
        };
        let other_config = FullConfig {
            emitter: None,
            absorber: Some(other_config),
        };

        let config_as_str = serde_json::to_string(&other_config).unwrap();
        std::fs::write(PathBuf::from("./my_config.json5"), config_as_str).unwrap();

        let args = [
            "protoglot",
            "--file",
            "./my_config.json5",
            "absorber",
            "--update-interval",
            "5000",
            "--message-type",
            "nd-json",
        ];
        let args = CliArgs::parse_from(args.iter());

        let config = AppSettings::load_absorber_config(args).unwrap();

        assert_matches!(config.mode, AppMode::Absorber);
        assert!(config.emitter.is_none());
        let found: AbsorberConfig = config.absorber.unwrap().try_into().unwrap();
        assert_matches!(
            found,
            AbsorberConfig {
                update_interval: 5000,
                message_type,
                listen_addresses,
                ..
            } if message_type == MessageType::NdJson && listen_addresses.is_empty()
        );
    }
}

#[cfg(test)]
mod test_load_emitter {
    use std::path::PathBuf;

    use clap::Parser as _;
    use pretty_assertions::{assert_eq, assert_matches, assert_ne};
    use sealed_test::prelude::*;

    use super::*;
    use crate::config::cli::CliArgs;

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn uses_defaults_when_nothing_else() {
        let args = ["protoglot"];
        let args = CliArgs::parse_from(args.iter());

        let config = AppSettings::load_emitter_config(args).unwrap();

        assert_matches!(config.mode, AppMode::Emitter);
        assert!(config.absorber.is_none());
        let found: EmitterConfig = config.emitter.unwrap().try_into().unwrap();
        assert_eq!(found, EmitterConfig::default());
    }

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn uses_base_config_file() {
        // write a base config file
        let config = PartialEmitterConfig {
            port: Some(4242),
            ..PartialEmitterConfig::default()
        };
        assert_ne!(EmitterConfig::default().port, 4242);
        let config = FullConfig {
            absorber: None,
            emitter: Some(config),
        };
        let config_dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let config_path = config_dir.config_dir().join("config.json5");

        std::fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
        let config_as_str = serde_json::to_string(&config).unwrap();
        std::fs::write(&config_path, config_as_str).unwrap();

        let args = ["protoglot"];
        let args = CliArgs::parse_from(args.iter());

        let config = AppSettings::load_emitter_config(args).unwrap();

        assert_matches!(config.mode, AppMode::Emitter);
        assert!(config.absorber.is_none());
        let found: EmitterConfig = config.emitter.unwrap().try_into().unwrap();
        assert_matches!(found, EmitterConfig { port: 4242, .. });
    }

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn merges_other_file_with_base_config() {
        // write a base config file
        let config = PartialEmitterConfig {
            port: Some(4242),
            host: Some("icanhashostname.com".into()),
            ..Default::default()
        };
        let config = FullConfig {
            absorber: None,
            emitter: Some(config),
        };
        let config_dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let config_path = config_dir.config_dir().join("config.json5");
        std::fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
        let config_as_str = serde_json::to_string(&config).unwrap();
        std::fs::write(&config_path, config_as_str).unwrap();

        // write a 'custom' config file
        let other_config = PartialEmitterConfig {
            port: Some(4243),
            ..Default::default()
        };
        let other_config = FullConfig {
            absorber: None,
            emitter: Some(other_config),
        };

        let config_as_str = serde_json::to_string(&other_config).unwrap();
        std::fs::write(PathBuf::from("./my_config.json5"), config_as_str).unwrap();

        let args = ["protoglot", "--file", "./my_config.json5", "absorber"];
        let args = CliArgs::parse_from(args.iter());

        let config = AppSettings::load_emitter_config(args).unwrap();

        assert_matches!(config.mode, AppMode::Emitter);
        assert!(config.absorber.is_none());
        let found: EmitterConfig = config.emitter.unwrap().try_into().unwrap();
        assert_matches!(
            found,
            EmitterConfig {
                port: 4243,
                host,
                ..
            } if host == "icanhashostname.com".to_string()
        );
    }

    #[sealed_test(env = [("XDG_CONFIG_HOME", "./.config"), ("HOME", "./")])]
    fn merges_cli_opts_on_top() {
        // write a base config file
        let config = PartialEmitterConfig {
            port: Some(4242),
            host: Some("icanhashostname.com".into()),
            ..PartialEmitterConfig::default()
        };
        let config = FullConfig {
            absorber: None,
            emitter: Some(config),
        };
        let config_dir = ProjectDirs::from("com", "ansonvandoren", "protoglot").unwrap();
        let config_path = config_dir.config_dir().join("config.json5");
        std::fs::create_dir_all(&config_path.parent().unwrap()).unwrap();
        let config_as_str = serde_json::to_string(&config).unwrap();
        std::fs::write(&config_path, config_as_str).unwrap();

        // write a 'custom' config file
        let other_config = PartialEmitterConfig {
            port: Some(4243),
            ..Default::default()
        };
        let other_config = FullConfig {
            absorber: None,
            emitter: Some(other_config),
        };

        let config_as_str = serde_json::to_string(&other_config).unwrap();
        std::fs::write(PathBuf::from("./my_config.json5"), config_as_str).unwrap();

        let args = [
            "protoglot",
            "--file",
            "./my_config.json5",
            "--port",
            "11000",
            "--host",
            "someotherhostname.com",
        ];
        let args = CliArgs::parse_from(args.iter());

        let config = AppSettings::load_emitter_config(args).unwrap();

        assert_matches!(config.mode, AppMode::Emitter);
        assert!(config.absorber.is_none());
        let found: EmitterConfig = config.emitter.unwrap().try_into().unwrap();
        assert_matches!(
            found,
            EmitterConfig {
                port: 11000,
                host,
                ..
            } if host == "someotherhostname.com".to_string()
        );
    }
}
