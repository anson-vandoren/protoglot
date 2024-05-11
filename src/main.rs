mod config;
mod generators;
mod emitter;
mod transports;

use std::sync::Arc;

use emitter::Emitter;
use log::info;

use crate::{generators::EventType, transports::TransportType};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    env_logger::init();

    let config = config::Settings::load().expect("Failed to load config, nothing to do");
    if std::env::var("RUST_LOG").is_err() {
        println!("Resolved configuration, starting senders. Set RUST_LOG=debug to see logs.");
    }
    info!(config:serde; "Resolved configuration");
    let message_generator = Arc::new(generators::RandomStringGenerator::new());

    // spawn each emitter as a separate task and collect their handles
    let mut handles = Vec::new();
    for emitter_config in config.emitters {
        let num_emitters = emitter_config.num_emitters;
        for _ in 0..num_emitters {
            let transport = match emitter_config.protocol.as_ref() {
                "tcp" => match emitter_config.tls {
                    true => TransportType::TcpTls(
                        transports::tcp_tls::TcpTlsTransport::new(
                            emitter_config.host.clone(),
                            emitter_config.port,
                        )
                        .await?,
                    ),
                    false => TransportType::Tcp(
                        transports::tcp::TcpTransport::new(
                            emitter_config.host.clone(),
                            emitter_config.port,
                        )
                        .await?,
                    ),
                },
                "udp" => TransportType::Udp(
                    transports::udp::UdpTransport::new(
                        emitter_config.host.clone(),
                        emitter_config.port,
                    )
                    .await?,
                ),
                _ => panic!("Unknown protocol: {}", emitter_config.protocol),
            };
            let generator = match emitter_config.message_type.as_ref() {
                "syslog3164" => {
                    EventType::Syslog3164(
                        generators::Syslog3164EventGenerator::new(message_generator.clone())
                    )
                }
                "syslog5424" => {
                    EventType::Syslog5424(
                        generators::Syslog5424EventGenerator::new(message_generator.clone())
                    )
                }
                _ => panic!("Unknown message type: {}", emitter_config.message_type),
            };
            let config = emitter::EmitterConfig {
                rate: emitter_config.rate,
                num_batches: emitter_config.num_batches,
                events_per_batch: emitter_config.events_per_batch,
                batch_delay: emitter_config.batch_delay,
            };
            let mut emitter = Emitter::new(transport, generator, config);

            handles.push(tokio::spawn(async move {
                emitter.run().await.expect("Failed to run emitter");
            }));
        }
    }

    // wait for all emitters to complete (i.e., in our case, run forever)
    for handle in handles {
        handle.await.expect("Failed to await emitter");
    }
    println!("All emitters completed, exiting...");
    Ok(())
}
