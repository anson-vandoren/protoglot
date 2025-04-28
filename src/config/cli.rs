use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use super::{absorber::HttpAuth, MessageType, Protocol};

#[derive(Parser, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[command(version, about)]
pub struct CliArgs {
    /// Path to the configuration file
    #[arg(short, long)]
    pub file: Option<PathBuf>,

    /// Target host
    #[arg(short = 'H', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Target port
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Rate in events per second
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<u64>,

    /// Use TLS
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,

    /// Protocol to use
    #[arg(short = 'P', long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Protocol>,

    /// Message type
    #[arg(short, long, value_enum)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,

    /// Number of emitters to run in parallel
    #[arg(long = "emitters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_emitters: Option<u64>,

    /// Number of events per cycle
    #[arg(long = "events")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_per_cycle: Option<u64>,

    /// Number of cycles to send
    #[arg(long = "cycles")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_cycles: Option<u64>,

    /// Delay between cycles in milliseconds
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_delay: Option<u64>,

    /// Control output verbosity
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Commands {
    /// Start an absorber instead
    Absorber {
        /// Update interval for absorber stats in milliseconds
        #[arg(long = "update-interval")]
        #[serde(skip_serializing_if = "Option::is_none")]
        update_interval: Option<u64>,

        /// Listen addresses for absorber (format: host:port:protocol, can be specified multiple times)
        #[arg(long = "listen")]
        #[serde(skip_serializing_if = "Option::is_none")]
        listen_addresses: Option<Vec<String>>,

        /// Message type
        #[arg(long = "message-type")]
        #[serde(skip_serializing_if = "Option::is_none")]
        message_type: Option<MessageType>,

        /// HTTP2-only server (if listening for HTTP). This implies HTTPS and is mutually exclusive
        /// with the --https flag
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "https")]
        http2: Option<bool>,

        /// HTTPS/1.1-only server (if listening for HTTP). With no other flags set, this will use
        /// local.fucktls.com certs. Mutually exclusive with the --http2 flag
        #[arg(long, action = ArgAction::SetTrue, conflicts_with = "http2")]
        https: Option<bool>,

        /// Use a plain self-signed server cert (if listening for HTTP and either HTTP2 or HTTPS is
        /// enabled). Mutually exclusive with --private-ca flag.
        #[arg(long = "self-signed", action = ArgAction::SetTrue, conflicts_with = "private_ca")]
        #[serde(skip_serializing_if = "Option::is_none")]
        self_signed: Option<bool>,

        /// Use an internally-generated private CA cert to sign the server cert (if listening for
        /// HTTP and either HTTP2 or HTTPS is enabled). Mutually exclusive with --self-signed flag.
        #[arg(long = "private-ca", action = ArgAction::SetTrue, conflicts_with = "self_signed")]
        #[serde(skip_serializing_if = "Option::is_none")]
        private_ca: Option<bool>,

        /// Auth mechanism for HTTP server
        #[arg(long, value_enum)]
        #[serde(skip_serializing_if = "Option::is_none")]
        auth: Option<HttpAuth>,
    },

    /// Write the default config to expected path, if one does not already exist
    Config {
        /// Overwrite the config file if it exists, saving the old one to a .$timestamp.bak
        #[arg(long)]
        #[serde(skip_serializing_if = "Option::is_none")]
        overwrite: Option<bool>,
    },
}
