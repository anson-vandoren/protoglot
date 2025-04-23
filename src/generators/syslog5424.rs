use std::sync::Arc;

use rand::Rng;
use uuid::Uuid;

use super::{Event, EventGenerator, RandomStringGenerator};

pub struct Syslog5424 {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub facility: u8,
    pub severity: u8,
    pub app_name: String,
    pub pid: u16,
    pub hostname: String,
}

impl Event for Syslog5424 {
    fn serialize(&self) -> Vec<u8> {
        // Return a valid RFC 5424 syslog message
        let time_string = self.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        format!(
            "<{}>1 {} {} {} {} {} {} {}\n",
            self.facility * 8 + self.severity,
            time_string,
            self.hostname,
            self.app_name,
            self.pid,
            "-", // no message id
            "-", // no structured data
            self.message
        )
        .into_bytes()
    }
}

pub struct Syslog5424EventGenerator {
    pub message_generator: Arc<RandomStringGenerator>,
    message_index: u64,
}

impl Syslog5424EventGenerator {
    pub fn new(message_generator: Arc<RandomStringGenerator>) -> Self {
        Self {
            message_generator,
            message_index: 0,
        }
    }
}

impl EventGenerator for Syslog5424EventGenerator {
    fn generate(&mut self) -> Box<dyn Event + Send> {
        let mut rng = rand::rng();
        let message = self.message_generator.generate_message();
        // prepend incrementing index and a uuidv4
        let message = format!("idx={}, uuid={}, msg={}", self.message_index, Uuid::new_v4(), message);
        self.message_index += 1;
        Box::new(Syslog5424 {
            timestamp: chrono::Utc::now(),
            message,
            facility: 1, // user-level messages
            severity: 6, // informational severity
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
        let event = Syslog5424 {
            timestamp: chrono::Utc.with_ymd_and_hms(2024, 7, 8, 9, 10, 11).unwrap(),
            message: "Hello, world!".to_string(),
            facility: 1,
            severity: 6,
            app_name: "test".to_string(),
            pid: 1234,
            hostname: "localhost".to_string(),
        };

        let serialized = String::from_utf8(event.serialize()).unwrap();
        assert_eq!(serialized, "<14>1 2024-07-08T09:10:11.000Z localhost test 1234 - - Hello, world!\n");
    }

    use proptest::prelude::*;
    proptest! {
        #[test]
        fn test_syslog5424_serialization(
            facility in 0u8..24u8,
            severity in 0u8..8u8,
            message in "\\PC*",
            hostname in "[a-zA-Z0-9\\.-]{1,255}",
            app_name in "[a-zA-Z0-9\\.-]{1,48}",
        ) {
            let event = Syslog5424 {
                timestamp: chrono::Utc::now(),
                message: message.clone(),
                facility,
                severity,
                app_name: app_name.clone(),
                pid: 1234,
                hostname: hostname.clone(),
            };

            let serialized = event.serialize();
            let serialized_str = String::from_utf8(serialized).expect("invalid utf-8");
            assert!(serialized_str.starts_with(&format!("<{}>1 ", facility * 8 + severity)),
                "Serialized syslog should start with correct PRI value");
            assert!(serialized_str.contains(&hostname), "Serialized syslog should contain hostname");
            assert!(serialized_str.contains(&app_name), "Serialized syslog should contain app_name");
            assert!(serialized_str.contains(&message), "Serialized syslog should contain message");
        }
    }
}
