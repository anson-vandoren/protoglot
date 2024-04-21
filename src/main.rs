pub mod sender;
mod serializers;
mod transports;
mod generators;

use sender::Sender;
use serializers::ndjson::NdJsonSerializer;
use std::time::Duration;
use transports::tcp_tls::TcpTlsTransport;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let fqdn = "default.main.exciting-sharp-d7ds9oz.cribl-staging.cloud";
    let port = 10070;

    let mut fast_sender = Sender {
        transport: TcpTlsTransport::new(fqdn.to_string(), port).await?,
        serializer: NdJsonSerializer,
        generator: generators::Syslog3164EventGenerator,
        rate: Duration::from_millis(10),
    };

    // let mut slow_sender = Sender {
    //     transport: TcpTlsTransport::new(fqdn.to_string(), port).await?,
    //     serializer: NdJsonSerializer,
    //     rate: Duration::from_secs(1),
    // };

    let _handles = vec![
        tokio::spawn(async move {
            fast_sender.run().await.unwrap();
        }),
        // tokio::spawn(async move {
        //     slow_sender.run().await.unwrap();
        // }),
    ];

    for handle in _handles {
        handle.await?;
    }

    Ok(())
}
