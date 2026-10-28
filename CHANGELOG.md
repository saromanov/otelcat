# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-28

### Added

- `otelcat serve`: OTLP/gRPC + OTLP/HTTP receiver (protobuf and JSON bodies,
  gzip decoding) with pretty terminal output for traces (span trees, events,
  error statuses), logs and metrics.
- `--out capture.jsonl`: capture all inbound telemetry as OTLP/JSON Lines.
- `--json`: emit OTLP/JSON Lines to stdout for piping into `jq` and friends.
- `otelcat replay`: play a capture back into any OTLP endpoint over gRPC or
  HTTP, with original pacing (`--speed`), max-speed (`--max`), looping
  (`--loop`), stdin input and `--dry-run` validation.
- Tolerant OTLP/JSON parsing (camelCase and snake_case keys, strings or
  numbers for 64-bit integers).
- Cross-platform release binaries (linux amd64/arm64 static musl, macOS
  x86_64/aarch64, Windows x86_64).
