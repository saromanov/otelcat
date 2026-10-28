# otelcat — kcat for OpenTelemetry

Receive, inspect, capture and replay OTLP telemetry from the command line.

`otelcat` is to OpenTelemetry what `kcat` is to Kafka: a small, scriptable
tool for the moments when you want to see exactly what your services emit —
without standing up a collector, a Grafana stack, or a UI.

```
$ otelcat serve
▸ POST /orders checkout-api  12.5ms  {http.method=POST, order.count=3} trace=5b8efff7 span=eee19b7e
  └ sql.query postgres  2.1ms  {db.system=postgresql} trace=5b8efff7 span=aaa17c42
    • cached
L ERROR checkout-api 2018-12-13 14:51:00.000 "order retry exceeded" trace=5b8efff7 span=eee19b7e
M sum orders.total 42 checkout-api cumulative monotonic
```

## Why

Every OpenTelemetry user eventually needs to answer *"what exactly is my app
sending?"*. Existing tools are interactive viewers — great for eyeballing,
awkward for pipelines. `otelcat` composes like a Unix tool:

- **Receive** — a full OTLP receiver (gRPC on `:4317`, HTTP on `:4318`) in a
  single static binary. Point your app's `OTEL_EXPORTER_OTLP_ENDPOINT` at it.
- **See** — human-readable output with span trees, error statuses, log
  severities and metric points.
- **Capture** — write everything to interoperable [OTLP/JSON Lines][otlpjson]
  (`--out capture.jsonl`), jq-parseable out of the box.
- **Replay** — push a capture back into any OTLP endpoint, with the original
  pacing (`--speed 2`, `--max`, `--loop`) — perfect for reproducing collector
  bugs, testing backends, or re-feeding a captured incident.
- **Validate** — `--dry-run` parses a capture file without touching the
  network.

[otlpjson]: https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding

## Install

Grab a prebuilt binary from [Releases](https://github.com/saromanov/otelcat/releases)
(linux/macOS, amd64/arm64, fully static on Linux musl) or build from source:

```sh
cargo install --git https://github.com/saromanov/otelcat
```

## Usage

### Receive and capture

```sh
otelcat serve                      # pretty output on both OTLP ports
otelcat serve --out capture.jsonl  # + JSONL capture file
otelcat serve --json               # OTLP/JSON lines to stdout, pipe into jq
otelcat serve --json | jq '.resourceSpans[0].scopeSpans[0].spans[].name'
```

Point your application at it:

```sh
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 ./my-app
```

Both transports are supported on both ports: OTLP/gRPC (`:4317` default) and
OTLP/HTTP (`:4318` default), protobuf and JSON bodies, gzip decompression.
Use `--bind`, `--grpc-port` and `--http-port` to customize.

### Replay a capture

```sh
otelcat replay capture.jsonl --to http://localhost:4317          # original pacing
otelcat replay capture.jsonl --to http://localhost:4317 --max    # as fast as possible
otelcat replay capture.jsonl --to http://localhost:4317 --speed 2 --loop
otelcat replay - --to http://localhost:4318 --protocol http < capture.jsonl
otelcat replay capture.jsonl --dry-run                           # validate only
```

Replay preserves the original timestamps recorded in the telemetry; the
`--speed` multiplier only controls how fast the wall-clock pacing is played
back.

### The capture format

One OTLP export request per line, as standard OTLP/JSON
(camelCase keys, 64-bit integers as strings, hex trace/span IDs, base64
bytes). The signal is identified by the top-level key
(`resourceSpans` / `resourceLogs` / `resourceMetrics`). Parsing accepts both
camelCase and snake_case keys, as the OTLP spec requires.

## Compatibility

| | |
|---|---|
| Signals | traces, logs, metrics |
| Transports | OTLP/gRPC, OTLP/HTTP (protobuf + JSON, gzip) |
| Output | pretty, OTLP/JSONL capture |
| Platforms | linux (amd64, arm64, static musl), macOS (x86_64, aarch64), Windows (x86_64) |

## Roadmap

- `otelcat gen` — synthetic traffic (service graph, configurable rps)
- Filtering (`--where 'status.code==ERROR'`) and live aggregate stats
- Forward mode: tap traffic and mirror it to a real backend (tcpdump-style)
- TLS for serve and replay targets
- MCP mode: let coding agents inspect local telemetry

## Development

```sh
cargo test        # unit + integration (in-process servers, no network)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option.
