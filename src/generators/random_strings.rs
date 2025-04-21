use std::sync::Arc;

use rand::Rng as _;

pub struct RandomStringGenerator {
    messages: Arc<Vec<String>>,
    hostnames: Arc<Vec<String>>,
    appnames: Arc<Vec<String>>,
}

impl RandomStringGenerator {
    pub fn new() -> Self {
        let content = include_str!("../../config/reticulating_splines.txt");
        let messages: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        // use "default message" if the file is empty
        let messages = if messages.is_empty() {
            vec!["default message".to_string()]
        } else {
            messages
        };
        let content = include_str!("../../config/hostnames.txt");
        let hostnames: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let hostnames = if hostnames.is_empty() {
            vec!["localhost".to_string()]
        } else {
            hostnames
        };
        let content = include_str!("../../config/appnames.txt");
        let appnames: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let appnames = if appnames.is_empty() { vec!["myapp".to_string()] } else { appnames };
        Self {
            messages: Arc::new(messages),
            hostnames: Arc::new(hostnames),
            appnames: Arc::new(appnames),
        }
    }

    pub fn generate_message(&self) -> String {
        let mut rng = rand::rng();
        self.messages[rng.random_range(0..self.messages.len())].clone()
    }

    pub fn generate_hostname(&self) -> String {
        let mut rng = rand::rng();
        self.hostnames[rng.random_range(0..self.hostnames.len())].clone()
    }

    pub fn generate_appname(&self) -> String {
        let mut rng = rand::rng();
        self.appnames[rng.random_range(0..self.appnames.len())].clone()
    }
}
