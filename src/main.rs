//! otelcat — kcat for OpenTelemetry.

use clap::Parser as _;
use otelcat::cli::{Cli, Command};
use std::net::{IpAddr, SocketAddr};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("otelcat=warn")),
        )
        .init();

    match Cli::parse().command {
        Command::Serve {
            bind,
            grpc_port,
            http_port,
            out,
            json,
        } => {
            let host: IpAddr = bind
                .parse()
                .map_err(|_| anyhow::anyhow!("invalid --bind: {bind}"))?;
            let cfg = otelcat::server::ServeConfig {
                grpc_addr: SocketAddr::new(host, grpc_port),
                http_addr: SocketAddr::new(host, http_port),
                sink: otelcat::sink::SinkConfig { json, out },
            };
            let (_, counts) = otelcat::server::run(cfg, async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await?;
            eprintln!(
                "captured {} request(s): {} spans, {} log records, {} metric points",
                counts.requests, counts.spans, counts.log_records, counts.metric_points
            );
        }
        Command::Replay {
            input,
            to,
            speed,
            max,
            r#loop,
            protocol,
            dry_run,
            max_gap_ms,
        } => {
            if !dry_run && to.is_none() {
                anyhow::bail!("--to is required unless --dry-run is set");
            }
            let cfg = otelcat::replay::ReplayConfig {
                input,
                target: to.unwrap_or_default(),
                speed: if max { f64::INFINITY } else { speed },
                max_speed: max,
                r#loop,
                protocol,
                dry_run,
                max_gap_ms,
            };
            let stats = otelcat::replay::run(cfg).await?;
            if dry_run {
                eprintln!(
                    "capture OK: {} line(s), {} spans, {} log records, {} metric points",
                    stats.lines, stats.spans, stats.log_records, stats.metric_points
                );
            } else {
                eprintln!(
                    "replayed {} line(s) in {} loop(s): {} items at {:.0} items/s",
                    stats.lines,
                    stats.loops,
                    stats.total_items(),
                    stats.items_per_sec()
                );
            }
        }
    }
    Ok(())
}
