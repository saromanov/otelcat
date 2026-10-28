//! trace/v1 conversions.

use super::vec_conv;
use crate::json::trace as jtrace;
use opentelemetry_proto::tonic::collector::trace::v1 as pcollect;
use opentelemetry_proto::tonic::resource::v1 as presource;
use opentelemetry_proto::tonic::trace::v1 as ptrace;

impl From<jtrace::ExportTraceServiceRequest> for pcollect::ExportTraceServiceRequest {
    fn from(v: jtrace::ExportTraceServiceRequest) -> Self {
        pcollect::ExportTraceServiceRequest {
            resource_spans: vec_conv(v.resource_spans),
        }
    }
}

impl From<pcollect::ExportTraceServiceRequest> for jtrace::ExportTraceServiceRequest {
    fn from(v: pcollect::ExportTraceServiceRequest) -> Self {
        jtrace::ExportTraceServiceRequest {
            resource_spans: vec_conv(v.resource_spans),
        }
    }
}

impl From<jtrace::ResourceSpans> for ptrace::ResourceSpans {
    fn from(v: jtrace::ResourceSpans) -> Self {
        ptrace::ResourceSpans {
            resource: v.resource.map(presource::Resource::from),
            scope_spans: vec_conv(v.scope_spans),
            schema_url: v.schema_url,
        }
    }
}

impl From<ptrace::ResourceSpans> for jtrace::ResourceSpans {
    fn from(v: ptrace::ResourceSpans) -> Self {
        jtrace::ResourceSpans {
            resource: v.resource.map(crate::json::common::Resource::from),
            scope_spans: vec_conv(v.scope_spans),
            schema_url: v.schema_url,
        }
    }
}

impl From<jtrace::ScopeSpans> for ptrace::ScopeSpans {
    fn from(v: jtrace::ScopeSpans) -> Self {
        ptrace::ScopeSpans {
            scope: v
                .scope
                .map(opentelemetry_proto::tonic::common::v1::InstrumentationScope::from),
            spans: vec_conv(v.spans),
            schema_url: v.schema_url,
        }
    }
}

impl From<ptrace::ScopeSpans> for jtrace::ScopeSpans {
    fn from(v: ptrace::ScopeSpans) -> Self {
        jtrace::ScopeSpans {
            scope: v.scope.map(crate::json::common::InstrumentationScope::from),
            spans: vec_conv(v.spans),
            schema_url: v.schema_url,
        }
    }
}

impl From<jtrace::Span> for ptrace::Span {
    fn from(v: jtrace::Span) -> Self {
        ptrace::Span {
            trace_id: v.trace_id,
            span_id: v.span_id,
            trace_state: v.trace_state,
            parent_span_id: v.parent_span_id,
            flags: v.flags,
            name: v.name,
            kind: v.kind,
            start_time_unix_nano: v.start_time_unix_nano,
            end_time_unix_nano: v.end_time_unix_nano,
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
            events: vec_conv(v.events),
            dropped_events_count: v.dropped_events_count,
            links: vec_conv(v.links),
            dropped_links_count: v.dropped_links_count,
            status: v.status.map(ptrace::Status::from),
        }
    }
}

impl From<ptrace::Span> for jtrace::Span {
    fn from(v: ptrace::Span) -> Self {
        jtrace::Span {
            trace_id: v.trace_id,
            span_id: v.span_id,
            trace_state: v.trace_state,
            parent_span_id: v.parent_span_id,
            flags: v.flags,
            name: v.name,
            kind: v.kind,
            start_time_unix_nano: v.start_time_unix_nano,
            end_time_unix_nano: v.end_time_unix_nano,
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
            events: vec_conv(v.events),
            dropped_events_count: v.dropped_events_count,
            links: vec_conv(v.links),
            dropped_links_count: v.dropped_links_count,
            status: v.status.map(jtrace::Status::from),
        }
    }
}

impl From<jtrace::Event> for ptrace::span::Event {
    fn from(v: jtrace::Event) -> Self {
        ptrace::span::Event {
            time_unix_nano: v.time_unix_nano,
            name: v.name,
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
        }
    }
}

impl From<ptrace::span::Event> for jtrace::Event {
    fn from(v: ptrace::span::Event) -> Self {
        jtrace::Event {
            time_unix_nano: v.time_unix_nano,
            name: v.name,
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
        }
    }
}

impl From<jtrace::Link> for ptrace::span::Link {
    fn from(v: jtrace::Link) -> Self {
        ptrace::span::Link {
            trace_id: v.trace_id,
            span_id: v.span_id,
            trace_state: v.trace_state,
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
            flags: v.flags,
        }
    }
}

impl From<ptrace::span::Link> for jtrace::Link {
    fn from(v: ptrace::span::Link) -> Self {
        jtrace::Link {
            trace_id: v.trace_id,
            span_id: v.span_id,
            trace_state: v.trace_state,
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
            flags: v.flags,
        }
    }
}

impl From<jtrace::Status> for ptrace::Status {
    fn from(v: jtrace::Status) -> Self {
        ptrace::Status {
            message: v.message,
            code: v.code,
        }
    }
}

impl From<ptrace::Status> for jtrace::Status {
    fn from(v: ptrace::Status) -> Self {
        jtrace::Status {
            message: v.message,
            code: v.code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_span() -> (ptrace::Span, jtrace::Span) {
        let pspan = ptrace::Span {
            trace_id: (0..16).collect(),
            span_id: (0..8).collect(),
            trace_state: "k=v".into(),
            parent_span_id: (0..8).collect(),
            flags: 0x100,
            name: "GET /users".into(),
            kind: 2,
            start_time_unix_nano: 1_000,
            end_time_unix_nano: 2_000,
            attributes: vec![opentelemetry_proto::tonic::common::v1::KeyValue {
                key: "http.method".into(),
                value: Some(opentelemetry_proto::tonic::common::v1::AnyValue {
                    value: Some(
                        opentelemetry_proto::tonic::common::v1::any_value::Value::StringValue(
                            "GET".into(),
                        ),
                    ),
                }),
                key_strindex: 0,
            }],
            dropped_attributes_count: 1,
            events: vec![ptrace::span::Event {
                time_unix_nano: 1_500,
                name: "event".into(),
                attributes: vec![],
                dropped_attributes_count: 0,
            }],
            dropped_events_count: 0,
            links: vec![ptrace::span::Link {
                trace_id: (16..32).collect(),
                span_id: (8..16).collect(),
                trace_state: String::new(),
                attributes: vec![],
                dropped_attributes_count: 0,
                flags: 0,
            }],
            dropped_links_count: 0,
            status: Some(ptrace::Status {
                message: "oops".into(),
                code: 2,
            }),
        };
        (pspan.clone(), pspan.into())
    }

    #[test]
    fn span_json_prost_roundtrip() {
        let (pspan, jspan) = sample_span();
        let back: ptrace::Span = jspan.into();
        assert_eq!(pspan, back);
    }

    #[test]
    fn span_json_roundtrip() {
        let (_, jspan) = sample_span();
        let line = serde_json::to_string(&jspan).unwrap();
        let back: jtrace::Span = serde_json::from_str(&line).unwrap();
        assert_eq!(jspan, back);
        assert!(line.contains("startTimeUnixNano"));
        assert!(line.contains("droppedAttributesCount"));
    }

    #[test]
    fn request_roundtrip() {
        let jreq = jtrace::ExportTraceServiceRequest {
            resource_spans: vec![jtrace::ResourceSpans {
                resource: Some(crate::json::common::Resource {
                    attributes: vec![crate::json::common::KeyValue {
                        key: "service.name".into(),
                        value: Some(crate::json::common::AnyValue::StringValue("svc".into())),
                    }],
                    dropped_attributes_count: 0,
                    entity_refs: vec![],
                }),
                scope_spans: vec![jtrace::ScopeSpans {
                    scope: Some(crate::json::common::InstrumentationScope {
                        name: "lib".into(),
                        ..Default::default()
                    }),
                    spans: vec![jtrace::Span::default()],
                    schema_url: "https://example.com".into(),
                }],
                schema_url: String::new(),
            }],
        };
        let preq: pcollect::ExportTraceServiceRequest = jreq.clone().into();
        let back: jtrace::ExportTraceServiceRequest = preq.into();
        assert_eq!(jreq, back);
    }
}
