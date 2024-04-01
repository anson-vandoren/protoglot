use serde_json::Value;
use std::{collections::HashMap, time::SystemTime};

pub struct Event {
    pub timestamp: u64,
    pub message: String,
    pub index: Option<String>,
    pub source: Option<String>,
    pub sourcetype: Option<String>,
    pub hostname: Option<String>,
    pub pri: Option<u8>,
    pub severity: Option<u8>,
    pub application_name: Option<String>,
    pub process_id: Option<u32>,
    pub message_id: Option<String>,
    pub fields: HashMap<String, Value>,
}

impl Event {
    pub fn new(timestamp: u64, message: String) -> Self {
        Event {
            timestamp,
            message,
            index: None,
            source: None,
            sourcetype: None,
            hostname: None,
            pri: None,
            severity: None,
            application_name: None,
            process_id: None,
            message_id: None,
            fields: HashMap::new(),
        }
    }
}

impl Default for Event {
    fn default() -> Self {
        Event {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
            message: "This is a fucking message".to_string(),
            index: None,
            source: None,
            sourcetype: None,
            hostname: None,
            pri: None,
            severity: None,
            application_name: None,
            process_id: None,
            message_id: None,
            fields: HashMap::new(),
        }
    }
}
