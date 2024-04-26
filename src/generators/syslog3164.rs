use std::sync::Arc;

use super::{Event, EventGenerator, RandomStringGenerator};

use chrono;
use rand::Rng;
use uuid::Uuid;

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
        let mut rng = rand::thread_rng();
        let message = self.message_generator.generate_message();
        // prepend incrementing index and a uuidv4
        let message = format!(
            "idx={}, uuid={}, msg={}",
            self.message_index,
            Uuid::new_v4(),
            message
        );
        self.message_index += 1;
        Box::new(Syslog3164 {
            timestamp: chrono::Utc::now(),
            message,
            facility: rng.gen_range(0..24),
            severity: rng.gen_range(0..8),
            app_name: self.message_generator.generate_appname(),
            pid: rng.gen_range(0..std::u16::MAX),
            hostname: self.message_generator.generate_hostname(),
        })
    }
}
