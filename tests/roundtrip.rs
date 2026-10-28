//! End-to-end round trips: gRPC/HTTP ingest → capture → replay.

use otelcat::json::{self, Signal};
use otelcat::server::{BoundAddrs, ServeConfig};
use otelcat::sink::SinkConfig;
use prost::Message as _;
use reqwest::header::{CONTENT_ENCODING, CONTENT_TYPE};
use std::io::Write as _;
use std::time::Duration;
use tokio::sync::oneshot;

use opentelemetry_proto::tonic::collector::logs::v1 as pcollect_logs;
use opentelemetry_proto::tonic::collector::logs::v1::logs_service_client::LogsServiceClient;
use opentelemetry_proto::tonic::collector::metrics::v1 as pcollect_metrics;
use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
use opentelemetry_proto::tonic::collector::trace::v1 as pcollect_trace;
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_client::TraceServiceClient;
use opentelemetry_proto::tonic::common::v1 as pcommon;
use opentelemetry_proto::tonic::logs::v1 as plogs;
use opentelemetry_proto::tonic::metrics::v1 as pmetrics;
use opentelemetry_proto::tonic::resource::v1 as presource;
use opentelemetry_proto::tonic::trace::v1 as ptrace;

struct TestServer {
    addrs: BoundAddrs,
    shutdown_tx: oneshot::Sender<()>,
    capture: std::path::PathBuf,
    task: tokio::task::JoinHandle<anyhow::Result<(BoundAddrs, otelcat::sink::Counts)>>,
    dir: tempfile::TempDir,
}

async fn start_server(json_stdout: bool) -> anyhow::Result<TestServer> {
    let dir = tempfile::tempdir()?;
    let capture = dir.path().join("capture.jsonl");
    let grpc_port = free_port()?;
    let http_port = free_port()?;
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let cfg = ServeConfig {
        grpc_addr: format!("127.0.0.1:{grpc_port}").parse()?,
        http_addr: format!("127.0.0.1:{http_port}").parse()?,
        sink: SinkConfig {
            json: json_stdout,
            out: Some(capture.clone()),
        },
    };
    let task = tokio::spawn(otelcat::server::run(cfg, async {
        let _ = shutdown_rx.await;
    }));
    // Wait until the HTTP port answers.
    let url = format!("http://127.0.0.1:{http_port}/v1/traces");
    let client = reqwest::Client::new();
    for _ in 0..100 {
        if client.get(&url).send().await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Ok(TestServer {
        addrs: BoundAddrs {
            grpc: format!("127.0.0.1:{grpc_port}").parse()?,
            http: format!("127.0.0.1:{http_port}").parse()?,
        },
        shutdown_tx,
        capture,
        task,
        dir,
    })
}

fn free_port() -> anyhow::Result<u16> {
    let l = std::net::TcpListener::bind("127.0.0.1:0")?;
    let port = l.local_addr()?.port();
    Ok(port)
}

impl TestServer {
    async fn stop(self) -> anyhow::Result<otelcat::sink::Counts> {
        let _ = self.shutdown_tx.send(());
        let (_, counts) = self
            .task
            .await
            .map_err(|e| anyhow::anyhow!("server task: {e}"))??;
        Ok(counts)
    }

    async fn capture_lines(&self, expected: usize) -> anyhow::Result<Vec<String>> {
        for _ in 0..200 {
            if let Ok(s) = std::fs::read_to_string(&self.capture) {
                let lines: Vec<String> = s
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(String::from)
                    .collect();
                if lines.len() >= expected {
                    return Ok(lines);
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("capture file never reached {expected} lines");
    }
}

fn sample_trace_request() -> pcollect_trace::ExportTraceServiceRequest {
    pcollect_trace::ExportTraceServiceRequest {
        resource_spans: vec![ptrace::ResourceSpans {
            resource: Some(presource::Resource {
                attributes: vec![pcommon::KeyValue {
                    key: "service.name".into(),
                    value: Some(pcommon::AnyValue {
                        value: Some(pcommon::any_value::Value::StringValue(
                            "checkout-api".into(),
                        )),
                    }),
                    key_strindex: 0,
                }],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_spans: vec![ptrace::ScopeSpans {
                scope: Some(pcommon::InstrumentationScope {
                    name: "otelcat-test".into(),
                    version: "1.0.0".into(),
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }),
                spans: vec![ptrace::Span {
                    trace_id: vec![0x5b; 16],
                    span_id: vec![0xee; 8],
                    trace_state: "k=v".into(),
                    parent_span_id: vec![],
                    flags: 0x100,
                    name: "POST /orders".into(),
                    kind: 2,
                    start_time_unix_nano: 1_544_712_660_000_000_000,
                    end_time_unix_nano: 1_544_712_660_012_500_000,
                    attributes: vec![
                        pcommon::KeyValue {
                            key: "http.method".into(),
                            value: Some(pcommon::AnyValue {
                                value: Some(pcommon::any_value::Value::StringValue("POST".into())),
                            }),
                            key_strindex: 0,
                        },
                        pcommon::KeyValue {
                            key: "order.count".into(),
                            value: Some(pcommon::AnyValue {
                                value: Some(pcommon::any_value::Value::IntValue(3)),
                            }),
                            key_strindex: 0,
                        },
                    ],
                    dropped_attributes_count: 0,
                    events: vec![ptrace::span::Event {
                        time_unix_nano: 1_544_712_660_006_000_000,
                        name: "cached".into(),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                    }],
                    dropped_events_count: 0,
                    links: vec![ptrace::span::Link {
                        trace_id: vec![0x01; 16],
                        span_id: vec![0x02; 8],
                        trace_state: String::new(),
                        attributes: vec![],
                        dropped_attributes_count: 0,
                        flags: 0,
                    }],
                    dropped_links_count: 0,
                    status: Some(ptrace::Status {
                        message: String::new(),
                        code: 1,
                    }),
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}

fn sample_logs_request() -> pcollect_logs::ExportLogsServiceRequest {
    pcollect_logs::ExportLogsServiceRequest {
        resource_logs: vec![plogs::ResourceLogs {
            resource: Some(presource::Resource {
                attributes: vec![pcommon::KeyValue {
                    key: "service.name".into(),
                    value: Some(pcommon::AnyValue {
                        value: Some(pcommon::any_value::Value::StringValue(
                            "checkout-api".into(),
                        )),
                    }),
                    key_strindex: 0,
                }],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_logs: vec![plogs::ScopeLogs {
                scope: Some(pcommon::InstrumentationScope {
                    name: "otelcat-test".into(),
                    version: String::new(),
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }),
                log_records: vec![plogs::LogRecord {
                    time_unix_nano: 1_544_712_660_000_000_000,
                    observed_time_unix_nano: 1_544_712_660_001_000_000,
                    severity_number: 17,
                    severity_text: "DEBUG".into(),
                    body: Some(pcommon::AnyValue {
                        value: Some(pcommon::any_value::Value::StringValue("cache hit".into())),
                    }),
                    attributes: vec![pcommon::KeyValue {
                        key: "cache.hit".into(),
                        value: Some(pcommon::AnyValue {
                            value: Some(pcommon::any_value::Value::BoolValue(true)),
                        }),
                        key_strindex: 0,
                    }],
                    dropped_attributes_count: 0,
                    flags: 0,
                    trace_id: vec![0x5b; 16],
                    span_id: vec![0xee; 8],
                    event_name: String::new(),
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}

fn sample_metrics_request() -> pcollect_metrics::ExportMetricsServiceRequest {
    pcollect_metrics::ExportMetricsServiceRequest {
        resource_metrics: vec![pmetrics::ResourceMetrics {
            resource: Some(presource::Resource {
                attributes: vec![pcommon::KeyValue {
                    key: "service.name".into(),
                    value: Some(pcommon::AnyValue {
                        value: Some(pcommon::any_value::Value::StringValue(
                            "checkout-api".into(),
                        )),
                    }),
                    key_strindex: 0,
                }],
                dropped_attributes_count: 0,
                entity_refs: vec![],
            }),
            scope_metrics: vec![pmetrics::ScopeMetrics {
                scope: Some(pcommon::InstrumentationScope {
                    name: "otelcat-test".into(),
                    version: String::new(),
                    attributes: vec![],
                    dropped_attributes_count: 0,
                }),
                metrics: vec![pmetrics::Metric {
                    name: "orders.total".into(),
                    description: String::new(),
                    unit: "1".into(),
                    metadata: vec![],
                    data: Some(pmetrics::metric::Data::Sum(pmetrics::Sum {
                        data_points: vec![pmetrics::NumberDataPoint {
                            attributes: vec![],
                            start_time_unix_nano: 1_544_712_660_000_000_000,
                            time_unix_nano: 1_544_712_660_012_500_000,
                            exemplars: vec![],
                            flags: 0,
                            value: Some(pmetrics::number_data_point::Value::AsInt(42)),
                        }],
                        aggregation_temporality: 2,
                        is_monotonic: true,
                    })),
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}

#[tokio::test]
async fn grpc_ingest_captures_all_signals() -> anyhow::Result<()> {
    let server = start_server(false).await?;
    let mut traces = TraceServiceClient::connect(format!("http://{}", server.addrs.grpc)).await?;
    let mut logs_client =
        LogsServiceClient::connect(format!("http://{}", server.addrs.grpc)).await?;
    let mut metrics_client =
        MetricsServiceClient::connect(format!("http://{}", server.addrs.grpc)).await?;

    traces.export(sample_trace_request()).await?;
    logs_client.export(sample_logs_request()).await?;
    metrics_client.export(sample_metrics_request()).await?;

    let lines = server.capture_lines(3).await?;
    let trace_sig = Signal::Traces(sample_trace_request().into());
    let logs_sig = Signal::Logs(sample_logs_request().into());
    let metrics_sig = Signal::Metrics(sample_metrics_request().into());
    let expected: Vec<String> = [trace_sig, logs_sig, metrics_sig]
        .iter()
        .map(|s| s.to_line())
        .collect();
    // The order between different signals is not guaranteed.
    for e in &expected {
        assert!(
            lines.iter().any(|l| l == e),
            "capture missing line:\n{e}\n---\ngot:\n{}",
            lines.join("\n---\n")
        );
    }
    let counts = server.stop().await?;
    assert_eq!(counts.spans, 1);
    assert_eq!(counts.log_records, 1);
    assert_eq!(counts.metric_points, 1);
    Ok(())
}

#[tokio::test]
async fn http_proto_and_json_ingest() -> anyhow::Result<()> {
    let server = start_server(false).await?;
    let client = reqwest::Client::new();
    let base = format!("http://{}", server.addrs.http);

    // protobuf traces
    let resp = client
        .post(format!("{base}/v1/traces"))
        .header(CONTENT_TYPE, "application/x-protobuf")
        .body(sample_trace_request().encode_to_vec())
        .send()
        .await?;
    assert!(resp.status().is_success(), "status {}", resp.status());

    // camelCase JSON logs (as produced by standard protojson)
    let logs_json: json::ExportLogsServiceRequest = sample_logs_request().into();
    let resp = client
        .post(format!("{base}/v1/logs"))
        .header(CONTENT_TYPE, "application/json")
        .body(Signal::Logs(logs_json).to_line())
        .send()
        .await?;
    assert!(resp.status().is_success(), "status {}", resp.status());

    // gzip metrics
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(sample_metrics_request().encode_to_vec().as_slice())?;
    let gz = enc.finish()?;
    let resp = client
        .post(format!("{base}/v1/metrics"))
        .header(CONTENT_TYPE, "application/x-protobuf")
        .header(CONTENT_ENCODING, "gzip")
        .body(gz)
        .send()
        .await?;
    assert!(resp.status().is_success(), "status {}", resp.status());

    let lines = server.capture_lines(3).await?;
    assert_eq!(lines.len(), 3);

    // bad payload → 400
    let resp = client
        .post(format!("{base}/v1/traces"))
        .body(b"garbage".to_vec())
        .send()
        .await?;
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);

    server.stop().await?;
    Ok(())
}

#[tokio::test]
async fn replay_roundtrip_preserves_content() -> anyhow::Result<()> {
    let source = start_server(false).await?;
    let target = start_server(false).await?;

    let mut client = TraceServiceClient::connect(format!("http://{}", source.addrs.grpc)).await?;
    client.export(sample_trace_request()).await?;

    let lines = source.capture_lines(1).await?;
    assert_eq!(lines.len(), 1);
    let cap_path = source.dir.path().join("replay_input.jsonl");
    std::fs::write(&cap_path, format!("{}\n", lines[0]))?;

    let stats = otelcat::replay::run(otelcat::replay::ReplayConfig {
        input: cap_path.to_string_lossy().to_string(),
        target: format!("http://{}", target.addrs.grpc),
        speed: 1.0,
        max_speed: true,
        r#loop: false,
        protocol: otelcat::replay::Protocol::Grpc,
        dry_run: false,
        max_gap_ms: 60_000,
    })
    .await?;
    assert_eq!(stats.lines, 1);
    assert_eq!(stats.spans, 1);

    let target_lines = target.capture_lines(1).await?;
    assert_eq!(
        target_lines[0], lines[0],
        "replayed content must be byte-identical"
    );

    source.stop().await?;
    target.stop().await?;
    Ok(())
}

#[tokio::test]
async fn replay_http_roundtrip_preserves_content() -> anyhow::Result<()> {
    let source = start_server(false).await?;
    let target = start_server(false).await?;

    let mut client = TraceServiceClient::connect(format!("http://{}", source.addrs.grpc)).await?;
    client.export(sample_trace_request()).await?;

    let lines = source.capture_lines(1).await?;
    let cap_path = source.dir.path().join("replay_http_input.jsonl");
    std::fs::write(&cap_path, format!("{}\n", lines[0]))?;

    let stats = otelcat::replay::run(otelcat::replay::ReplayConfig {
        input: cap_path.to_string_lossy().to_string(),
        target: format!("http://{}", target.addrs.http),
        speed: 1.0,
        max_speed: true,
        r#loop: false,
        protocol: otelcat::replay::Protocol::Http,
        dry_run: false,
        max_gap_ms: 60_000,
    })
    .await?;
    assert_eq!(stats.lines, 1);
    assert_eq!(stats.spans, 1);

    let target_lines = target.capture_lines(1).await?;
    assert_eq!(
        target_lines[0], lines[0],
        "HTTP replay must be byte-identical"
    );

    source.stop().await?;
    target.stop().await?;
    Ok(())
}

#[tokio::test]
async fn json_stdout_mode() -> anyhow::Result<()> {
    let server = start_server(true).await?;
    let mut client = TraceServiceClient::connect(format!("http://{}", server.addrs.grpc)).await?;
    client.export(sample_trace_request()).await?;
    let _ = server.capture_lines(1).await?;
    server.stop().await?;
    Ok(())
}
