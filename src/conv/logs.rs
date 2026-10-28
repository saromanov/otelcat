//! logs/v1 conversions.

use super::vec_conv;
use crate::json::logs as jlogs;
use opentelemetry_proto::tonic::collector::logs::v1 as pcollect;
use opentelemetry_proto::tonic::logs::v1 as plogs;

impl From<jlogs::ExportLogsServiceRequest> for pcollect::ExportLogsServiceRequest {
    fn from(v: jlogs::ExportLogsServiceRequest) -> Self {
        pcollect::ExportLogsServiceRequest {
            resource_logs: vec_conv(v.resource_logs),
        }
    }
}

impl From<pcollect::ExportLogsServiceRequest> for jlogs::ExportLogsServiceRequest {
    fn from(v: pcollect::ExportLogsServiceRequest) -> Self {
        jlogs::ExportLogsServiceRequest {
            resource_logs: vec_conv(v.resource_logs),
        }
    }
}

impl From<jlogs::ResourceLogs> for plogs::ResourceLogs {
    fn from(v: jlogs::ResourceLogs) -> Self {
        plogs::ResourceLogs {
            resource: v
                .resource
                .map(opentelemetry_proto::tonic::resource::v1::Resource::from),
            scope_logs: vec_conv(v.scope_logs),
            schema_url: v.schema_url,
        }
    }
}

impl From<plogs::ResourceLogs> for jlogs::ResourceLogs {
    fn from(v: plogs::ResourceLogs) -> Self {
        jlogs::ResourceLogs {
            resource: v.resource.map(crate::json::common::Resource::from),
            scope_logs: vec_conv(v.scope_logs),
            schema_url: v.schema_url,
        }
    }
}

impl From<jlogs::ScopeLogs> for plogs::ScopeLogs {
    fn from(v: jlogs::ScopeLogs) -> Self {
        plogs::ScopeLogs {
            scope: v
                .scope
                .map(opentelemetry_proto::tonic::common::v1::InstrumentationScope::from),
            log_records: vec_conv(v.log_records),
            schema_url: v.schema_url,
        }
    }
}

impl From<plogs::ScopeLogs> for jlogs::ScopeLogs {
    fn from(v: plogs::ScopeLogs) -> Self {
        jlogs::ScopeLogs {
            scope: v.scope.map(crate::json::common::InstrumentationScope::from),
            log_records: vec_conv(v.log_records),
            schema_url: v.schema_url,
        }
    }
}

impl From<jlogs::LogRecord> for plogs::LogRecord {
    fn from(v: jlogs::LogRecord) -> Self {
        plogs::LogRecord {
            time_unix_nano: v.time_unix_nano,
            observed_time_unix_nano: v.observed_time_unix_nano,
            severity_number: v.severity_number,
            severity_text: v.severity_text,
            body: v
                .body
                .map(opentelemetry_proto::tonic::common::v1::AnyValue::from),
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
            flags: v.flags,
            trace_id: v.trace_id,
            span_id: v.span_id,
            event_name: v.event_name,
        }
    }
}

impl From<plogs::LogRecord> for jlogs::LogRecord {
    fn from(v: plogs::LogRecord) -> Self {
        jlogs::LogRecord {
            time_unix_nano: v.time_unix_nano,
            observed_time_unix_nano: v.observed_time_unix_nano,
            severity_number: v.severity_number,
            severity_text: v.severity_text,
            body: v.body.map(crate::json::common::AnyValue::from),
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
            flags: v.flags,
            trace_id: v.trace_id,
            span_id: v.span_id,
            event_name: v.event_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::common as jcommon;

    #[test]
    fn log_record_roundtrip() {
        let jrec = jlogs::LogRecord {
            time_unix_nano: 100,
            observed_time_unix_nano: 101,
            severity_number: 17,
            severity_text: "ERROR".into(),
            body: Some(jcommon::AnyValue::StringValue("boom".into())),
            attributes: vec![jcommon::KeyValue {
                key: "k".into(),
                value: Some(jcommon::AnyValue::BoolValue(true)),
            }],
            dropped_attributes_count: 2,
            flags: 1,
            trace_id: (0..16).collect(),
            span_id: (0..8).collect(),
            event_name: "ev".into(),
        };
        let prec: plogs::LogRecord = jrec.clone().into();
        let back: jlogs::LogRecord = prec.into();
        assert_eq!(jrec, back);
        let line = serde_json::to_string(&jrec).unwrap();
        let parsed: jlogs::LogRecord = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed, jrec);
    }

    #[test]
    fn logs_request_roundtrip() {
        let jreq = jlogs::ExportLogsServiceRequest {
            resource_logs: vec![jlogs::ResourceLogs {
                resource: Some(crate::json::common::Resource {
                    attributes: vec![],
                    dropped_attributes_count: 0,
                    entity_refs: vec![jcommon::EntityRef::default()],
                }),
                scope_logs: vec![jlogs::ScopeLogs {
                    scope: None,
                    log_records: vec![jlogs::LogRecord::default()],
                    schema_url: String::new(),
                }],
                schema_url: "u".into(),
            }],
        };
        let preq: pcollect::ExportLogsServiceRequest = jreq.clone().into();
        let back: jlogs::ExportLogsServiceRequest = preq.into();
        assert_eq!(jreq, back);
    }
}
