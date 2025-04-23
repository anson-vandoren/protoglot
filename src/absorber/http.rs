use http_body_util::BodyExt;
use hyper::{server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use log::{error, info};
use tokio::net::TcpListener;
use tokio_stream::{wrappers::TcpListenerStream, StreamExt};

use super::{extract_message, validate_message, AbsorberInner, StatsSvc};
use crate::config::MessageType;

pub struct HttpAbsorber {
    address: String,
    port: u16,
    message_type: MessageType,
}

impl HttpAbsorber {
    pub async fn build(address: &str, port: u16, message_type: MessageType) -> Self {
        Self {
            message_type,
            address: address.to_string(),
            port,
        }
    }

    pub(super) async fn run(self, stats: StatsSvc) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port))
            .await
            .expect("Could not bind to TCP address & port");

        let mut listener = TcpListenerStream::new(listener);
        while let Some(s) = listener.next().await {
            match s {
                Ok(s) => {
                    let remote_addr = s.peer_addr().unwrap();
                    info!("Accepted new connection from {}", remote_addr);
                    let message_type = self.message_type.clone();

                    let io = TokioIo::new(s);
                    let stats = stats.clone();
                    tokio::spawn(async move {
                        let message_type = message_type.clone();
                        let service = service_fn(|req| handle_request(req, stats.clone(), message_type.clone()));
                        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                            // TODO: auto if HTTP2 isn't explicitly requested?
                            //if let Err(err) = http2::Builder::new(TokioExecutor::new()).serve_connection(io, service).await {
                            error!("Error while receiving HTTP stream from {remote_addr}: {:?}", err);
                        }
                        info!("Connection closed: {remote_addr}");
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
    let mut body = req.into_data_stream();
    let mut events = 0;
    let mut bytes = 0;
    while let Some(msg) = body.next().await {
        let mut msg = msg.unwrap().to_vec();
        while let Some(message) = extract_message(&mut msg) {
            validate_message(&message, &message_type);
            events += 1;
            bytes += message.len();
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
