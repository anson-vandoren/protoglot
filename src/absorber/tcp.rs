use std::sync::Arc;

use async_compression::tokio::bufread::GzipDecoder;
use log::{debug, error, info, trace};
use tokio::{
    io::{AsyncReadExt as _, BufReader},
    net::TcpListener,
};
use tokio_rustls::TlsAcceptor;

use super::{AbsorberInner, ConnOptions, CountingReader, StatsSvc, extract_message, get_cert};
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

        let cert_key = get_cert(&cert_type, self.opts.mtls).await?;
        let acceptor = if let Some(cert_key) = cert_key {
            let key = cert_key.key();
            let builder = rustls::ServerConfig::builder();

            let builder = if self.opts.mtls {
                if let Some(roots) = cert_key.root_cert() {
                    let mut store = rustls::RootCertStore::empty();
                    for root in roots {
                        store.add(root).expect("Failed to add root cert");
                    }
                    let verifier = rustls::server::WebPkiClientVerifier::builder(Arc::new(store))
                        .build()
                        .expect("Failed to build client verifier");
                    builder.with_client_cert_verifier(verifier)
                } else {
                    panic!("mTLS enabled but no root cert available");
                }
            } else {
                builder.with_no_client_auth()
            };

            let config = builder.with_single_cert(cert_key.cert(), key)?;
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
pub(crate) async fn handle_tcp_connection(
    socket: impl tokio::io::AsyncRead + Unpin + Send + 'static,
    stats: &StatsSvc,
    message_type: &MessageType,
) -> tokio::io::Result<()> {
    use tokio::io::AsyncBufReadExt as _;
    let counting_reader = CountingReader::new(socket, stats.clone());
    let mut reader = BufReader::new(counting_reader);

    // Peek for gzip magic bytes (0x1f 0x8b)
    let is_gzip = match reader.fill_buf().await {
        Ok(buf) => buf.len() >= 2 && buf[0] == 0x1f && buf[1] == 0x8b,
        Err(e) => {
            error!("Error peeking into socket: {}", e);
            return Err(e);
        }
    };

    let mut buf = Vec::new();
    if is_gzip {
        debug!("Detected gzipped stream, decompressing...");
        let mut decoder = GzipDecoder::new(reader);
        // We might need to handle multiple members, but for now let's ensure we read to the end of the DECODER
        loop {
            match decoder.read_buf(&mut buf).await {
                Ok(0) => break,
                Ok(_) => {
                    while let Some(message) = extract_message(&mut buf, false) {
                        trace!("Received gzipped message: {:?}", String::from_utf8_lossy(&message));
                        process_message(&message, stats, message_type).await;
                    }
                }
                Err(e) => {
                    error!("Decompression error: {}", e);
                    return Err(e);
                }
            }
        }
        // Final check for remaining messages in the buffer after decoder EOF
        if let Some(message) = extract_message(&mut buf, true) {
            process_message(&message, stats, message_type).await;
        }
    } else {
        loop {
            match reader.read_buf(&mut buf).await {
                Ok(0) => break,
                Ok(_) => {
                    while let Some(message) = extract_message(&mut buf, false) {
                        trace!("Received message: {:?}", String::from_utf8_lossy(&message));
                        process_message(&message, stats, message_type).await;
                    }
                }
                Err(e) => {
                    error!("Read error: {}", e);
                    return Err(e);
                }
            }
        }
        // Final check for remaining messages in the buffer after reader EOF
        if let Some(message) = extract_message(&mut buf, true) {
            process_message(&message, stats, message_type).await;
        }
    }

    debug!("Connection closed normally");
    Ok(())
}

impl From<TcpAbsorber> for AbsorberInner {
    fn from(value: TcpAbsorber) -> Self {
        AbsorberInner::Tcp(value)
    }
}
