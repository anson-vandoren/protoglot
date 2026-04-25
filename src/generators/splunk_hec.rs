use std::io::Write as _;

use super::EventGenerator;

pub struct SplunkHecEventGenerator {
    message_index: u64,
}

impl SplunkHecEventGenerator {
    pub fn new() -> Self {
        Self { message_index: 0 }
    }
}

impl EventGenerator for SplunkHecEventGenerator {
    fn generate_into(&mut self, buf: &mut Vec<u8>) {
        let idx = self.message_index;
        self.message_index += 1;

        let host = HOSTS[idx as usize % HOSTS.len()];
        let source = SOURCES[idx as usize % SOURCES.len()];
        let sourcetype = SOURCETYPES[idx as usize % SOURCETYPES.len()];
        let time = 1_714_243_969.0 + (idx as f64 / 1000.0);

        let event = match idx % 5 {
            0 => format!(
                r#"{{"kind":"access","request_id":"req-{idx:08}","method":"GET","path":"/api/widgets/{idx}","status":200,"bytes":{}}}"#,
                512 + (idx % 4096)
            ),
            1 => format!(r#""plain text HEC event idx={idx} action=login result=success""#),
            2 => format!(
                r#"{{"kind":"metric","metric_name":"pipeline.events","value":{},"dimensions":{{"region":"us-east-1","tier":"ingest"}}}}"#,
                1000 + idx
            ),
            3 => format!(
                r#"{{"kind":"audit","actor":"user{}","operation":"token.create","success":{},"attempt":{}}}"#,
                idx % 17,
                idx.is_multiple_of(2),
                idx
            ),
            _ => format!(
                r#"{{"kind":"nested","trace":{{"id":"trace-{idx:08}","span":"span-{}"}},"tags":["hec","protoglot","xenomux"]}}"#,
                idx % 128
            ),
        };

        let _ = writeln!(
            buf,
            r#"{{"time":{time:.3},"host":"{host}","source":"{source}","sourcetype":"{sourcetype}","index":"main","fields":{{"generator":"protoglot","sequence":{idx},"variant":{}}},"event":{event}}}"#,
            idx % 5
        );
    }
}

const HOSTS: &[&str] = &["protoglot-01", "protoglot-02", "xenomux-hec-test"];
const SOURCES: &[&str] = &["protoglot://hec/access", "protoglot://hec/audit", "protoglot://hec/metrics"];
const SOURCETYPES: &[&str] = &["protoglot:json", "protoglot:text", "protoglot:metric"];

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_matches};

    use super::*;

    #[test]
    fn emits_valid_hec_envelopes_with_required_fields() {
        let mut generator = SplunkHecEventGenerator::new();
        let mut buf = Vec::new();

        for _ in 0..10 {
            buf.clear();
            generator.generate_into(&mut buf);

            let value: serde_json::Value = serde_json::from_slice(&buf).unwrap();
            assert!(value.get("time").is_some());
            assert!(value.get("host").is_some());
            assert!(value.get("source").is_some());
            assert!(value.get("sourcetype").is_some());
            assert!(value.get("fields").is_some());
            assert!(value.get("event").is_some());
        }
    }

    #[test]
    fn varies_event_shapes() {
        let mut generator = SplunkHecEventGenerator::new();
        let mut buf = Vec::new();
        let mut kinds = Vec::new();

        for _ in 0..5 {
            buf.clear();
            generator.generate_into(&mut buf);
            let value: serde_json::Value = serde_json::from_slice(&buf).unwrap();
            kinds.push(value["event"].clone());
        }

        assert_matches!(kinds[0], serde_json::Value::Object(_));
        assert_matches!(kinds[1], serde_json::Value::String(_));
        assert_eq!(kinds.len(), 5);
    }
}
