use serde_json::Value;

use super::Serializer;
use crate::generators::Event;
pub struct NdJsonSerializer;

impl Serializer for NdJsonSerializer {
    fn serialize(&self, event: &Event) -> Vec<u8> {
        let mut json_map = serde_json::Map::new();

        // Mandatory fields
        json_map.insert("_time".to_string(), Value::from(event.timestamp));
        json_map.insert(
            "message".to_string(),
            Value::from(event.message.to_string()),
        );

        // Optional fields
        if let Some(ref index) = event.index {
            json_map.insert("index".to_string(), Value::from(index.to_string()));
        }
        if let Some(ref source) = event.source {
            json_map.insert("source".to_string(), Value::from(source.to_string()));
        }
        if let Some(ref sourcetype) = event.sourcetype {
            json_map.insert(
                "sourcetype".to_string(),
                Value::from(sourcetype.to_string()),
            );
        }
        if let Some(ref hostname) = event.hostname {
            json_map.insert("hostname".to_string(), Value::from(hostname.to_string()));
        }
        if let Some(ref facility) = event.facility {
            json_map.insert("pri".to_string(), Value::from(*facility));
        }
        if let Some(ref severity) = event.severity {
            json_map.insert("severity".to_string(), Value::from(*severity));
        }
        if let Some(ref application_name) = event.application_name {
            json_map.insert(
                "application_name".to_string(),
                Value::from(application_name.to_string()),
            );
        }
        if let Some(ref process_id) = event.process_id {
            json_map.insert("process_id".to_string(), Value::from(*process_id));
        }
        if let Some(ref message_id) = event.message_id {
            json_map.insert(
                "message_id".to_string(),
                Value::from(message_id.to_string()),
            );
        }
        for (key, value) in event.fields.iter() {
            json_map.insert(key.to_string(), value.clone());
        }

        let mut serialized = Value::Object(json_map).to_string().into_bytes();
        serialized.push(b'\n');
        serialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize_mandatory_fields() {
        let event = Event {
            timestamp: 1623456789,
            message: "Test message".to_string(),
            index: None,
            source: None,
            sourcetype: None,
            hostname: None,
            facility: None,
            severity: None,
            application_name: None,
            process_id: None,
            message_id: None,
            fields: Default::default(),
        };

        let serializer = NdJsonSerializer;
        let serialized = serializer.serialize(&event);

        let expected = json!({
            "_time": 1623456789,
            "message": "Test message"
        });
        let expected_serialized = format!("{}\n", expected.to_string());

        assert_eq!(String::from_utf8(serialized).unwrap(), expected_serialized);
    }

    #[test]
    fn test_serialize_optional_fields() {
        let event = Event {
            timestamp: 1623456789,
            message: "Test message".to_string(),
            index: Some("test_index".to_string()),
            source: Some("test_source".to_string()),
            sourcetype: Some("test_sourcetype".to_string()),
            hostname: Some("test_hostname".to_string()),
            facility: Some(1),
            severity: Some(2),
            application_name: Some("test_app".to_string()),
            process_id: Some(123),
            message_id: Some("test_message_id".to_string()),
            fields: Default::default(),
        };

        let serializer = NdJsonSerializer;
        let serialized = serializer.serialize(&event);

        let expected = json!({
            "_time": 1623456789,
            "message": "Test message",
            "index": "test_index",
            "source": "test_source",
            "sourcetype": "test_sourcetype",
            "hostname": "test_hostname",
            "pri": 1,
            "severity": 2,
            "application_name": "test_app",
            "process_id": 123,
            "message_id": "test_message_id"
        });
        let expected_serialized = format!("{}\n", expected.to_string());

        assert_eq!(String::from_utf8(serialized).unwrap(), expected_serialized);
    }

    #[test]
    fn test_serialize_custom_fields() {
        let mut event = Event {
            timestamp: 1623456789,
            message: "Test message".to_string(),
            index: None,
            source: None,
            sourcetype: None,
            hostname: None,
            facility: None,
            severity: None,
            application_name: None,
            process_id: None,
            message_id: None,
            fields: Default::default(),
        };

        event
            .fields
            .insert("custom_field1".to_string(), json!("custom_value1"));
        event.fields.insert("custom_field2".to_string(), json!(42));

        let serializer = NdJsonSerializer;
        let serialized = serializer.serialize(&event);

        let expected = json!({
            "_time": 1623456789,
            "message": "Test message",
            "custom_field1": "custom_value1",
            "custom_field2": 42
        });
        let expected_serialized = format!("{}\n", expected.to_string());

        assert_eq!(String::from_utf8(serialized).unwrap(), expected_serialized);
    }
}
