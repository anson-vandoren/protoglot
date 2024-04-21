use rand::prelude::*;
use std::sync::Arc;
use std::{io, fs};

pub trait Event {
    fn serialize(&self) -> Vec<u8>;
}

pub trait EventGenerator {
    fn generate(&self) -> Box<dyn Event + Send>;
}

pub struct MessageGenerator {
    lines: Arc<Vec<String>>,
}

impl MessageGenerator {
    pub fn new(file_path: &str) -> io::Result<Self> {
        let lines = fs::read_to_string(file_path)?
            .lines()
            .map(|s| s.to_string())
            .collect();
        Ok(Self {
            lines: Arc::new(lines),
        })
    }

    pub fn generate(&self) -> Option<String> {
        if self.lines.is_empty() {
            None
        } else {
            let mut rng = thread_rng();
            Some(self.lines[rng.gen_range(0..self.lines.len())].clone())
        }
    }
}