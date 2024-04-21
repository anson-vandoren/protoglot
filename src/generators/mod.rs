mod event;
mod syslog;

pub use event::{Event, EventGenerator, MessageGenerator};
pub use syslog::Syslog3164EventGenerator;