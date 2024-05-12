use clap::Parser;
use serde::{Deserialize, Serialize};

use super::{MessageType, Protocol};

#[derive(Parser, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[command(version, about)]
pub(super) struct CliArgs {
    /// Path to the configuration file
    #[arg(short, long)]
    pub(super) file: Option<std::path::PathBuf>,

    /// Target host
    #[arg(short = 'H', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) host: Option<String>,

    /// Target port
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) port: Option<u16>,

    /// Rate in events per second
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) rate: Option<u64>,

    /// Use TLS
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) tls: Option<bool>,

    /// Protocol to use
    #[arg(short = 'P', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) protocol: Option<Protocol>,

    /// Message type
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) message_type: Option<MessageType>,

    /// Number of emitters to run in parallel
    #[arg(long = "emitters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) num_emitters: Option<u64>,

    /// Number of events per cycle
    #[arg(long = "events")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) events_per_cycle: Option<u64>,

    /// Number of cycles to send
    #[arg(long = "cycles")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) num_cycles: Option<u64>,

    /// Delay between cycles in milliseconds
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) cycle_delay: Option<u64>,
}
