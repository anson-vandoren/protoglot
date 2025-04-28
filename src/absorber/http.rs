use std::sync::Arc;

use async_compression::tokio::bufread::GzipDecoder;
use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{
    header::CONTENT_ENCODING,
    server::conn::{http1, http2},
    service::service_fn,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use log::{debug, error, info};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};
use tokio_util::io::StreamReader;

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
                    let expected_token = self.opts.token.clone();

                    let stats = stats.clone();
                    let acceptor = acceptor.clone();
                    tokio::spawn(async move {
                        let message_type = message_type.clone();
                        let service = service_fn(|req| handle_request(req, stats.clone(), message_type.clone(), expected_token.clone()));

                        // Handle either TLS or non-TLS connection
                        if let Some(tls_acceptor) = acceptor {
                            // TLS connection
                            match tls_acceptor.accept(s).await {
                                Ok(tls_stream) => {
                                    info!("TLS handshake successful with {remote_addr}");
                                    let io = TokioIo::new(tls_stream);
                                    match self.opts.http_version {
                                        hyper::Version::HTTP_2 => {
                                            info!("Starting HTTP/2 servicer");
                                            if let Err(err) = http2::Builder::new(hyper_util::rt::TokioExecutor::new())
                                                .serve_connection(io, service)
                                                .await
                                            {
                                                error!("Error in TLS HTTP stream from {remote_addr}: {:?}", err);
                                            }
                                        }
                                        hyper::Version::HTTP_11 => {
                                            info!("Starting HTTP/1.1 servicer");
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
    token: String,
) -> Result<Response<String>, hyper::Error> {
    if let Err(err) = check_auth(&req, token) {
        return Ok(err);
    }
    let stream = get_decompressed(req);

    let StatsUpdate { events, bytes } = match process_messages(stream, message_type).await {
        Ok(stats) => stats,
        Err(err) => return Ok(err),
    };
    stats.increment(events, bytes).await;

    Ok(Response::new("OK".to_string()))
}

fn check_auth(req: &Request<hyper::body::Incoming>, expected: String) -> Result<(), Response<String>> {
    let token = req
        .headers()
        .get(hyper::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string());

    if let Some(auth_value) = &token {
        if auth_value == &expected {
            return Ok(());
        }
    }
    error!("Unauthorized access attempt. Wanted: {}, got: {:?}", expected, token);
    Err(Response::builder()
        .status(hyper::StatusCode::UNAUTHORIZED)
        .body("Unauthorized".to_string())
        .unwrap())
}

struct StatsUpdate {
    events: usize,
    bytes: usize,
}
async fn process_messages(stream: Stream, message_type: MessageType) -> Result<StatsUpdate, Response<String>> {
    let mut msg = Vec::new();
    let mut events = 0;
    let mut bytes = 0;

    let mut extract_all = |msg: &mut Vec<u8>, fin: bool| -> Result<(), Response<String>> {
        while let Some(message) = extract_message(msg, fin) {
            if !validate_message(&message, &message_type) {
                error!(
                    "Invalid message received. Expected type: {:?}, found {:?}",
                    message_type,
                    String::from_utf8_lossy(&message)
                );
                return Err(Response::builder()
                    .status(hyper::StatusCode::BAD_REQUEST)
                    .body("Invalid message format".to_string())
                    .unwrap());
            }
            events += 1;
            bytes += message.len();
        }
        Ok(())
    };

    tokio::pin!(stream);
    while let Some(next_msg) = stream.next().await {
        match next_msg {
            Ok(data) => {
                msg.extend(data.to_vec());
                extract_all(&mut msg, false)?;
            }
            Err(e) => {
                error!("Error processing message: {}", e);
                return Err(Response::builder()
                    .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                    .body(format!("Error processing message: {}", e))
                    .unwrap());
            }
        }
    }
    // Try for any residual messages in the buffer
    extract_all(&mut msg, true)?;
    if msg.len() > 0 {
        error!("Received message with trailing data: {}", String::from_utf8_lossy(&msg));
        return Err(Response::builder()
            .status(hyper::StatusCode::BAD_REQUEST)
            .body("Received message with trailing data".to_string())
            .unwrap());
    }

    Ok(StatsUpdate { events, bytes })
}

type Stream = Box<dyn tokio_stream::Stream<Item = anyhow::Result<Bytes>> + Unpin + Send>;
fn get_decompressed(req: Request<hyper::body::Incoming>) -> Stream {
    let is_gzipped = req
        .headers()
        .get(CONTENT_ENCODING)
        .map(|value| value.as_bytes())
        .map_or(false, |enc| enc.eq_ignore_ascii_case(b"gzip"));
    let body = req.into_body().into_data_stream();

    if is_gzipped {
        let reader = StreamReader::new(body.map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))));
        let decoder = GzipDecoder::new(reader);
        let decompressed = tokio_util::io::ReaderStream::new(decoder)
            .map(|result| result.map(Bytes::from).map_err(|e| anyhow::anyhow!("Decompression error: {}", e)));
        Box::new(decompressed)
    } else {
        Box::new(body.map(|result| result.map_err(|e| anyhow::anyhow!("Body error: {}", e)).map(Bytes::from)))
    }
}

impl From<HttpAbsorber> for AbsorberInner {
    fn from(value: HttpAbsorber) -> Self {
        AbsorberInner::Http(value)
    }
}
