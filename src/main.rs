mod absorber;
mod config;
mod emitter;
mod generators;
mod transports;

use std::sync::Arc;

use emitter::{Emitter, EmitterConfig};
use generators::create_generator;
use log::{error, info};
use transports::create_transport;

use crate::{absorber::Absorber, config::AppSettings};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let config = AppSettings::load().unwrap_or_else(|err| {
        error!("Failed to load configuration: {}", err);
        std::process::exit(1);
    });
    if !log::log_enabled!(log::Level::Info) {
        println!("Resolved configuration, starting... Use -v[vv] to see more logs");
    }
    info!(config:serde; "Resolved configuration");

    let mut handles = Vec::new();

    if let Some(emitter_config) = &config.emitter {
        let message_generator = Arc::new(generators::RandomStringGenerator::new());
        for _ in 0..emitter_config.num_emitters {
            let transport = create_transport(&emitter_config).await?;
            let generator =
                create_generator(&emitter_config.message_type, message_generator.clone());
            let emitter_config = EmitterConfig {
                rate: emitter_config.rate,
                num_cycles: emitter_config.num_cycles,
                events_per_cycle: emitter_config.events_per_cycle,
                cycle_delay: emitter_config.cycle_delay,
            };
            let mut emitter = Emitter::new(transport, generator, emitter_config);

            handles.push(tokio::spawn(async move {
                match emitter.run().await {
                    Ok(_) => {
                        info!(emitter = emitter.transport.to_string(); "Emitter completed successfully")
                    }
                    Err(err) => error!("Emitter failed: {}", err),
                }
            }))
        }
    }

    if let Some(absorber_config) = &config.absorber {
        let absorber = Absorber::new(absorber_config.clone());
        handles.push(tokio::spawn(async move {
            if let Err(e) = absorber.run().await {
                error!("Absorber failed: {}", e);
            }
        }))
    }

    // wait for all emitters to complete
    for handle in handles {
        handle.await.expect("Failed to await emitter");
    }
    println!("All emitters completed, exiting...");
    Ok(())
}
