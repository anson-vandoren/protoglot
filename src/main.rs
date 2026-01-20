mod absorber;
mod config;
mod emitter;
mod generators;
mod transports;

use clap::Parser as _;
use config::AppMode;
use emitter::{Emitter, EmitterConfig};
use generators::create_generator;
use log::{error, info};
use tokio::task::JoinSet;
use transports::create_transport;

use crate::{absorber::Absorber, config::AppSettings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::ring::default_provider().install_default().unwrap();
    let args = config::cli::CliArgs::parse();
    let config = AppSettings::load(args).unwrap_or_else(|err| {
        error!("Failed to load configuration: {}", err);
        std::process::exit(1);
    });
    if matches!(config.mode, AppMode::Config) {
        return Ok(());
    }
    if !log::log_enabled!(log::Level::Info) {
        println!("Resolved configuration, starting... Use -v[vv] to see more logs");
    }
    info!(config:serde; "Resolved configuration");

    let mut handles = JoinSet::new();

    if let Some(emitter_config) = &config.emitter {
        for _ in 0..emitter_config.num_emitters {
            let transport = create_transport(emitter_config).await?;
            let generator = create_generator(&emitter_config.message_type);
            let emitter_config = EmitterConfig {
                rate: emitter_config.rate,
                num_cycles: emitter_config.num_cycles,
                events_per_cycle: emitter_config.events_per_cycle,
                cycle_delay: emitter_config.cycle_delay,
            };
            let mut emitter = Emitter::new(transport, generator, emitter_config);

            handles.spawn(async move {
                match emitter.run().await {
                    Ok(_) => {
                        info!(emitter = emitter.transport.to_string(); "Emitter completed successfully");
                        info!(total_events = emitter.total_events, total_bytes = emitter.total_bytes; "Totals");
                    }
                    Err(err) => error!("Emitter failed: {}", err),
                }
            });
        }
    }

    if let Some(absorber_config) = &config.absorber {
        let absorber = Absorber::new(absorber_config.clone());
        handles.spawn(async move {
            if let Err(e) = absorber.run().await {
                error!("Absorber failed: {}", e);
            }
        });
    }

    // wait for all emitters to complete
    handles.join_all().await;
    println!("All emitters completed, exiting...");
    Ok(())
}
