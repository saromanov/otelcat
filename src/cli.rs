//! Command-line interface.

use crate::replay::Protocol;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "otelcat",
    version,
    about = "kcat for OpenTelemetry: receive, capture and replay OTLP",
    after_help = "Examples:\n  otelcat serve --out capture.jsonl\n  otelcat replay capture.jsonl --to http://localhost:4317 --speed 2\n  cat capture.jsonl | otelcat replay - --to http://localhost:4318 --max"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Receive OTLP and print/capture it.
    Serve {
        /// Host to bind both endpoints on.
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
        /// OTLP/gRPC port (4317-style).
        #[arg(long, default_value_t = 4317)]
        grpc_port: u16,
        /// OTLP/HTTP port (4318-style).
        #[arg(long, default_value_t = 4318)]
        http_port: u16,
        /// Write a JSONL capture file.
        #[arg(long, short = 'o')]
        out: Option<PathBuf>,
        /// Emit OTLP/JSON lines to stdout instead of the pretty format.
        #[arg(long)]
        json: bool,
    },
    /// Replay a JSONL capture into a live endpoint.
    Replay {
        /// Capture file (JSONL), or `-` for stdin.
        input: String,
        /// Target endpoint, e.g. http://localhost:4317 (gRPC) or http://localhost:4318 (HTTP).
        /// Optional with --dry-run.
        #[arg(long)]
        to: Option<String>,
        /// Playback speed multiplier (2 = twice as fast).
        #[arg(long, default_value_t = 1.0)]
        speed: f64,
        /// Send as fast as possible, ignoring original pacing.
        #[arg(long)]
        max: bool,
        /// Restart from the beginning until interrupted.
        #[arg(long)]
        r#loop: bool,
        /// Transport protocol for the target.
        #[arg(long, value_enum, default_value_t = Protocol::Grpc)]
        protocol: Protocol,
        /// Validate the capture file without sending anything.
        #[arg(long)]
        dry_run: bool,
        /// Cap for a single inter-request delay, milliseconds.
        #[arg(long, default_value_t = 60_000)]
        max_gap_ms: u64,
    },
}
