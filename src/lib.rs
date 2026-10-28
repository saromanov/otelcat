//! `otelcat` — kcat for OpenTelemetry.
//!
//! Receive, inspect, capture and replay OTLP telemetry from the command line.
//!
//! - [`server`]: OTLP/gRPC + OTLP/HTTP receiver that streams requests into a sink.
//! - [`json`]: OTLP/JSON model (camelCase emission, tolerant camelCase/snake_case parsing).
//! - [`conv`]: conversions between prost-generated and JSON model types.
//! - [`sink`]: pretty printer + JSONL capture writer.
//! - [`replay`]: capture-file playback to a live endpoint.

pub mod cli;
pub mod conv;
pub mod json;
pub mod printer;
pub mod replay;
pub mod server;
pub mod sink;
