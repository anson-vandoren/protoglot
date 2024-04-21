mod generators;
mod sender;
mod transports;
mod config;

use std::sync::Arc;

use transports::tcp_tls::TcpTlsTransport;

use sender::Sender;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let config = config::Settings::load().expect("Failed to load config, nothing to do");
    println!("{:?}", config);
    let message_generator = Arc::new(
        generators::MessageGenerator::new(&config.message_file).expect("Failed to load message file")
    );

    // spawn each sender as a separate task and collect their handles
    let mut handles = Vec::new();
    for sender_config in config.senders {
        let transport = TcpTlsTransport::new(sender_config.host, sender_config.port).await?;
        let generator = match sender_config.message_type.as_ref() {
            "syslog3164" => crate::generators::Syslog3164EventGenerator {
                message_generator: message_generator.clone(),
            },
            _ => panic!("Unknown message type: {}", sender_config.message_type),
        };
        let mut sender = Sender{ 
            transport, 
            generator, 
            rate: sender_config.rate,
        };

        handles.push(tokio::spawn(async move {
            sender.run().await.expect("Failed to run sender");
        }));
    }

    // wait for all senders to complete (i.e., in our case, run forever)
    for handle in handles {
        handle.await.expect("Failed to await sender");
    }
    Ok(())
}
