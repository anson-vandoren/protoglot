mod config;
mod generators;
mod emitter;
mod transports;

use std::sync::Arc;

use emitter::Emitter;
use log::{error, info};

use crate::{config::{MessageType, Protocol}, generators::EventType, transports::TransportType};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    env_logger::init();

    let config = config::EmitterSettings::load().unwrap_or_else(|err| {
        error!("Failed to load configuration: {}", err);
        std::process::exit(1);
    });
    if std::env::var("RUST_LOG").is_err() {
        println!("Resolved configuration, starting senders. Set RUST_LOG=debug to see logs.");
    }
    info!(config:serde; "Resolved configuration");
    let message_generator = Arc::new(generators::RandomStringGenerator::new());

    // spawn each emitter as a separate task and collect their handles
    let mut handles = Vec::new();
    let num_emitters = config.num_emitters;
    for _ in 0..num_emitters {
        let transport = match config.protocol {
            Protocol::Tcp => match config.tls {
                true => TransportType::TcpTls(
                    match transports::tcp_tls::TcpTlsTransport::new(
                        config.host.clone(),
                        config.port,
                    )
                    .await {
                        Ok(transport) => transport,
                        Err(_err) => {
                            // error already logged in TcpTlsTransport
                            continue;
                        }
                    }
                ),
                false => TransportType::Tcp(
                    match transports::tcp::TcpTransport::new(
                        config.host.clone(),
                        config.port,
                    )
                    .await {
                        Ok(transport) => transport,
                        Err(_err) => {
                            // error already logged in TcpTransport
                            continue;
                        }
                    }
                ),
            },
            Protocol::Udp => TransportType::Udp(
                transports::udp::UdpTransport::new(
                    config.host.clone(),
                    config.port,
                )
                .await?,
            ),
        };
        let generator = match config.message_type {
            MessageType::Syslog3164 => {
                EventType::Syslog3164(
                    generators::Syslog3164EventGenerator::new(message_generator.clone())
                )
            }
            MessageType::Syslog5424 => {
                EventType::Syslog5424(
                    generators::Syslog5424EventGenerator::new(message_generator.clone())
                )
            }
        };
        let config = emitter::EmitterConfig {
            rate: config.rate,
            num_cycles: config.num_cycles,
            events_per_cycle: config.events_per_cycle,
            cycle_delay: config.cycle_delay,
        };
        let mut emitter = Emitter::new(transport, generator, config);

        handles.push(tokio::spawn(async move {
            match emitter.run().await {
                Ok(_) => info!(emitter = emitter.transport.to_string(); "Emitter completed successfully"),
                Err(err) => error!("Emitter failed: {}", err),
            }
        }));
    }

    // wait for all emitters to complete
    for handle in handles {
        handle.await.expect("Failed to await emitter");
    }
    println!("All emitters completed, exiting...");
    Ok(())
}
