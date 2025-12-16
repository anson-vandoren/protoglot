mod nd_json;
mod syslog3164;
mod syslog5424;

pub use nd_json::NdJsonEventGenerator;
pub use syslog3164::Syslog3164EventGenerator;
pub use syslog5424::Syslog5424EventGenerator;

use crate::config::MessageType;

pub enum EventType {
    Syslog3164(Syslog3164EventGenerator),
    Syslog5424(Syslog5424EventGenerator),
    NdJson(NdJsonEventGenerator),
}

impl EventGenerator for EventType {
    fn generate_bytes(&mut self) -> Vec<u8> {
        match self {
            EventType::Syslog3164(generator) => generator.generate_bytes(),
            EventType::Syslog5424(generator) => generator.generate_bytes(),
            EventType::NdJson(generator) => generator.generate_bytes(),
        }
    }
}
pub trait EventGenerator {
    fn generate_bytes(&mut self) -> Vec<u8>;
}

pub fn create_generator(message_type: &MessageType) -> EventType {
    match message_type {
        MessageType::Syslog3164 => EventType::Syslog3164(Syslog3164EventGenerator::new()),
        MessageType::Syslog5424 => {
            EventType::Syslog5424(Syslog5424EventGenerator::new(false))
        }
        MessageType::Syslog5424Octet => {
            EventType::Syslog5424(Syslog5424EventGenerator::new(true))
        }
        MessageType::NdJson => EventType::NdJson(NdJsonEventGenerator::new()),
    }
}
