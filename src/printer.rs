//! Pretty human-readable rendering of signals to stdout.

use crate::json::{self, AnyValue, Signal};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::IsTerminal;

#[derive(Clone, Copy, Debug)]
pub struct Style {
    enabled: bool,
}

impl Style {
    pub fn auto() -> Self {
        let tty = std::io::stdout().is_terminal();
        let no_color = std::env::var_os("NO_COLOR").is_some();
        Style {
            enabled: tty && !no_color,
        }
    }

    pub fn plain() -> Self {
        Style { enabled: false }
    }

    fn wrap(&self, s: &str, f: impl FnOnce(&str) -> String) -> String {
        if self.enabled {
            f(s)
        } else {
            s.to_owned()
        }
    }

    pub fn bold(&self, s: &str) -> String {
        self.wrap(s, |s| s.bold().to_string())
    }

    pub fn dim(&self, s: &str) -> String {
        self.wrap(s, |s| s.dimmed().to_string())
    }

    pub fn cyan(&self, s: &str) -> String {
        self.wrap(s, |s| s.cyan().to_string())
    }

    pub fn green(&self, s: &str) -> String {
        self.wrap(s, |s| s.green().to_string())
    }

    pub fn yellow(&self, s: &str) -> String {
        self.wrap(s, |s| s.yellow().to_string())
    }

    pub fn red(&self, s: &str) -> String {
        self.wrap(s, |s| s.red().to_string())
    }

    pub fn magenta(&self, s: &str) -> String {
        self.wrap(s, |s| s.magenta().to_string())
    }
}

fn attrs_string(kvs: &[json::KeyValue]) -> String {
    if kvs.is_empty() {
        return String::new();
    }
    let mut out = String::from("{");
    for (i, kv) in kvs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "{}={}", kv.key, value_display(kv.value.as_ref()));
    }
    out.push('}');
    if out.len() > 160 {
        out.truncate(157);
        out.push_str("...}");
    }
    out
}

pub fn value_display(v: Option<&AnyValue>) -> String {
    match v {
        None => String::new(),
        Some(AnyValue::StringValue(s)) => s.clone(),
        Some(AnyValue::BoolValue(b)) => b.to_string(),
        Some(AnyValue::IntValue(i)) => i.to_string(),
        Some(AnyValue::DoubleValue(d)) => format!("{d}"),
        Some(AnyValue::ArrayValue(a)) => {
            let items: Vec<String> = a.values.iter().map(|v| value_display(Some(v))).collect();
            format!("[{}]", items.join(","))
        }
        Some(AnyValue::KvlistValue(k)) => attrs_string(&k.values),
        Some(AnyValue::BytesValue(b)) => format!("<{} bytes>", b.len()),
    }
}

pub fn duration_string(ns: u64) -> String {
    match ns {
        0 => String::new(),
        n if n < 1_000 => format!("{n}ns"),
        n if n < 1_000_000 => format!("{}µs", n / 1_000),
        n if n < 1_000_000_000 => format!("{:.1}ms", n as f64 / 1_000_000.0),
        n => format!("{:.2}s", n as f64 / 1_000_000_000.0),
    }
}

/// Civil UTC time from unix nanoseconds (no external time crate).
pub fn time_utc_string(ns: u64) -> String {
    let secs = (ns / 1_000_000_000) as i64;
    let millis = (ns % 1_000_000_000) / 1_000_000;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
        y,
        m,
        d,
        sod / 3600,
        (sod % 3600) / 60,
        sod % 60,
        millis
    )
}

/// Howard Hinnant's civil-from-days algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

pub fn severity_label(n: i32) -> &'static str {
    match n {
        1..=4 => "FATAL",
        5..=8 => "ERROR",
        9..=12 => "WARN",
        13..=16 => "INFO",
        17..=20 => "DEBUG",
        21..=24 => "TRACE",
        _ => "UNSET",
    }
}

fn service_name(attrs: &[json::KeyValue]) -> String {
    attrs
        .iter()
        .find(|kv| kv.key == "service.name")
        .and_then(|kv| kv.value.as_ref())
        .map(|v| value_display(Some(v)))
        .unwrap_or_default()
}

fn short_hex(v: &[u8], n: usize) -> String {
    let s = json::hex_encode(v);
    s.chars().take(n).collect()
}

pub fn print_signal(out: &mut dyn std::io::Write, sig: &Signal, st: &Style) {
    match sig {
        Signal::Traces(t) => print_traces(out, t, st),
        Signal::Logs(l) => print_logs(out, l, st),
        Signal::Metrics(m) => print_metrics(out, m, st),
    }
}

fn print_traces(out: &mut dyn std::io::Write, t: &json::ExportTraceServiceRequest, st: &Style) {
    use std::collections::HashMap;
    for rs in &t.resource_spans {
        let svc = rs
            .resource
            .as_ref()
            .map(|r| service_name(&r.attributes))
            .unwrap_or_default();
        for ss in &rs.scope_spans {
            // parent -> depth resolution within the whole request
            let mut parents: HashMap<&[u8], &[u8]> = HashMap::new();
            for ss2 in &rs.scope_spans {
                for sp in &ss2.spans {
                    parents.insert(sp.span_id.as_slice(), sp.parent_span_id.as_slice());
                }
            }
            let mut depths: HashMap<&[u8], usize> = HashMap::new();
            for sp in &ss.spans {
                let depth = depth_of(&parents, &mut depths, sp.span_id.as_slice());
                print_span(out, sp, &svc, depth, st);
            }
        }
    }
}

fn depth_of<'a>(
    parents: &HashMap<&'a [u8], &'a [u8]>,
    memo: &mut HashMap<&'a [u8], usize>,
    id: &'a [u8],
) -> usize {
    if let Some(d) = memo.get(id) {
        return *d;
    }
    let mut depth = 0;
    let mut cur = parents.get(id).copied();
    let mut guard = 0;
    while let Some(p) = cur {
        if p.is_empty() || guard > 64 {
            break;
        }
        if let Some(d) = memo.get(p) {
            depth = d + 1;
            break;
        }
        depth += 1;
        cur = parents.get(p).copied();
        guard += 1;
    }
    memo.insert(id, depth);
    depth
}

fn print_span(out: &mut dyn std::io::Write, sp: &json::Span, svc: &str, depth: usize, st: &Style) {
    let prefix = if depth == 0 {
        format!("{}", "▸ ".dimmed())
    } else {
        format!("{}└ ", "  ".repeat(depth))
    };
    let dur_ns = sp
        .end_time_unix_nano
        .saturating_sub(sp.start_time_unix_nano);
    let dur = if dur_ns > 0 {
        format!(" {} ", duration_string(dur_ns))
    } else {
        " ".to_string()
    };
    let dur_colored = if dur_ns >= 500_000_000 {
        st.red(&dur)
    } else {
        st.green(&dur)
    };
    let status = match sp.status.as_ref().map(|s| s.code).unwrap_or(0) {
        2 => format!(
            " {}",
            st.red(&format!(
                "ERROR {}",
                sp.status.as_ref().map(|s| s.message.as_str()).unwrap_or("")
            ))
        ),
        _ => String::new(),
    };
    let attrs = attrs_string(&sp.attributes);
    let attrs = if attrs.is_empty() {
        String::new()
    } else {
        format!(" {} ", st.dim(&attrs))
    };
    let svc = if svc.is_empty() {
        String::new()
    } else {
        format!(" {} ", st.cyan(svc))
    };
    let ids = format!(
        "trace={} span={}",
        st.dim(&short_hex(&sp.trace_id, 8)),
        st.dim(&short_hex(&sp.span_id, 8))
    );
    let tail = format!("{attrs}{ids}");
    let _ = writeln!(
        out,
        "{}{}{}{}{}{}",
        prefix,
        st.bold(&sp.name),
        svc,
        dur_colored,
        status,
        tail
    );
    for ev in &sp.events {
        let _ = writeln!(
            out,
            "{}  {} {}",
            " ".repeat(depth * 2 + 2),
            st.magenta("•"),
            st.dim(&ev.name)
        );
    }
}

fn print_logs(out: &mut dyn std::io::Write, l: &json::ExportLogsServiceRequest, st: &Style) {
    for rl in &l.resource_logs {
        let svc = rl
            .resource
            .as_ref()
            .map(|r| service_name(&r.attributes))
            .unwrap_or_default();
        for sl in &rl.scope_logs {
            for lr in &sl.log_records {
                let sev_num = lr.severity_number;
                let sev_text = if !lr.severity_text.is_empty() {
                    lr.severity_text.clone()
                } else {
                    severity_label(sev_num).to_string()
                };
                let sev = match severity_label(sev_num) {
                    "FATAL" | "ERROR" => st.red(&sev_text),
                    "WARN" => st.yellow(&sev_text),
                    "DEBUG" | "TRACE" => st.dim(&sev_text),
                    _ => sev_text,
                };
                let body = value_display(lr.body.as_ref());
                let t = if lr.time_unix_nano > 0 {
                    lr.time_unix_nano
                } else {
                    lr.observed_time_unix_nano
                };
                let time = if t > 0 {
                    format!("{} ", st.dim(&time_utc_string(t)))
                } else {
                    String::new()
                };
                let svc = if svc.is_empty() {
                    String::new()
                } else {
                    format!("{} ", st.cyan(&svc))
                };
                let attrs = attrs_string(&lr.attributes);
                let attrs = if attrs.is_empty() {
                    String::new()
                } else {
                    format!(" {} ", st.dim(&attrs))
                };
                let ids = if lr.trace_id.is_empty() {
                    String::new()
                } else {
                    format!(
                        "trace={} span={}",
                        st.dim(&short_hex(&lr.trace_id, 8)),
                        st.dim(&short_hex(&lr.span_id, 8))
                    )
                };
                let head = format!("{} {}", st.magenta("L"), sev);
                let body_part = format!(" {}", st.bold(&body));
                let svc_part = format!(" {}", svc);
                let time_attrs = format!("{time}{attrs}");
                let _ = writeln!(
                    out,
                    "{}{}{}{}{}",
                    head, body_part, svc_part, time_attrs, ids
                );
            }
        }
    }
}

fn temporality_string(t: i32) -> &'static str {
    match t {
        1 => "delta",
        2 => "cumulative",
        _ => "",
    }
}

fn print_metrics(out: &mut dyn std::io::Write, m: &json::ExportMetricsServiceRequest, st: &Style) {
    for rm in &m.resource_metrics {
        let svc = rm
            .resource
            .as_ref()
            .map(|r| service_name(&r.attributes))
            .unwrap_or_default();
        for sm in &rm.scope_metrics {
            for metric in &sm.metrics {
                let Some(data) = &metric.data else {
                    continue;
                };
                match data {
                    json::MetricData::Gauge(g) => {
                        for p in &g.data_points {
                            point_line(out, "gauge", metric, p, "", st, &svc);
                        }
                    }
                    json::MetricData::Sum(s) => {
                        let extra = format!(
                            "{}{}",
                            temporality_string(s.aggregation_temporality),
                            if s.is_monotonic { " monotonic" } else { "" }
                        );
                        for p in &s.data_points {
                            point_line(out, "sum", metric, p, &extra, st, &svc);
                        }
                    }
                    json::MetricData::Histogram(h) => {
                        let extra = temporality_string(h.aggregation_temporality);
                        for p in &h.data_points {
                            let mut val = format!("count={}", p.count);
                            if let Some(s) = p.sum {
                                let _ = write!(val, " sum={s}");
                            }
                            if let Some(mn) = p.min {
                                let _ = write!(val, " min={mn}");
                            }
                            if let Some(mx) = p.max {
                                let _ = write!(val, " max={mx}");
                            }
                            let _ = write!(val, " buckets={}", p.bucket_counts.len());
                            metric_line(
                                out,
                                "histogram",
                                &metric.name,
                                &p.attributes,
                                &val,
                                extra,
                                st,
                                &svc,
                            );
                        }
                    }
                    json::MetricData::ExponentialHistogram(h) => {
                        let extra = temporality_string(h.aggregation_temporality);
                        for p in &h.data_points {
                            let mut val = format!("count={} scale={}", p.count, p.scale);
                            if let Some(s) = p.sum {
                                let _ = write!(val, " sum={s}");
                            }
                            metric_line(
                                out,
                                "exphistogram",
                                &metric.name,
                                &p.attributes,
                                &val,
                                extra,
                                st,
                                &svc,
                            );
                        }
                    }
                    json::MetricData::Summary(s) => {
                        for p in &s.data_points {
                            let mut val = format!("count={} sum={}", p.count, p.sum);
                            for q in &p.quantile_values {
                                let _ = write!(val, " p{}={}", q.quantile, q.value);
                            }
                            metric_line(
                                out,
                                "summary",
                                &metric.name,
                                &p.attributes,
                                &val,
                                "",
                                st,
                                &svc,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn point_line(
    out: &mut dyn std::io::Write,
    kind: &str,
    metric: &json::Metric,
    p: &json::NumberDataPoint,
    extra: &str,
    st: &Style,
    svc: &str,
) {
    let val = match p.value {
        Some(json::PointValue::AsDouble(d)) => format!("{d}"),
        Some(json::PointValue::AsInt(i)) => format!("{i}"),
        None => String::new(),
    };
    metric_line(out, kind, &metric.name, &p.attributes, &val, extra, st, svc);
}

#[allow(clippy::too_many_arguments)]
fn metric_line(
    out: &mut dyn std::io::Write,
    kind: &str,
    name: &str,
    attrs: &[json::KeyValue],
    value: &str,
    extra: &str,
    st: &Style,
    svc: &str,
) {
    let attrs_s = attrs_string(attrs);
    let attrs_s = if attrs_s.is_empty() {
        String::new()
    } else {
        format!(" {}", st.dim(&attrs_s))
    };
    let svc = if svc.is_empty() {
        String::new()
    } else {
        format!(" {}", st.cyan(svc))
    };
    let extra = if extra.is_empty() {
        String::new()
    } else {
        format!(" {}", st.dim(extra))
    };
    let head = format!("{} {} {}", st.magenta("M"), st.dim(kind), st.bold(name));
    let value_part = format!(" {}", st.green(value));
    let tail = format!("{svc}{extra}");
    let _ = writeln!(out, "{}{}{}{}", head, attrs_s, value_part, tail);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_formatting() {
        assert_eq!(duration_string(0), "");
        assert_eq!(duration_string(500), "500ns");
        assert_eq!(duration_string(1_500_000), "1.5ms");
        assert_eq!(duration_string(2_500_000_000), "2.50s");
    }

    #[test]
    fn utc_time_formatting() {
        assert_eq!(time_utc_string(0), "1970-01-01 00:00:00.000");
        // 1544712660000000000 ns = 2018-12-13 14:51:00 UTC (OTLP example timestamp)
        assert_eq!(
            time_utc_string(1_544_712_660_000_000_000),
            "2018-12-13 14:51:00.000"
        );
    }

    #[test]
    fn severity_labels() {
        assert_eq!(severity_label(9), "WARN");
        assert_eq!(severity_label(17), "DEBUG");
        assert_eq!(severity_label(0), "UNSET");
        assert_eq!(severity_label(25), "UNSET");
    }
}
