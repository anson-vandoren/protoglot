use clap::{Parser, ValueEnum};
use config;
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

// CLI arguments
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
    rate: Option<u64>,

    /// Use TLS
    #[arg(short, long, default_value = "false")]
    tls: Option<bool>,

    /// Protocol to use
    #[arg(short = 'P', long)]
    protocol: Option<Protocol>,

    /// Message type
    #[arg(short, long, default_value = "syslog3164")]
    message_type: Option<String>,

    /// Number of emitters to run in parallel
    #[arg(long = "emitters", default_value = "1")]
    num_emitters: Option<u64>,

    /// Number of events per batch
    #[arg(long = "events", default_value = "10000")]
    events_per_batch: Option<u64>,

    /// Number of batches to send
    #[arg(long = "batches", default_value = "1")]
    num_batches: Option<u64>,

    /// Delay between batches in milliseconds
    #[arg(long, default_value = "0")]
    batch_delay: Option<u64>,
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
        let args = CliArgs::parse();
        println!("{:?}", args);
        if args.file.is_some() {
            println!("Loading settings from file {:?}", args.file.as_ref().unwrap());
            let builder = config::Config::builder()
                .add_source(config::File::from(args.file.unwrap()))
                .add_source(config::Environment::with_prefix("BABL"));
            let settings: Settings = builder.build()?.try_deserialize()?;
            return Ok(settings);
        } else if args.host.is_some() && args.port.is_some() {
            println!("Loading settings from CLI arguments");
            let emitter = EmitterSettings {
                host: args.host.unwrap(),
                port: args.port.unwrap(),
                rate: args.rate.unwrap(),
                tls: args.tls.unwrap(),
                protocol: args.protocol.unwrap().to_string(),
                message_type: args.message_type.unwrap(),
                num_emitters: args.num_emitters.unwrap(),
                events_per_batch: args.events_per_batch.unwrap(),
                num_batches: args.num_batches.unwrap(),
                batch_delay: args.batch_delay.unwrap(),
            };
            let settings = Settings {
                emitters: vec![emitter],
            };
            return Ok(settings);
        } else {
            println!("Loading settings from default and local config files");
            let builder = config::Config::builder()
                .add_source(config::File::with_name("config/default"))
                .add_source(config::File::with_name("bablfsh").required(false))
                .add_source(config::File::with_name("config/local").required(false))
                .add_source(config::Environment::with_prefix("BABL"));
            let settings: Settings = builder.build()?.try_deserialize()?;
            return Ok(settings);
        }
    }
}
