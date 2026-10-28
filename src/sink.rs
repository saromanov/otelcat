//! Sink: consumes signals, prints them and/or writes a JSONL capture file.

use crate::json::Signal;
use crate::printer::{self, Style};
use std::io::Write as _;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt as _;
use tokio::sync::mpsc;

/// Messages fed into the sink.
pub enum SinkMsg {
    Signal(Signal),
    Close,
}

#[derive(Clone, Debug, Default)]
pub struct SinkConfig {
    /// Emit OTLP/JSON lines to stdout instead of the pretty format.
    pub json: bool,
    /// Write a JSONL capture file (always OTLP/JSON).
    pub out: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub requests: u64,
    pub spans: u64,
    pub log_records: u64,
    pub metric_points: u64,
}

impl Counts {
    pub fn total(&self) -> u64 {
        self.spans + self.log_records + self.metric_points
    }
}

pub async fn run(
    mut rx: mpsc::Receiver<SinkMsg>,
    cfg: SinkConfig,
    style: Style,
) -> anyhow::Result<Counts> {
    let mut counts = Counts::default();
    let mut file = match &cfg.out {
        Some(path) => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    tokio::fs::create_dir_all(parent).await?;
                }
            }
            let f = tokio::fs::File::create(path).await?;
            Some(tokio::io::BufWriter::new(f))
        }
        None => None,
    };

    let stdout = std::io::stdout();
    while let Some(msg) = rx.recv().await {
        let sig = match msg {
            SinkMsg::Signal(sig) => sig,
            SinkMsg::Close => break,
        };
        counts.requests += 1;
        let (spans, records, points) = count(&sig);
        counts.spans += spans;
        counts.log_records += records;
        counts.metric_points += points;

        if let Some(f) = file.as_mut() {
            let line = sig.to_line();
            f.write_all(line.as_bytes()).await?;
            f.write_all(b"\n").await?;
            f.flush().await?;
        }

        if cfg.json {
            let line = sig.to_line();
            let mut out = stdout.lock();
            let _ = out.write_all(line.as_bytes());
            let _ = out.write_all(b"\n");
            let _ = out.flush();
        } else {
            let mut out = stdout.lock();
            printer::print_signal(&mut out, &sig, &style);
            let _ = out.flush();
        }
    }
    if let Some(f) = file.as_mut() {
        f.flush().await?;
    }
    Ok(counts)
}

/// Per-signal item counts: (spans, log records, metric points).
pub fn count(sig: &Signal) -> (u64, u64, u64) {
    match sig {
        Signal::Traces(t) => (
            t.resource_spans
                .iter()
                .flat_map(|rs| rs.scope_spans.iter())
                .map(|ss| ss.spans.len() as u64)
                .sum(),
            0,
            0,
        ),
        Signal::Logs(l) => (
            0,
            l.resource_logs
                .iter()
                .flat_map(|rl| rl.scope_logs.iter())
                .map(|sl| sl.log_records.len() as u64)
                .sum(),
            0,
        ),
        Signal::Metrics(m) => {
            let mut points = 0u64;
            for rm in &m.resource_metrics {
                for sm in &rm.scope_metrics {
                    for metric in &sm.metrics {
                        if let Some(data) = &metric.data {
                            points += match data {
                                crate::json::MetricData::Gauge(g) => g.data_points.len() as u64,
                                crate::json::MetricData::Sum(s) => s.data_points.len() as u64,
                                crate::json::MetricData::Histogram(h) => h.data_points.len() as u64,
                                crate::json::MetricData::ExponentialHistogram(e) => {
                                    e.data_points.len() as u64
                                }
                                crate::json::MetricData::Summary(s) => s.data_points.len() as u64,
                            };
                        }
                    }
                }
            }
            (0, 0, points)
        }
    }
}
