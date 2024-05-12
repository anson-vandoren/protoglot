use clap::{Parser, ValueEnum};
use config;
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, ValueEnum, Clone)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Protocol {
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
#[serde(rename_all = "lowercase")]
pub(crate) enum MessageType {
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
#[serde(rename_all = "camelCase")]
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
    pub fn load() -> Result<Self, config::ConfigError> {
        let args = CliArgs::parse();
        debug!(args:serde; "CLI args received");
        if args.file.is_some() {
            info!(file = args.file.as_ref().unwrap().to_str(); "Loading settings from specified file:");
            let builder = config::Config::builder()
                .add_source(config::File::from(args.file.unwrap()))
                .add_source(config::Environment::with_prefix("BABL"));
            let settings: EmitterSettings = builder.build()?.try_deserialize()?;
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
                protocol: args.protocol,
                message_type: args.message_type,
                num_emitters: args.num_emitters,
                events_per_cycle: args.events_per_cycle,
                num_cycles: args.num_cycles,
                cycle_delay: args.cycle_delay,
            };
            return Ok(emitter);
        } else {
            info!("Looking for config files in default locations");
            let builder = config::Config::builder()
                .add_source(config::File::with_name("config/default").required(false))
                .add_source(config::File::with_name("bablfsh").required(false))
                .add_source(config::File::with_name("config/local").required(false))
                .add_source(config::Environment::with_prefix("BABL"));
            let settings: EmitterSettings = builder.build()?.try_deserialize()?;
            return Ok(settings);
        }
    }
}
