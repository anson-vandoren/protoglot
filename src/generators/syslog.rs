use std::sync::Arc;

use super::{event::EventGenerator, Event, MessageGenerator};
use chrono;
use rand::Rng;

pub struct Syslog3164 {
    pub timestamp: u64,
    pub message: String,
    pub facility: u8,
    pub severity: u8,
    pub app_name: Option<String>,
    pub pid: Option<u32>,
    pub hostname: Option<String>,
}

impl Event for Syslog3164 {
    fn serialize(&self) -> Vec<u8> {
        // Return a valid RFC 3164 syslog message
        let time_string = chrono::DateTime::from_timestamp(self.timestamp as i64, 0)
            .expect("Missing timestamp")
            .format("%b %e %T")
            .to_string();
        let binding = "localhost".to_string();
        let hostname = self.hostname.as_ref().unwrap_or(&binding);
        let binding = "myapp".to_string();
        let app_name = self.app_name.as_ref().unwrap_or(&binding);
        let pid = self.pid.unwrap_or(0);
        format!(
            "<{}>{} {} {}[{}]: {}\n",
            self.facility * 8 + self.severity,
            time_string,
            hostname,
            app_name,
            pid,
            self.message
        )
        .into_bytes()
    }
}

pub struct Syslog3164EventGenerator {
    pub message_generator: Arc<MessageGenerator>,
}

impl EventGenerator for Syslog3164EventGenerator {
    fn generate(&self) -> Box<dyn Event + Send> {
        let mut rng = rand::thread_rng();
        Box::new(Syslog3164 {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
            message: self.message_generator.generate().expect("No message"),
            facility: rng.gen_range(0..24),
            severity: rng.gen_range(0..8),
            app_name: Some("myapp".to_string()), // TODO: generate random application name
            pid: Some(rng.gen_range(0..std::u32::MAX)),
            hostname: Some("localhost".to_string()), // TODO: generate random hostname
        })
    }
}
