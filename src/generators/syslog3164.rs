use std::sync::Arc;

use rand::Rng as _;
use uuid::Uuid;

use super::{Event, EventGenerator, RandomStringGenerator};

pub struct Syslog3164 {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub facility: u8,
    pub severity: u8,
    pub app_name: String,
    pub pid: u16,
    pub hostname: String,
}

impl Event for Syslog3164 {
    fn serialize(&self) -> Vec<u8> {
        // Return a valid RFC 3164 syslog message
        let time_string = self.timestamp.format("%b %e %T").to_string();

        format!(
            "<{}>{} {} {}[{}]: {}\n",
            self.facility * 8 + self.severity,
            time_string,
            self.hostname,
            self.app_name,
            self.pid,
            self.message
        )
        .into_bytes()
    }
}

pub struct Syslog3164EventGenerator {
    pub message_generator: Arc<RandomStringGenerator>,
    message_index: u64,
}

impl Syslog3164EventGenerator {
    pub fn new(message_generator: Arc<RandomStringGenerator>) -> Self {
        Self {
            message_generator,
            message_index: 0,
        }
    }
}

impl EventGenerator for Syslog3164EventGenerator {
    fn generate(&mut self) -> Box<dyn Event + Send> {
        let mut rng = rand::rng();
        let message = self.message_generator.generate_message();
        // prepend incrementing index and a uuidv4
        let message = format!("idx={}, uuid={}, msg={}", self.message_index, Uuid::new_v4(), message);
        self.message_index += 1;
        Box::new(Syslog3164 {
            timestamp: chrono::Utc::now(),
            message,
            facility: rng.random_range(0..24),
            severity: rng.random_range(0..8),
            app_name: self.message_generator.generate_appname(),
            pid: rng.random_range(0..u16::MAX),
            hostname: self.message_generator.generate_hostname(),
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_serialization() {
        let event = Syslog3164 {
            timestamp: chrono::Utc.with_ymd_and_hms(20024, 7, 8, 9, 10, 11).unwrap(),
            message: "test message".to_string(),
            facility: 1,
            severity: 5,
            app_name: "test_app".to_string(),
            pid: 1234,
            hostname: "test_host".to_string(),
        };
        let serialized = String::from_utf8(event.serialize()).unwrap();
        assert_eq!(serialized, "<13>Jul  8 09:10:11 test_host test_app[1234]: test message\n");
    }
}
