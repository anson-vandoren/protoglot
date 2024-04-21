use super::Event;

pub trait EventGenerator {
  fn generate(&self) -> Event;
}

pub struct Syslog3164EventGenerator;

impl EventGenerator for Syslog3164EventGenerator {
  fn generate(&self) -> Event {
    Event {
      message: "Reticulating splines".to_string(), // TODO: generate random message
      // facility is random number between 0 and 23
      facility: Some(rand::random::<u8>() % 24),
      // severity is random number between 0 and 7
      severity: Some(rand::random::<u8>() % 8),
      hostname: Some("localhost".to_string()), // TODO: generate random hostname
      application_name: Some("myapp".to_string()), // TODO: generate random application name
      process_id: Some(rand::random::<u32>() / 2),
      ..Default::default()
    }
  }
}