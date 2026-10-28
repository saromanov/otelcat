//! OTLP/JSON model.
//!
//! Emission follows the standard protobuf-JSON mapping used by the OTLP spec
//! and its official examples: camelCase keys, 64-bit integers as decimal
//! strings, hex-encoded trace/span IDs, base64 bytes. Parsing additionally
//! accepts snake_case aliases for every key (the spec requires parsers to
//! accept the original proto field names).

use serde::de::Error as _;

pub mod common;
pub mod logs;
pub mod metrics;
pub mod trace;
pub mod wire;

pub use common::{AnyValue, ArrayValue, InstrumentationScope, KeyValue, KvlistValue, Resource};
pub use logs::{ExportLogsServiceRequest, LogRecord, ResourceLogs, ScopeLogs};
pub use metrics::{
    Buckets, Exemplar, ExponentialHistogram, ExponentialHistogramDataPoint,
    ExportMetricsServiceRequest, Gauge, Histogram, HistogramDataPoint, Metric, MetricData,
    NumberDataPoint, PointValue, QuantileValue, ResourceMetrics, ScopeMetrics, Summary,
    SummaryDataPoint,
};
pub use trace::{Event, ExportTraceServiceRequest, Link, ResourceSpans, ScopeSpans, Span, Status};

/// A parsed OTLP export request of any signal type.
#[derive(Clone, Debug, PartialEq)]
pub enum Signal {
    Traces(ExportTraceServiceRequest),
    Logs(ExportLogsServiceRequest),
    Metrics(ExportMetricsServiceRequest),
}

impl Signal {
    /// Parse a JSONL line by looking at its top-level key.
    pub fn from_json_value(v: serde_json::Value) -> Result<Signal, serde_json::Error> {
        match top_key(&v) {
            Some(("resourceSpans", "resource_spans")) => {
                Ok(Signal::Traces(serde_json::from_value(v)?))
            }
            Some(("resourceLogs", "resource_logs")) => Ok(Signal::Logs(serde_json::from_value(v)?)),
            Some(("resourceMetrics", "resource_metrics")) => {
                Ok(Signal::Metrics(serde_json::from_value(v)?))
            }
            _ => Err(serde_json::Error::custom(
                "expected one of resourceSpans/resourceLogs/resourceMetrics at the top level",
            )),
        }
    }

    /// Parse from a JSONL line.
    pub fn from_line(line: &str) -> Result<Signal, serde_json::Error> {
        let v: serde_json::Value = serde_json::from_str(line)?;
        Signal::from_json_value(v)
    }

    /// Serialize to a JSONL line (camelCase).
    pub fn to_line(&self) -> String {
        match self {
            Signal::Traces(s) => serde_json::to_string(s),
            Signal::Logs(s) => serde_json::to_string(s),
            Signal::Metrics(s) => serde_json::to_string(s),
        }
        .expect("OTLP/JSON serialization cannot fail")
    }

    /// Earliest timestamp in the signal, used by replay pacing.
    pub fn time_unix_nano(&self) -> Option<u64> {
        match self {
            Signal::Traces(s) => s
                .resource_spans
                .iter()
                .flat_map(|rs| rs.scope_spans.iter())
                .flat_map(|ss| ss.spans.iter())
                .map(|sp| sp.start_time_unix_nano)
                .filter(|t| *t > 0)
                .min(),
            Signal::Logs(s) => s
                .resource_logs
                .iter()
                .flat_map(|rl| rl.scope_logs.iter())
                .flat_map(|sl| sl.log_records.iter())
                .map(|lr| {
                    if lr.time_unix_nano > 0 {
                        lr.time_unix_nano
                    } else {
                        lr.observed_time_unix_nano
                    }
                })
                .filter(|t| *t > 0)
                .min(),
            Signal::Metrics(s) => s
                .resource_metrics
                .iter()
                .flat_map(|rm| rm.scope_metrics.iter())
                .flat_map(|sm| sm.metrics.iter())
                .filter_map(|m| m.data.as_ref())
                .filter_map(|data| match data {
                    MetricData::Gauge(g) => {
                        points_time(g.data_points.iter().map(|p| p.time_unix_nano))
                    }
                    MetricData::Sum(su) => {
                        points_time(su.data_points.iter().map(|p| p.time_unix_nano))
                    }
                    MetricData::Histogram(h) => {
                        points_time(h.data_points.iter().map(|p| p.time_unix_nano))
                    }
                    MetricData::ExponentialHistogram(e) => {
                        points_time(e.data_points.iter().map(|p| p.time_unix_nano))
                    }
                    MetricData::Summary(su) => {
                        points_time(su.data_points.iter().map(|p| p.time_unix_nano))
                    }
                })
                .min(),
        }
    }
}

fn points_time(iter: impl Iterator<Item = u64>) -> Option<u64> {
    iter.filter(|t| *t > 0).min()
}

fn top_key(v: &serde_json::Value) -> Option<(&str, &str)> {
    let obj = v.as_object()?;
    let keys = obj.keys().map(String::as_str).collect::<Vec<_>>();
    for (camel, snake) in [
        ("resourceSpans", "resource_spans"),
        ("resourceLogs", "resource_logs"),
        ("resourceMetrics", "resource_metrics"),
    ] {
        if keys.iter().any(|k| *k == camel || *k == snake) {
            return Some((camel, snake));
        }
    }
    None
}

pub(crate) fn is_zero<T: Default + PartialEq>(v: &T) -> bool {
    *v == T::default()
}

pub(crate) fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}

pub(crate) fn hex_encode(v: &[u8]) -> String {
    let mut out = String::with_capacity(v.len() * 2);
    for b in v {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

const HEX: &[u8; 16] = b"0123456789abcdef";

pub(crate) fn hex_decode(v: &str) -> Result<Vec<u8>, String> {
    let v = v.trim();
    if v.len() % 2 != 0 {
        return Err("odd length".to_string());
    }
    fn val(c: u8) -> Result<u8, String> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            _ => Err(format!("invalid hex character {}", c as char)),
        }
    }
    let bytes = v.as_bytes();
    (0..bytes.len())
        .step_by(2)
        .map(|i| Ok((val(bytes[i])? << 4) | val(bytes[i + 1])?))
        .collect()
}

pub(crate) fn b64_encode(v: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(v)
}

pub(crate) fn b64_decode(v: &str) -> Result<Vec<u8>, serde_json::Error> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(v.trim())
        .map_err(|e| serde_json::Error::custom(format!("invalid base64: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let v = vec![0xde, 0xad, 0xbe, 0xef];
        let s = hex_encode(&v);
        assert_eq!(s, "deadbeef");
        assert_eq!(hex_decode(&s).unwrap(), v);
        assert_eq!(hex_decode("DEADBEEF").unwrap(), v);
        assert!(hex_decode("xyz").is_err());
        assert!(hex_decode("abc").is_err());
    }

    #[test]
    fn signal_detect_camel_and_snake() {
        let t = Signal::from_line(r#"{"resourceSpans":[]}"#).unwrap();
        assert!(matches!(t, Signal::Traces(_)));
        let t = Signal::from_line(r#"{"resource_spans":[]}"#).unwrap();
        assert!(matches!(t, Signal::Traces(_)));
        let t = Signal::from_line(r#"{"resource_logs":[]}"#).unwrap();
        assert!(matches!(t, Signal::Logs(_)));
        let t = Signal::from_line(r#"{"resourceMetrics":[]}"#).unwrap();
        assert!(matches!(t, Signal::Metrics(_)));
        assert!(Signal::from_line("{}").is_err());
        assert!(Signal::from_line("not json").is_err());
    }

    #[test]
    fn trace_json_roundtrip_official_example() {
        // From https://github.com/open-telemetry/opentelemetry-proto examples/trace.json
        let raw = r#"{
            "resourceSpans": [{
                "resource": {"attributes": [{"key": "service.name", "value": {"stringValue": "my.service"}}]},
                "scopeSpans": [{
                    "scope": {"name": "my.library", "version": "1.0.0", "attributes": [
                        {"key": "my.scope.attribute", "value": {"stringValue": "some scope attribute"}}
                    ]},
                    "spans": [{
                        "traceId": "5B8EFFF798038103D269B633813FC60C",
                        "spanId": "EEE19B7EC3C1B174",
                        "parentSpanId": "EEE19B7EC3C1B173",
                        "name": "I'm a server span",
                        "startTimeUnixNano": "1544712660000000000",
                        "endTimeUnixNano": "1544712661000000000",
                        "kind": 2,
                        "attributes": [{"key": "my.span.attr", "value": {"intValue": 42}}],
                        "status": {"code": 2, "message": "boom"}
                    }]
                }]
            }]
        }"#;
        let sig = Signal::from_line(raw).unwrap();
        let line = sig.to_line();
        let back = Signal::from_line(&line).unwrap();
        assert_eq!(sig, back);
        let Signal::Traces(t) = &back else {
            panic!("wrong signal");
        };
        let span = &t.resource_spans[0].scope_spans[0].spans[0];
        assert_eq!(
            span.trace_id,
            vec![
                0x5b, 0x8e, 0xff, 0xf7, 0x98, 0x03, 0x81, 0x03, 0xd2, 0x69, 0xb6, 0x33, 0x81, 0x3f,
                0xc6, 0x0c
            ]
        );
        assert_eq!(span.start_time_unix_nano, 1544712660000000000);
        assert!(line.contains("resourceSpans"));
        assert!(line.contains("startTimeUnixNano"));
    }

    #[test]
    fn anyvalue_kinds() {
        let cases = [
            (r#"{"stringValue":"x"}"#, AnyValue::StringValue("x".into())),
            (r#"{"string_value":"x"}"#, AnyValue::StringValue("x".into())),
            (r#"{"boolValue":true}"#, AnyValue::BoolValue(true)),
            (r#"{"intValue":42}"#, AnyValue::IntValue(42)),
            (r#"{"intValue":"42"}"#, AnyValue::IntValue(42)),
            (r#"{"intValue":-7}"#, AnyValue::IntValue(-7)),
            (r#"{"doubleValue":1.5}"#, AnyValue::DoubleValue(1.5)),
            (
                r#"{"arrayValue":{"values":[{"intValue":1},{"stringValue":"a"}]}}"#,
                AnyValue::ArrayValue(ArrayValue {
                    values: vec![AnyValue::IntValue(1), AnyValue::StringValue("a".into())],
                }),
            ),
            (
                r#"{"kvlistValue":{"values":[{"key":"k","value":{"stringValue":"v"}}]}}"#,
                AnyValue::KvlistValue(KvlistValue {
                    values: vec![KeyValue {
                        key: "k".into(),
                        value: Some(AnyValue::StringValue("v".into())),
                    }],
                }),
            ),
        ];
        for (raw, want) in cases {
            let got: AnyValue = serde_json::from_str(raw).unwrap();
            assert_eq!(got, want, "parsing {raw}");
            let line = serde_json::to_string(&got).unwrap();
            let back: AnyValue = serde_json::from_str(&line).unwrap();
            assert_eq!(back, want, "roundtrip {raw}");
        }
        // bytes roundtrip
        let b = AnyValue::BytesValue(vec![0x00, 0xff, 0x10]);
        let line = serde_json::to_string(&b).unwrap();
        assert!(line.contains("bytesValue"));
        assert_eq!(serde_json::from_str::<AnyValue>(&line).unwrap(), b);
    }
}
