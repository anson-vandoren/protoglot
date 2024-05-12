use clap::{Parser, ValueEnum};
use config;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone)]
enum Protocol {
    Tcp,
    Udp,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        };
        s.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone)]
enum MessageType {
    Syslog3164,
    Syslog5424,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MessageType::Syslog3164 => "syslog3164",
            MessageType::Syslog5424 => "syslog5424",
        };
        s.fmt(f)
    }
}

/// CLI arguments
#[derive(Parser, Serialize, Deserialize, Debug)]
#[command(version, about)]
struct CliArgs {
    /// Path to the configuration file
    #[arg(short, long)]
    file: Option<std::path::PathBuf>,

    /// Target host
    #[arg(short = 'H', long)]
    host: Option<String>,

    /// Target port
    #[arg(short, long)]
    port: Option<u16>,

    /// Rate in events per second
    #[arg(short, long, default_value = "1000")]
    rate: u64,

    /// Use TLS
    #[arg(short, long, default_value = "false")]
    tls: bool,

    /// Protocol to use
    #[arg(short = 'P', long, default_value_t = Protocol::Tcp)]
    protocol: Protocol,

    /// Message type
    #[arg(short, long, default_value_t = MessageType::Syslog3164)]
    message_type: MessageType,

    /// Number of emitters to run in parallel
    #[arg(long = "emitters", default_value = "1")]
    num_emitters: u64,

    /// Number of events per cycle
    #[arg(long = "events", default_value = "10000")]
    events_per_cycle: u64,

    /// Number of cycles to send
    #[arg(long = "cycles", default_value = "1")]
    num_cycles: u64,

    /// Delay between cycles in milliseconds
    #[arg(long, default_value = "0")]
    cycle_delay: u64,
}

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
    #[serde(default = "default_events_per_cycle")]
    pub events_per_cycle: u64,
    #[serde(default = "default_num_cycles")]
    pub num_cycles: u64,
    #[serde(default = "default_cycle_delay")]
    pub cycle_delay: u64,
}

fn default_tls() -> bool {
    false
}

fn default_cycle_delay() -> u64 {
    0
}

fn default_num_cycles() -> u64 {
    1
}

fn default_events_per_cycle() -> u64 {
    10000
}

fn default_num_emitters() -> u64 {
    1
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let args = CliArgs::parse();
        debug!(args:serde; "CLI args received");
        if args.file.is_some() {
            info!(file = args.file.as_ref().unwrap().to_str(); "Loading settings from specified file:");
            let builder = config::Config::builder()
                .add_source(config::File::from(args.file.unwrap()))
                .add_source(config::Environment::with_prefix("BABL"));
            let settings: Settings = builder.build()?.try_deserialize()?;
            return Ok(settings);
        } else if (args.host.is_some() && args.port.is_none()) || (args.port.is_some() && args.host.is_none()) {
            if args.host.is_some() {
                let msg = format!("Host (-H) specified without port (-p). host = {}", args.host.unwrap());
                return Err(config::ConfigError::Message(msg));
            } else {
                let msg = format!("Port (-p) specified without host (-H). port = {}", args.port.unwrap());
                return Err(config::ConfigError::Message(msg));
            }
        } else if args.host.is_some() && args.port.is_some() {
            let host = args.host.unwrap();
            let port = args.port.unwrap();
            info!(host, port; "No config file specified, using CLI args as settings");
            let emitter = EmitterSettings {
                host,
                port,
                rate: args.rate,
                tls: args.tls,
                protocol: args.protocol.to_string(),
                message_type: args.message_type.to_string(),
                num_emitters: args.num_emitters,
                events_per_cycle: args.events_per_cycle,
                num_cycles: args.num_cycles,
                cycle_delay: args.cycle_delay,
            };
            let settings = Settings {
                emitters: vec![emitter],
            };
            return Ok(settings);
        } else {
            info!("Looking for config files in default locations");
            let builder = config::Config::builder()
                .add_source(config::File::with_name("config/default").required(false))
                .add_source(config::File::with_name("bablfsh").required(false))
                .add_source(config::File::with_name("config/local").required(false))
                .add_source(config::Environment::with_prefix("BABL"));
            let settings: Settings = builder.build()?.try_deserialize()?;
            return Ok(settings);
        }
    }
}
