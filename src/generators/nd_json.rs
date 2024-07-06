use std::sync::Arc;

use super::{Event, EventGenerator, RandomStringGenerator};
use rand::Rng;
use uuid::Uuid;

pub struct NdJson {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub hostname: String,
    pub pid: u16,
    pub app_name: String,
}

impl Event for NdJson {
    fn serialize(&self) -> Vec<u8> {
        // Return a valid NDJSON message
        let time_string = self.timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        format!(
      "{{\"timestamp\":\"{}\",\"hostname\":\"{}\",\"app_name\":\"{}\",\"pid\":{},\"message\":\"{}\"}}\n",
      time_string,
      self.hostname,
      self.app_name,
      self.pid,
      self.message,
    )
    .into_bytes()
    }
}

pub struct NdJsonEventGenerator {
    pub message_generator: Arc<RandomStringGenerator>,
    message_index: u64,
}

impl NdJsonEventGenerator {
    pub fn new(message_generator: Arc<RandomStringGenerator>) -> Self {
        Self {
            message_generator,
            message_index: 0,
        }
    }
}

impl EventGenerator for NdJsonEventGenerator {
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

        Box::new(NdJson {
            timestamp: chrono::Utc::now(),
            message,
            hostname: "example.com".to_string(),
            app_name: "example".to_string(),
            pid: rng.gen_range(1000..9999),
        })
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_ndjson_serialize() {
        let event = NdJson {
            timestamp: chrono::Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap(),
            message: "Hello, world!".to_string(),
            hostname: "example.com".to_string(),
            app_name: "example".to_string(),
            pid: 1234,
        };

        let serialized = event.serialize();
        let expected = "{\"timestamp\":\"2021-01-01T00:00:00.000Z\",\"hostname\":\"example.com\",\"app_name\":\"example\",\"pid\":1234,\"message\":\"Hello, world!\"}\n".as_bytes();

        assert_eq!(serialized, expected);
    }
}
