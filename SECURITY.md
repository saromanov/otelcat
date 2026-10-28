# Security Policy

## Supported versions

Only the latest release is supported with security fixes.

## Reporting a vulnerability

Please do **not** open a public issue for security problems.

Report privately via GitHub Security Advisories
(**Security → Report a vulnerability** on this repository) or by contacting
the maintainer directly. Include a description, reproduction steps and any
capture files or telemetry samples (sanitized) that help reproduce the issue.

You can expect a response within 7 days.

## Scope

- `otelcat serve` exposes a network listener; treat deployment accordingly
  (bind to loopback unless you know you need more).
- Capture files contain full telemetry payloads — handle them like logs.
