mod nd_json;
mod random_strings;
mod syslog3164;
mod syslog5424;

use std::sync::Arc;

pub use nd_json::NdJsonEventGenerator;
pub use random_strings::RandomStringGenerator;
pub use syslog3164::Syslog3164EventGenerator;
pub use syslog5424::Syslog5424EventGenerator;

use crate::config::MessageType;

pub trait Event {
    fn serialize(&self) -> Vec<u8>;
}

pub enum EventType {
    Syslog3164(Syslog3164EventGenerator),
    Syslog5424(Syslog5424EventGenerator),
    NdJson(NdJsonEventGenerator),
}

impl EventGenerator for EventType {
    fn generate(&mut self) -> Box<dyn Event + Send> {
        match self {
            EventType::Syslog3164(generator) => generator.generate(),
            EventType::Syslog5424(generator) => generator.generate(),
            EventType::NdJson(generator) => generator.generate(),
        }
    }
}
pub trait EventGenerator {
    fn generate(&mut self) -> Box<dyn Event + Send>;
}

pub fn create_generator(message_type: &MessageType, message_generator: Arc<random_strings::RandomStringGenerator>) -> EventType {
    match message_type {
        MessageType::Syslog3164 => EventType::Syslog3164(Syslog3164EventGenerator::new(message_generator)),
        MessageType::Syslog5424 => EventType::Syslog5424(Syslog5424EventGenerator::new(message_generator)),
        MessageType::NdJson => EventType::NdJson(NdJsonEventGenerator::new(message_generator)),
    }
}
