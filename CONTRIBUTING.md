# Contributing to otelcat

Thanks for your interest in improving otelcat!

## Development

```sh
cargo test                                        # unit + integration tests
cargo clippy --all-targets -- -D warnings         # lints
cargo fmt                                         # formatting
```

Tests run fully in-process (the test suite starts its own OTLP servers on
ephemeral ports), so no external services are needed.

## Guidelines

- Keep `cargo fmt`, `clippy -D warnings` and `cargo test` green; CI enforces all three.
- Every feature or fix should come with tests. Round-trip tests
  (gRPC/HTTP → serve → capture → replay → byte-identical capture) are the
  backbone of this repo — extend them when adding new signal fields.
- OTLP/JSON emission must stay spec-compatible: camelCase keys, 64-bit
  integers as strings, hex IDs, base64 bytes. Parsing must accept both
  camelCase and snake_case.
- User-visible changes go to CHANGELOG.md (Keep a Changelog format).
- Commit messages: short imperative summary, details below if needed.

## Pull requests

1. Fork and create a feature branch.
2. Make your change with tests.
3. Open a PR and fill in the checklist from the template.

## Reporting bugs

Open an issue with the bug template — include `otelcat --version`, your OS,
and a sanitized capture excerpt if possible.
