//! OTLP receiver: gRPC + HTTP endpoints feeding a sink.

use crate::json::{self, Signal};
use crate::printer;
use crate::sink::{self, SinkConfig, SinkMsg};
use anyhow::Context as _;
use opentelemetry_proto::tonic::collector::logs::v1::logs_service_server::{
    LogsService, LogsServiceServer,
};
use opentelemetry_proto::tonic::collector::logs::v1::{
    ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::{
    MetricsService, MetricsServiceServer,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::{
    TraceService, TraceServiceServer,
};
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use prost::Message as _;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

#[derive(Clone, Debug)]
pub struct ServeConfig {
    pub grpc_addr: SocketAddr,
    pub http_addr: SocketAddr,
    pub sink: SinkConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundAddrs {
    pub grpc: SocketAddr,
    pub http: SocketAddr,
}

#[derive(Clone)]
struct Receiver {
    tx: mpsc::Sender<SinkMsg>,
}

impl Receiver {
    async fn forward(&self, sig: Signal) -> Result<(), Status> {
        self.tx
            .send(SinkMsg::Signal(sig))
            .await
            .map_err(|_| Status::unavailable("sink is shutting down"))
    }
}

macro_rules! impl_export_service {
    ($trait_name:ident, $req:ident, $resp:ident, $variant:ident) => {
        #[async_trait::async_trait]
        impl $trait_name for Receiver {
            async fn export(&self, request: Request<$req>) -> Result<Response<$resp>, Status> {
                let inner = request.into_inner();
                self.forward(Signal::$variant(inner.into())).await?;
                Ok(Response::new($resp {
                    partial_success: None,
                }))
            }
        }
    };
}

impl_export_service!(
    TraceService,
    ExportTraceServiceRequest,
    ExportTraceServiceResponse,
    Traces
);
impl_export_service!(
    LogsService,
    ExportLogsServiceRequest,
    ExportLogsServiceResponse,
    Logs
);
impl_export_service!(
    MetricsService,
    ExportMetricsServiceRequest,
    ExportMetricsServiceResponse,
    Metrics
);

/// Runs both receivers until the shutdown future resolves or one of them
/// fails. Returns the bound addresses and item counts seen by the sink.
pub async fn run(
    cfg: ServeConfig,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> anyhow::Result<(BoundAddrs, sink::Counts)> {
    let style = if cfg.sink.json {
        printer::Style::plain()
    } else {
        printer::Style::auto()
    };
    let (tx, rx) = mpsc::channel::<SinkMsg>(1024);
    let sink_task = tokio::spawn(sink::run(rx, cfg.sink.clone(), style));

    let receiver = Receiver { tx: tx.clone() };
    let grpc_router = tonic::transport::Server::builder()
        .add_service(
            TraceServiceServer::new(receiver.clone())
                .accept_compressed(tonic::codec::CompressionEncoding::Gzip),
        )
        .add_service(
            LogsServiceServer::new(receiver.clone())
                .accept_compressed(tonic::codec::CompressionEncoding::Gzip),
        )
        .add_service(
            MetricsServiceServer::new(receiver.clone())
                .accept_compressed(tonic::codec::CompressionEncoding::Gzip),
        );

    let http_router = build_http_router(tx.clone());

    // Bind HTTP first to surface port conflicts early and learn the resolved address.
    let http_listener = tokio::net::TcpListener::bind(cfg.http_addr)
        .await
        .with_context(|| format!("binding OTLP/HTTP on {}", cfg.http_addr))?;
    let http_bound = http_listener.local_addr()?;

    // Probe the gRPC port for a clear error before tonic takes it.
    {
        let probe = tokio::net::TcpListener::bind(cfg.grpc_addr)
            .await
            .with_context(|| format!("binding OTLP/gRPC on {}", cfg.grpc_addr))?;
        drop(probe);
    }

    eprintln!(
        "otelcat listening on OTLP/gRPC {} and OTLP/HTTP {} (ctrl-c to stop)",
        cfg.grpc_addr, http_bound
    );

    let bound = BoundAddrs {
        grpc: cfg.grpc_addr,
        http: http_bound,
    };

    tokio::select! {
        r = grpc_router.serve(cfg.grpc_addr) => r.context("OTLP/gRPC server failed")?,
        r = axum::serve(http_listener, http_router) => r.context("OTLP/HTTP server failed")?,
        _ = shutdown => {
            eprintln!("shutting down…");
        }
    }

    // Connection tasks may keep service clones (and their Sender) alive, so
    // the sink is closed explicitly instead of relying on channel drain.
    let _ = tx.send(SinkMsg::Close).await;
    let counts = sink_task.await.context("sink task failed")??;
    Ok((bound, counts))
}

fn build_http_router(tx: mpsc::Sender<SinkMsg>) -> axum::Router {
    use axum::extract::State;
    use axum::http::{header, HeaderMap, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::post;
    use axum::Router;
    use bytes::Bytes;

    #[derive(Clone, Copy)]
    enum Kind {
        Traces,
        Logs,
        Metrics,
    }

    async fn handle(
        tx: mpsc::Sender<SinkMsg>,
        kind: Kind,
        headers: HeaderMap,
        body: Bytes,
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        let payload =
            decode_body(&headers, &body).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        let sig =
            parse_payload(kind, &payload).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
        tx.send(SinkMsg::Signal(sig))
            .await
            .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "shutting down".to_string()))?;
        Ok((
            [(header::CONTENT_TYPE, "application/x-protobuf")],
            Bytes::new(),
        ))
    }

    fn decode_body(headers: &HeaderMap, body: &Bytes) -> anyhow::Result<Vec<u8>> {
        let enc = headers
            .get(header::CONTENT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if enc.eq_ignore_ascii_case("gzip") {
            let mut out = Vec::with_capacity(body.len() * 2);
            let mut decoder = flate2::read::GzDecoder::new(body.as_ref());
            std::io::Read::read_to_end(&mut decoder, &mut out).context("gzip decode")?;
            Ok(out)
        } else if enc.is_empty() || enc.eq_ignore_ascii_case("identity") {
            Ok(body.to_vec())
        } else {
            anyhow::bail!("unsupported content-encoding: {enc}")
        }
    }

    fn parse_payload(kind: Kind, payload: &[u8]) -> anyhow::Result<Signal> {
        let ct = match kind {
            Kind::Traces => "traces",
            Kind::Logs => "logs",
            Kind::Metrics => "metrics",
        };
        // Try JSON first (camelCase or snake_case), then protobuf.
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(payload) {
            return Signal::from_json_value(v).context("invalid OTLP/JSON payload");
        }
        match kind {
            Kind::Traces => {
                let req: json::ExportTraceServiceRequest = opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest::decode(payload)
                    .context("invalid protobuf")?
                    .into();
                Ok(Signal::Traces(req))
            }
            Kind::Logs => {
                let req: json::ExportLogsServiceRequest = opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest::decode(payload)
                    .context("invalid protobuf")?
                    .into();
                Ok(Signal::Logs(req))
            }
            Kind::Metrics => {
                let req: json::ExportMetricsServiceRequest = opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest::decode(payload)
                    .context("invalid protobuf")?
                    .into();
                Ok(Signal::Metrics(req))
            }
        }
        .map_err(|e: anyhow::Error| e.context(format!("while parsing {ct} payload")))
    }

    async fn traces(
        State(tx): State<mpsc::Sender<SinkMsg>>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        handle(tx, Kind::Traces, headers, body).await
    }

    async fn logs(
        State(tx): State<mpsc::Sender<SinkMsg>>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        handle(tx, Kind::Logs, headers, body).await
    }

    async fn metrics(
        State(tx): State<mpsc::Sender<SinkMsg>>,
        headers: HeaderMap,
        body: Bytes,
    ) -> Result<impl IntoResponse, (StatusCode, String)> {
        handle(tx, Kind::Metrics, headers, body).await
    }

    Router::new()
        .route("/v1/traces", post(traces))
        .route("/v1/logs", post(logs))
        .route("/v1/metrics", post(metrics))
        .with_state(tx)
}
