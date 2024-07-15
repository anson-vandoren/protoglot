use crate::config::{AbsorberConfig, ListenAddress, MessageType, Protocol};
use std::sync::Arc;
use tokio::{io::AsyncReadExt, sync::Mutex};

pub struct Absorber {
    config: Arc<AbsorberConfig>,
    stats: Arc<Mutex<AbsorberStats>>,
}

struct AbsorberStats {
    total_events: u64,
    total_bytes: u64,
    start_time: std::time::Instant,
}

impl Absorber {
    pub fn new(config: AbsorberConfig) -> Self {
        Self {
            config: Arc::new(config),
            stats: Arc::new(Mutex::new(AbsorberStats {
                total_events: 0,
                total_bytes: 0,
                start_time: std::time::Instant::now(),
            })),
        }
    }

    pub async fn run(&self) -> tokio::io::Result<()> {
        let mut handles = vec![];

        for address in &self.config.listen_addresses {
            let absorber = self.clone();
            let address = address.clone();
            let handle =
                tokio::spawn(
                    async move { absorber.listen(address, Arc::clone(&absorber.stats)).await },
                );
            handles.push(handle);
        }

        let update_interval = self.config.update_interval;
        let stats_handle = tokio::spawn(Absorber::update_stats(
            Arc::clone(&self.stats),
            update_interval,
        ));
        handles.push(stats_handle);

        let input_handle = tokio::spawn(Absorber::handle_user_input(Arc::clone(&self.stats)));
        handles.push(input_handle);

        for handle in handles {
            handle.await??;
        }

        Ok(())
    }

    async fn listen(
        &self,
        address: ListenAddress,
        stats: Arc<Mutex<AbsorberStats>>,
    ) -> tokio::io::Result<()> {
        match address.protocol {
            Protocol::Tcp => self.listen_tcp(address, stats).await,
            Protocol::Udp => self.listen_udp(address, stats).await,
        }
    }

    async fn listen_tcp(
        &self,
        address: ListenAddress,
        stats: Arc<Mutex<AbsorberStats>>,
    ) -> tokio::io::Result<()> {
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", address.host, address.port)).await?;
        let absorber = Arc::new(self.clone());
        loop {
            let (socket, _) = listener.accept().await?;
            let stats = Arc::clone(&stats);
            let absorber = Arc::clone(&absorber);
            tokio::spawn(async move {
                if let Err(e) = absorber.handle_tcp_connection(socket, stats).await {
                    eprintln!("Error handling TCP connection: {}", e);
                }
            });
        }
    }

    async fn listen_udp(
        &self,
        address: ListenAddress,
        stats: Arc<Mutex<AbsorberStats>>,
    ) -> tokio::io::Result<()> {
        let socket =
            tokio::net::UdpSocket::bind(format!("{}:{}", address.host, address.port)).await?;
        let mut buf = [0; 65535];
        loop {
            let (len, _) = socket.recv_from(&mut buf).await?;
            let message = &buf[..len];
            self.process_message(message, stats.clone()).await;
        }
    }

    async fn handle_tcp_connection(
        &self,
        mut socket: tokio::net::TcpStream,
        stats: Arc<Mutex<AbsorberStats>>,
    ) -> tokio::io::Result<()> {
        let mut buf = Vec::new();
        loop {
            match socket.read_buf(&mut buf).await {
                Ok(0) => break, // connection closed
                Ok(_) => {
                    if let Some(message) = self.extract_message(&mut buf) {
                        self.process_message(&message, stats.clone()).await;
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

    fn extract_message(&self, buf: &mut Vec<u8>) -> Option<Vec<u8>> {
        // Implement message extraction logic based on the message type
        // For now, assume newline-delimited messages
        if let Some(pos) = buf.iter().position(|&x| x == b'\n') {
            let message = buf.drain(..=pos).collect();
            Some(message)
        } else {
            None
        }
    }

    async fn process_message(&self, message: &[u8], stats: Arc<Mutex<AbsorberStats>>) {
        // Validate and process the message
        if self.validate_message(message) {
            let mut stats = stats.lock().await;
            stats.total_events += 1;
            stats.total_bytes += message.len() as u64;
        }
    }

    fn validate_message(&self, message: &[u8]) -> bool {
        match self.config.message_type {
            MessageType::Syslog3164 => self.validate_syslog3164(message),
            MessageType::Syslog5424 => self.validate_syslog5424(message),
            MessageType::NdJson => self.validate_ndjson(message),
        }
    }

    fn validate_syslog3164(&self, message: &[u8]) -> bool {
        // TODO: Implement full Syslog 3164 message validation
        message.starts_with(b"<") && message.contains(&b'>')
    }

    fn validate_syslog5424(&self, message: &[u8]) -> bool {
        // TODO: Implement full Syslog 5424 message validation
        let s = String::from_utf8_lossy(message);
        s.starts_with("<") && s.contains(">1 ") && s.split_whitespace().count() >= 7
    }

    fn validate_ndjson(&self, message: &[u8]) -> bool {
        serde_json::from_slice::<serde_json::Value>(message).is_ok()
    }

    async fn update_stats(
        stats: Arc<Mutex<AbsorberStats>>,
        update_interval: u64,
    ) -> Result<(), tokio::io::Error> {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(update_interval)).await;
            let stats = stats.lock().await;
            let elapsed = stats.start_time.elapsed().as_secs_f64();
            let events_per_sec = stats.total_events as f64 / elapsed;
            let bytes_per_sec = stats.total_bytes as f64 / elapsed;
            println!(
                "Events: {}, Bytes: {}, EPS: {:.2}, BPS: {:.2}",
                stats.total_events, stats.total_bytes, events_per_sec, bytes_per_sec
            );
        }
    }

    async fn handle_user_input(stats: Arc<Mutex<AbsorberStats>>) -> Result<(), tokio::io::Error> {
        let mut input = String::new();
        loop {
            if let Ok(_) = std::io::stdin().read_line(&mut input) {
                match input.trim() {
                    "rs" => {
                        let mut stats = stats.lock().await;
                        *stats = AbsorberStats {
                            total_events: 0,
                            total_bytes: 0,
                            start_time: std::time::Instant::now(),
                        };
                        println!("Stats reset")
                    }
                    "q" => {
                        println!("Exiting...");
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
            input.clear();
        }
    }
}

impl Clone for Absorber {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            stats: Arc::clone(&self.stats),
        }
    }
}
