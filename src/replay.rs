//! Replay: stream a capture file into a live OTLP endpoint with pacing.

use crate::json::Signal;
use crate::sink;
use anyhow::Context as _;
use std::io::BufRead as _;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Protocol {
    /// OTLP/gRPC (default).
    #[default]
    Grpc,
    /// OTLP/HTTP protobuf.
    Http,
}

#[derive(Clone, Debug)]
pub struct ReplayConfig {
    /// Path to the capture file, or `-` for stdin.
    pub input: String,
    /// Target endpoint, e.g. `http://localhost:4317` (gRPC) or `http://localhost:4318`.
    pub target: String,
    /// Playback speed multiplier (1.0 = original pacing).
    pub speed: f64,
    /// Send everything as fast as possible.
    pub max_speed: bool,
    /// Restart from the beginning after reaching the end, until interrupted.
    pub r#loop: bool,
    pub protocol: Protocol,
    /// Parse and validate only, no network traffic.
    pub dry_run: bool,
    /// Upper bound for a single inter-request delay in milliseconds.
    pub max_gap_ms: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplayStats {
    pub loops: u64,
    pub lines: u64,
    pub spans: u64,
    pub log_records: u64,
    pub metric_points: u64,
    pub elapsed: Duration,
}

impl ReplayStats {
    pub fn total_items(&self) -> u64 {
        self.spans + self.log_records + self.metric_points
    }

    pub fn items_per_sec(&self) -> f64 {
        let secs = self.elapsed.as_secs_f64();
        if secs <= 0.0 {
            return 0.0;
        }
        self.total_items() as f64 / secs
    }
}

/// Sends one parsed signal to the target.
#[async_trait::async_trait]
trait SignalSender: Send {
    async fn send(&mut self, sig: &Signal) -> anyhow::Result<()>;
}

pub async fn run(cfg: ReplayConfig) -> anyhow::Result<ReplayStats> {
    let started = Instant::now();
    let mut stats = ReplayStats::default();

    if cfg.dry_run {
        let reader = open_input(&cfg.input)?;
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let sig = Signal::from_line(&line).context("invalid capture line")?;
            accumulate(&mut stats, &sig);
            stats.lines += 1;
        }
        stats.elapsed = started.elapsed();
        return Ok(stats);
    }

    if !cfg.target.starts_with("http://") && !cfg.target.starts_with("https://") {
        anyhow::bail!("target must include a scheme, e.g. http://localhost:4317");
    }

    tokio::select! {
        r = run_passes(&cfg, &mut stats) => r?,
        _ = tokio::signal::ctrl_c() => {
            eprintln!("interrupted");
        }
    }

    stats.elapsed = started.elapsed();
    Ok(stats)
}

fn open_input(path: &str) -> anyhow::Result<Box<dyn std::io::BufRead>> {
    if path == "-" {
        Ok(Box::new(std::io::BufReader::new(std::io::stdin())))
    } else {
        let f =
            std::fs::File::open(path).with_context(|| format!("opening capture file {path}"))?;
        Ok(Box::new(std::io::BufReader::new(f)))
    }
}

fn accumulate(stats: &mut ReplayStats, sig: &Signal) {
    let (spans, records, points) = sink::count(sig);
    stats.spans += spans;
    stats.log_records += records;
    stats.metric_points += points;
}

async fn run_passes(cfg: &ReplayConfig, stats: &mut ReplayStats) -> anyhow::Result<()> {
    let mut pass = 0u64;
    loop {
        pass += 1;
        stats.loops = pass;
        match cfg.protocol {
            Protocol::Grpc => {
                let mut sender = GrpcSender::connect(&cfg.target).await?;
                one_pass(cfg, stats, &mut sender).await?;
            }
            Protocol::Http => {
                let mut sender = HttpSender::new(&cfg.target)?;
                one_pass(cfg, stats, &mut sender).await?;
            }
        }
        if !cfg.r#loop {
            break;
        }
    }
    Ok(())
}

async fn one_pass(
    cfg: &ReplayConfig,
    stats: &mut ReplayStats,
    sender: &mut dyn SignalSender,
) -> anyhow::Result<()> {
    let reader = open_input(&cfg.input)?;
    let mut prev_t: Option<u64> = None;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let sig = Signal::from_line(&line).context("invalid capture line")?;
        let t = sig.time_unix_nano().unwrap_or(0);
        if !cfg.max_speed {
            if let Some(prev) = prev_t.filter(|p| t > *p) {
                let gap_ns = (t - prev) as f64 / cfg.speed.max(0.001);
                let gap_ms = (gap_ns / 1_000_000.0).min(cfg.max_gap_ms as f64);
                if gap_ms >= 1.0 {
                    sleep(Duration::from_millis(gap_ms as u64)).await;
                }
            }
        }
        prev_t = Some(t);
        sender.send(&sig).await?;
        accumulate(stats, &sig);
        stats.lines += 1;
    }
    Ok(())
}

struct GrpcSender {
    traces: opentelemetry_proto::tonic::collector::trace::v1::trace_service_client::TraceServiceClient<
        tonic::transport::Channel,
    >,
    logs: opentelemetry_proto::tonic::collector::logs::v1::logs_service_client::LogsServiceClient<
        tonic::transport::Channel,
    >,
    metrics:
        opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_client::MetricsServiceClient<
            tonic::transport::Channel,
        >,
}

impl GrpcSender {
    async fn connect(target: &str) -> anyhow::Result<Self> {
        if target.starts_with("https://") {
            anyhow::bail!("TLS targets are not supported yet; use http://");
        }
        let endpoint = tonic::transport::Endpoint::from_shared(target.to_string())
            .context("invalid target URI")?;
        let channel = endpoint.connect().await.context("connecting to target")?;
        Ok(Self {
            traces: opentelemetry_proto::tonic::collector::trace::v1::trace_service_client::TraceServiceClient::new(
                channel.clone(),
            ),
            logs: opentelemetry_proto::tonic::collector::logs::v1::logs_service_client::LogsServiceClient::new(
                channel.clone(),
            ),
            metrics: opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_client::MetricsServiceClient::new(
                channel,
            ),
        })
    }
}

#[async_trait::async_trait]
impl SignalSender for GrpcSender {
    async fn send(&mut self, sig: &Signal) -> anyhow::Result<()> {
        use opentelemetry_proto::tonic::collector::{logs, metrics, trace};
        match sig {
            Signal::Traces(req) => {
                let req: trace::v1::ExportTraceServiceRequest = req.clone().into();
                self.traces
                    .export(tonic::Request::new(req))
                    .await
                    .map_err(|e| anyhow::anyhow!("gRPC export failed: {e}"))?;
            }
            Signal::Logs(req) => {
                let req: logs::v1::ExportLogsServiceRequest = req.clone().into();
                self.logs
                    .export(tonic::Request::new(req))
                    .await
                    .map_err(|e| anyhow::anyhow!("gRPC export failed: {e}"))?;
            }
            Signal::Metrics(req) => {
                let req: metrics::v1::ExportMetricsServiceRequest = req.clone().into();
                self.metrics
                    .export(tonic::Request::new(req))
                    .await
                    .map_err(|e| anyhow::anyhow!("gRPC export failed: {e}"))?;
            }
        }
        Ok(())
    }
}

struct HttpSender {
    client: hyper_util::client::legacy::Client<
        hyper_util::client::legacy::connect::HttpConnector,
        http_body_util::Full<bytes::Bytes>,
    >,
    base: String,
}

impl HttpSender {
    fn new(target: &str) -> anyhow::Result<Self> {
        if target.starts_with("https://") {
            anyhow::bail!("TLS targets are not supported yet; use http://");
        }
        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();
        Ok(Self {
            client,
            base: target.trim_end_matches('/').to_string(),
        })
    }
}

#[async_trait::async_trait]
impl SignalSender for HttpSender {
    async fn send(&mut self, sig: &Signal) -> anyhow::Result<()> {
        use http_body_util::{BodyExt as _, Full};
        use hyper::body::Bytes;
        use hyper::header::{HeaderValue, CONTENT_TYPE};
        use hyper::{Method, Request as HttpRequest};
        use opentelemetry_proto::tonic::collector::{logs, metrics, trace};
        let (path, body): (&str, Vec<u8>) = match sig {
            Signal::Traces(req) => {
                let r: trace::v1::ExportTraceServiceRequest = req.clone().into();
                ("/v1/traces", prost::Message::encode_to_vec(&r))
            }
            Signal::Logs(req) => {
                let r: logs::v1::ExportLogsServiceRequest = req.clone().into();
                ("/v1/logs", prost::Message::encode_to_vec(&r))
            }
            Signal::Metrics(req) => {
                let r: metrics::v1::ExportMetricsServiceRequest = req.clone().into();
                ("/v1/metrics", prost::Message::encode_to_vec(&r))
            }
        };
        let uri = format!("{}{}", self.base, path);
        let req = HttpRequest::builder()
            .method(Method::POST)
            .uri(&uri)
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-protobuf"),
            )
            .body(Full::new(Bytes::from(body)))
            .context("building HTTP request")?;
        let resp = self
            .client
            .request(req)
            .await
            .context("HTTP export failed")?;
        let status = resp.status();
        if !status.is_success() {
            let mut text = String::new();
            if let Ok(b) = resp.into_body().collect().await {
                text.push_str(&String::from_utf8_lossy(&b.to_bytes()));
            }
            anyhow::bail!("target responded {status}: {}", truncate(&text, 200));
        }
        // Drain the body so the connection can be reused.
        let _ = resp.into_body().collect().await?;
        Ok(())
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dry_run_counts_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cap.jsonl");
        std::fs::write(
            &path,
            concat!(
                r#"{"resourceSpans":[{"scopeSpans":[{"spans":[{"name":"a","traceId":"aa","spanId":"bb"}]}]}]}"#,
                "\n",
                r#"{"resource_logs":[{"scope_logs":[{"log_records":[{"severityText":"INFO"}]}]}]}"#,
                "\n"
            ),
        )
        .unwrap();
        let stats = run(ReplayConfig {
            input: path.to_string_lossy().to_string(),
            target: String::new(),
            speed: 1.0,
            max_speed: false,
            r#loop: false,
            protocol: Protocol::Grpc,
            dry_run: true,
            max_gap_ms: 60_000,
        })
        .await
        .unwrap();
        assert_eq!(stats.lines, 2);
        assert_eq!(stats.spans, 1);
        assert_eq!(stats.log_records, 1);
    }

    #[tokio::test]
    async fn dry_run_rejects_bad_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.jsonl");
        std::fs::write(&path, "{\"unknown\":[]}\n").unwrap();
        let res = run(ReplayConfig {
            input: path.to_string_lossy().to_string(),
            target: String::new(),
            speed: 1.0,
            max_speed: false,
            r#loop: false,
            protocol: Protocol::Grpc,
            dry_run: true,
            max_gap_ms: 60_000,
        })
        .await;
        assert!(res.is_err());
    }
}
