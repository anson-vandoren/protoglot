use std::sync::Arc;

use async_compression::tokio::bufread::GzipDecoder;
use bytes::Bytes;
use flate2::bufread::GzDecoder;
use http_body_util::{BodyExt, StreamBody};
use hyper::{
    header::CONTENT_ENCODING,
    server::conn::{http1, http2},
    service::service_fn,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use log::{debug, error, info};
use tokio::{io::BufReader, net::TcpListener};
use tokio_rustls::TlsAcceptor;
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};
use tokio_util::io::{ReaderStream, StreamReader};

use super::{extract_message, get_cert, validate_message, AbsorberInner, ConnOptions, StatsSvc};
use crate::config::MessageType;

pub struct HttpAbsorber {
    opts: ConnOptions,
    message_type: MessageType,
}

impl HttpAbsorber {
    pub async fn build(opts: ConnOptions, message_type: MessageType) -> Self {
        Self { message_type, opts }
    }

    pub(super) async fn run(self, stats: StatsSvc) -> anyhow::Result<()> {
        debug!("Building a HTTP absorber with opts={:?}", self.opts);
        let ConnOptions { addr, cert_type, .. } = self.opts;
        let listener = TcpListener::bind((addr.host, addr.port))
            .await
            .expect("Could not bind to TCP address & port");
        let cert_key = get_cert(&cert_type).await?;
        let acceptor = if let Some(cert_key) = cert_key {
            let key = cert_key.key();
            let mut config = rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(cert_key.cert(), key)?;
            if self.opts.http_version == hyper::Version::HTTP_2 {
                config.alpn_protocols = vec!["h2".into()];
            } else {
                config.alpn_protocols = vec!["http/1.1".into()];
            }
            //config.alpn_protocols = vec!["h2".into(), "http/1.1".into()];
            Some(TlsAcceptor::from(Arc::new(config)))
        } else {
            None
        };

        let mut listener = TcpListenerStream::new(listener);
        while let Some(s) = listener.next().await {
            match s {
                Ok(s) => {
                    let remote_addr = s.peer_addr().unwrap();
                    info!("Accepted new connection from {}", remote_addr);
                    let message_type = self.message_type.clone();

                    let stats = stats.clone();
                    let acceptor = acceptor.clone();
                    tokio::spawn(async move {
                        let message_type = message_type.clone();
                        let service = service_fn(|req| handle_request(req, stats.clone(), message_type.clone()));

                        // Handle either TLS or non-TLS connection
                        if let Some(tls_acceptor) = acceptor {
                            // TLS connection
                            match tls_acceptor.accept(s).await {
                                Ok(tls_stream) => {
                                    info!("TLS handshake successful with {remote_addr}");
                                    let io = TokioIo::new(tls_stream);
                                    match self.opts.http_version {
                                        hyper::Version::HTTP_2 => {
                                            info!("Starting HTTP2 servicer");
                                            if let Err(err) = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
                                                .serve_connection(io, service)
                                                .await
                                            {
                                                error!("Error in TLS HTTP stream from {remote_addr}: {:?}", err);
                                            }
                                        }
                                        hyper::Version::HTTP_11 => {
                                            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                                                error!("Error in TLS HTTP stream from {remote_addr}: {:?}", err);
                                            }
                                        }
                                        ver => anyhow::bail!("Unsupported HTTP version {:?}", ver),
                                    }
                                }
                                Err(err) => {
                                    anyhow::bail!("TLS handshake failed with {remote_addr}: {:?}", err);
                                }
                            }
                        } else {
                            // Non-TLS connection
                            let io = TokioIo::new(s);
                            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                                anyhow::bail!("Error in plain HTTP stream from {remote_addr}: {:?}", err);
                            }
                        }

                        info!("Connection closed: {remote_addr}");
                        Ok(())
                    });
                }
                Err(e) => {
                    error!("Failed to accept HTTP connection: {}", e);
                }
            }
        }

        Ok(())
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    stats: StatsSvc,
    message_type: MessageType,
) -> Result<Response<String>, hyper::Error> {
    let is_gzipped = req
        .headers()
        .get(CONTENT_ENCODING)
        .map(|value| value.as_bytes())
        .map_or(false, |enc| enc.eq_ignore_ascii_case(b"gzip"));
    let mut body = req.into_body().into_data_stream();
    let mut events = 0;
    let mut bytes = 0;

    if is_gzipped {
        let reader = StreamReader::new(body.map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))));
        let decoder = GzipDecoder::new(reader);
        let decompressed = tokio_util::io::ReaderStream::new(decoder).map(|result| result.map(Bytes::from));
        let mut stream = StreamBody::new(decompressed);
        while let Some(msg) = stream.next().await {
            let mut msg = msg.unwrap().to_vec();
            while let Some(message) = extract_message(&mut msg) {
                validate_message(&message, &message_type);
                events += 1;
                bytes += message.len();
            }
        }
    } else {
        while let Some(msg) = body.next().await {
            let mut msg = msg.unwrap().to_vec();
            while let Some(message) = extract_message(&mut msg) {
                validate_message(&message, &message_type);
                events += 1;
                bytes += message.len();
            }
        }
    }
    stats.increment(events, bytes).await;

    Ok(Response::new("OK".to_string()))
}

impl From<HttpAbsorber> for AbsorberInner {
    fn from(value: HttpAbsorber) -> Self {
        AbsorberInner::Http(value)
    }
}
