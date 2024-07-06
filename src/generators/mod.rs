mod nd_json;
mod random_strings;
mod syslog3164;
mod syslog5424;

pub use nd_json::NdJsonEventGenerator;
pub use random_strings::RandomStringGenerator;
pub use syslog3164::Syslog3164EventGenerator;
pub use syslog5424::Syslog5424EventGenerator;

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
