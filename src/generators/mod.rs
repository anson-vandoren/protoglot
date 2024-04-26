mod event;
mod syslog;

pub use event::{Event, EventGenerator, RandomStringGenerator};
pub use syslog::Syslog3164EventGenerator;
