//! trace/v1 JSON model.

use serde::{Deserialize, Serialize};

use super::common::{InstrumentationScope, KeyValue, Resource};
use super::wire;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTraceServiceRequest {
    #[serde(
        alias = "resource_spans",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub resource_spans: Vec<ResourceSpans>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpans {
    #[serde(alias = "resource", default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    #[serde(alias = "scope_spans", default, skip_serializing_if = "Vec::is_empty")]
    pub scope_spans: Vec<ScopeSpans>,
    #[serde(
        alias = "schema_url",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub schema_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeSpans {
    #[serde(alias = "scope", default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<InstrumentationScope>,
    #[serde(alias = "spans", default, skip_serializing_if = "Vec::is_empty")]
    pub spans: Vec<Span>,
    #[serde(
        alias = "schema_url",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub schema_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    #[serde(
        alias = "trace_id",
        with = "wire::hex_bytes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub trace_id: Vec<u8>,
    #[serde(
        alias = "span_id",
        with = "wire::hex_bytes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub span_id: Vec<u8>,
    #[serde(
        alias = "trace_state",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub trace_state: String,
    #[serde(
        alias = "parent_span_id",
        with = "wire::hex_bytes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub parent_span_id: Vec<u8>,
    #[serde(alias = "flags", default, skip_serializing_if = "crate::json::is_zero")]
    pub flags: u32,
    #[serde(alias = "name", default)]
    pub name: String,
    #[serde(alias = "kind", default, skip_serializing_if = "crate::json::is_zero")]
    pub kind: i32,
    #[serde(
        alias = "start_time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub start_time_unix_nano: u64,
    #[serde(
        alias = "end_time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub end_time_unix_nano: u64,
    #[serde(alias = "attributes", default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(
        alias = "dropped_attributes_count",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub dropped_attributes_count: u32,
    #[serde(alias = "events", default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,
    #[serde(
        alias = "dropped_events_count",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub dropped_events_count: u32,
    #[serde(alias = "links", default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    #[serde(
        alias = "dropped_links_count",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub dropped_links_count: u32,
    #[serde(alias = "status", default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(
        alias = "time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub time_unix_nano: u64,
    #[serde(alias = "name", default)]
    pub name: String,
    #[serde(alias = "attributes", default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(
        alias = "dropped_attributes_count",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub dropped_attributes_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    #[serde(
        alias = "trace_id",
        with = "wire::hex_bytes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub trace_id: Vec<u8>,
    #[serde(
        alias = "span_id",
        with = "wire::hex_bytes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub span_id: Vec<u8>,
    #[serde(
        alias = "trace_state",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub trace_state: String,
    #[serde(alias = "attributes", default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(
        alias = "dropped_attributes_count",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub dropped_attributes_count: u32,
    #[serde(alias = "flags", default, skip_serializing_if = "crate::json::is_zero")]
    pub flags: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    #[serde(alias = "message", default, skip_serializing_if = "String::is_empty")]
    pub message: String,
    #[serde(alias = "code", default, skip_serializing_if = "crate::json::is_zero")]
    pub code: i32,
}
