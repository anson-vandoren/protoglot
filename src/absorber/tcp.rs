use std::sync::Arc;

use log::{debug, error, info, trace};
use tokio::{io::AsyncReadExt as _, net::TcpListener};
use tokio_rustls::TlsAcceptor;

use super::{extract_message, get_cert, AbsorberInner, ConnOptions, StatsSvc};
use crate::{absorber::process_message, config::MessageType};

pub struct TcpAbsorber {
    opts: ConnOptions,
    message_type: MessageType,
}

impl TcpAbsorber {
    pub async fn build(opts: ConnOptions, message_type: MessageType) -> Self {
        Self { message_type, opts }
    }

    pub(super) async fn run(self, stats: StatsSvc) -> anyhow::Result<()> {
        let ConnOptions { addr, cert_type, .. } = self.opts;
        let listener = TcpListener::bind((addr.host.as_str(), addr.port))
            .await
            .expect("Could not bind to TCP address & port");

        let cert_key = get_cert(&cert_type).await?;
        let acceptor = if let Some(cert_key) = cert_key {
            let key = cert_key.key();
            let config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(cert_key.cert(), key)?;
            Some(TlsAcceptor::from(Arc::new(config)))
        } else {
            None
        };

        loop {
            let message_type = self.message_type.clone();
            let (socket, _) = listener.accept().await?;
            let stats = stats.clone();
            let acceptor = acceptor.clone();

            tokio::spawn(async move {
                let remote_addr = socket.peer_addr().unwrap();
                debug!("Accepted TCP connection from: {}", remote_addr);

                if let Some(tls_acceptor) = acceptor {
                    match tls_acceptor.accept(socket).await {
                        Ok(tls_stream) => {
                            info!("TLS handshake successful with {}", remote_addr);
                            if let Err(e) = handle_tcp_connection(tls_stream, &stats, &message_type).await {
                                eprintln!("Error handling TLS TCP connection: {}", e);
                            }
                        }
                        Err(err) => {
                            error!("TLS handshake failed with {}: {:?}", remote_addr, err);
                        }
                    }
                } else {
                    if let Err(e) = handle_tcp_connection(socket, &stats, &message_type).await {
                        eprintln!("Error handling plain TCP connection: {}", e);
                    }
                }
            });
        }
    }
}
async fn handle_tcp_connection(
    mut socket: impl tokio::io::AsyncRead + Unpin,
    stats: &StatsSvc,
    message_type: &MessageType,
) -> tokio::io::Result<()> {
    let mut buf = Vec::new();
    loop {
        match socket.read_buf(&mut buf).await {
            Ok(0) => break, // connection closed
            Ok(_) => {
                if let Some(message) = extract_message(&mut buf, false) {
                    // convert message to string for logging
                    let message_str = String::from_utf8_lossy(&message);
                    trace!("Received message: {:?}", message_str);
                    process_message(&message, stats, message_type).await;
                }
            }
            Err(err) => {
                eprintln!("Error reading from socket: {}", err);
                break;
            }
        }
    }
    Ok(())
}

impl From<TcpAbsorber> for AbsorberInner {
    fn from(value: TcpAbsorber) -> Self {
        AbsorberInner::Tcp(value)
    }
}
