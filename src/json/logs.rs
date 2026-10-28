//! logs/v1 JSON model.

use serde::{Deserialize, Serialize};

use super::common::{AnyValue, InstrumentationScope, KeyValue, Resource};
use super::wire;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLogsServiceRequest {
    #[serde(
        alias = "resource_logs",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub resource_logs: Vec<ResourceLogs>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLogs {
    #[serde(alias = "resource", default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    #[serde(alias = "scope_logs", default, skip_serializing_if = "Vec::is_empty")]
    pub scope_logs: Vec<ScopeLogs>,
    #[serde(
        alias = "schema_url",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub schema_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeLogs {
    #[serde(alias = "scope", default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<InstrumentationScope>,
    #[serde(alias = "log_records", default, skip_serializing_if = "Vec::is_empty")]
    pub log_records: Vec<LogRecord>,
    #[serde(
        alias = "schema_url",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub schema_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRecord {
    #[serde(
        alias = "time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub time_unix_nano: u64,
    #[serde(
        alias = "observed_time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub observed_time_unix_nano: u64,
    #[serde(
        alias = "severity_number",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub severity_number: i32,
    #[serde(
        alias = "severity_text",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub severity_text: String,
    #[serde(alias = "body", default, skip_serializing_if = "Option::is_none")]
    pub body: Option<AnyValue>,
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
        alias = "event_name",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub event_name: String,
}
