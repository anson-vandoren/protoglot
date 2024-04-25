mod config;
mod generators;
mod sender;
mod transports;

use std::sync::Arc;

use sender::Sender;

use crate::transports::TransportType;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let config = config::Settings::load().expect("Failed to load config, nothing to do");
    println!("{:?}", config);
    let message_generator = Arc::new(
        generators::MessageGenerator::new(&config.message_file)
            .expect("Failed to load message file"),
    );

    // spawn each sender as a separate task and collect their handles
    let mut handles = Vec::new();
    for sender_config in config.senders {
        let num_senders = sender_config.num_senders;
        for _ in 0..num_senders {
            let transport = match sender_config.protocol.as_ref() {
                "tcp" => match sender_config.tls {
                    true => TransportType::TcpTls(transports::tcp_tls::TcpTlsTransport::new(sender_config.host.clone(), sender_config.port).await?),
                    false => TransportType::Tcp(transports::tcp::TcpTransport::new(sender_config.host.clone(), sender_config.port).await?),
                },
                "udp" => {
                    TransportType::Udp(transports::udp::UdpTransport::new(sender_config.host.clone(), sender_config.port).await?)
                }
                _ => panic!("Unknown protocol: {}", sender_config.protocol),
            };
            let generator = match sender_config.message_type.as_ref() {
                "syslog3164" => crate::generators::Syslog3164EventGenerator {
                    message_generator: message_generator.clone(),
                },
                _ => panic!("Unknown message type: {}", sender_config.message_type),
            };
            let config = sender::SenderConfig {
                rate: sender_config.rate,
                num_batches: sender_config.num_batches,
                events_per_batch: sender_config.events_per_batch,
                batch_delay: sender_config.batch_delay,
            };
            let mut sender = Sender::new(transport, generator, config);

            handles.push(tokio::spawn(async move {
                sender.run().await.expect("Failed to run sender");
            }));
        }
    }

    // wait for all senders to complete (i.e., in our case, run forever)
    for handle in handles {
        handle.await.expect("Failed to await sender");
    }
    Ok(())
}
