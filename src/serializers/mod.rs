use crate::generators::Event;
pub mod ndjson;

pub trait Serializer {
    fn serialize(&self, event: &Event) -> Vec<u8>;
}
