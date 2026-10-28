//! metrics/v1 conversions.

use super::vec_conv;
use crate::json::metrics as jmetrics;
use opentelemetry_proto::tonic::collector::metrics::v1 as pcollect;
use opentelemetry_proto::tonic::metrics::v1 as pmetrics;

impl From<jmetrics::ExportMetricsServiceRequest> for pcollect::ExportMetricsServiceRequest {
    fn from(v: jmetrics::ExportMetricsServiceRequest) -> Self {
        pcollect::ExportMetricsServiceRequest {
            resource_metrics: vec_conv(v.resource_metrics),
        }
    }
}

impl From<pcollect::ExportMetricsServiceRequest> for jmetrics::ExportMetricsServiceRequest {
    fn from(v: pcollect::ExportMetricsServiceRequest) -> Self {
        jmetrics::ExportMetricsServiceRequest {
            resource_metrics: vec_conv(v.resource_metrics),
        }
    }
}

impl From<jmetrics::ResourceMetrics> for pmetrics::ResourceMetrics {
    fn from(v: jmetrics::ResourceMetrics) -> Self {
        pmetrics::ResourceMetrics {
            resource: v
                .resource
                .map(opentelemetry_proto::tonic::resource::v1::Resource::from),
            scope_metrics: vec_conv(v.scope_metrics),
            schema_url: v.schema_url,
        }
    }
}

impl From<pmetrics::ResourceMetrics> for jmetrics::ResourceMetrics {
    fn from(v: pmetrics::ResourceMetrics) -> Self {
        jmetrics::ResourceMetrics {
            resource: v.resource.map(crate::json::common::Resource::from),
            scope_metrics: vec_conv(v.scope_metrics),
            schema_url: v.schema_url,
        }
    }
}

impl From<jmetrics::ScopeMetrics> for pmetrics::ScopeMetrics {
    fn from(v: jmetrics::ScopeMetrics) -> Self {
        pmetrics::ScopeMetrics {
            scope: v
                .scope
                .map(opentelemetry_proto::tonic::common::v1::InstrumentationScope::from),
            metrics: vec_conv(v.metrics),
            schema_url: v.schema_url,
        }
    }
}

impl From<pmetrics::ScopeMetrics> for jmetrics::ScopeMetrics {
    fn from(v: pmetrics::ScopeMetrics) -> Self {
        jmetrics::ScopeMetrics {
            scope: v.scope.map(crate::json::common::InstrumentationScope::from),
            metrics: vec_conv(v.metrics),
            schema_url: v.schema_url,
        }
    }
}

impl From<jmetrics::Metric> for pmetrics::Metric {
    fn from(v: jmetrics::Metric) -> Self {
        pmetrics::Metric {
            name: v.name,
            description: v.description,
            unit: v.unit,
            metadata: vec_conv(v.metadata),
            data: v.data.map(pmetrics::metric::Data::from),
        }
    }
}

impl From<pmetrics::Metric> for jmetrics::Metric {
    fn from(v: pmetrics::Metric) -> Self {
        jmetrics::Metric {
            name: v.name,
            description: v.description,
            unit: v.unit,
            metadata: vec_conv(v.metadata),
            data: v.data.map(jmetrics::MetricData::from),
        }
    }
}

impl From<jmetrics::MetricData> for pmetrics::metric::Data {
    fn from(v: jmetrics::MetricData) -> Self {
        match v {
            jmetrics::MetricData::Gauge(g) => pmetrics::metric::Data::Gauge(g.into()),
            jmetrics::MetricData::Sum(s) => pmetrics::metric::Data::Sum(s.into()),
            jmetrics::MetricData::Histogram(h) => pmetrics::metric::Data::Histogram(h.into()),
            jmetrics::MetricData::ExponentialHistogram(e) => {
                pmetrics::metric::Data::ExponentialHistogram(e.into())
            }
            jmetrics::MetricData::Summary(s) => pmetrics::metric::Data::Summary(s.into()),
        }
    }
}

impl From<pmetrics::metric::Data> for jmetrics::MetricData {
    fn from(v: pmetrics::metric::Data) -> Self {
        match v {
            pmetrics::metric::Data::Gauge(g) => jmetrics::MetricData::Gauge(g.into()),
            pmetrics::metric::Data::Sum(s) => jmetrics::MetricData::Sum(s.into()),
            pmetrics::metric::Data::Histogram(h) => jmetrics::MetricData::Histogram(h.into()),
            pmetrics::metric::Data::ExponentialHistogram(e) => {
                jmetrics::MetricData::ExponentialHistogram(e.into())
            }
            pmetrics::metric::Data::Summary(s) => jmetrics::MetricData::Summary(s.into()),
        }
    }
}

impl From<jmetrics::Gauge> for pmetrics::Gauge {
    fn from(v: jmetrics::Gauge) -> Self {
        pmetrics::Gauge {
            data_points: vec_conv(v.data_points),
        }
    }
}

impl From<pmetrics::Gauge> for jmetrics::Gauge {
    fn from(v: pmetrics::Gauge) -> Self {
        jmetrics::Gauge {
            data_points: vec_conv(v.data_points),
        }
    }
}

impl From<jmetrics::Sum> for pmetrics::Sum {
    fn from(v: jmetrics::Sum) -> Self {
        pmetrics::Sum {
            data_points: vec_conv(v.data_points),
            aggregation_temporality: v.aggregation_temporality,
            is_monotonic: v.is_monotonic,
        }
    }
}

impl From<pmetrics::Sum> for jmetrics::Sum {
    fn from(v: pmetrics::Sum) -> Self {
        jmetrics::Sum {
            data_points: vec_conv(v.data_points),
            aggregation_temporality: v.aggregation_temporality,
            is_monotonic: v.is_monotonic,
        }
    }
}

impl From<jmetrics::Histogram> for pmetrics::Histogram {
    fn from(v: jmetrics::Histogram) -> Self {
        pmetrics::Histogram {
            data_points: vec_conv(v.data_points),
            aggregation_temporality: v.aggregation_temporality,
        }
    }
}

impl From<pmetrics::Histogram> for jmetrics::Histogram {
    fn from(v: pmetrics::Histogram) -> Self {
        jmetrics::Histogram {
            data_points: vec_conv(v.data_points),
            aggregation_temporality: v.aggregation_temporality,
        }
    }
}

impl From<jmetrics::ExponentialHistogram> for pmetrics::ExponentialHistogram {
    fn from(v: jmetrics::ExponentialHistogram) -> Self {
        pmetrics::ExponentialHistogram {
            data_points: vec_conv(v.data_points),
            aggregation_temporality: v.aggregation_temporality,
        }
    }
}

impl From<pmetrics::ExponentialHistogram> for jmetrics::ExponentialHistogram {
    fn from(v: pmetrics::ExponentialHistogram) -> Self {
        jmetrics::ExponentialHistogram {
            data_points: vec_conv(v.data_points),
            aggregation_temporality: v.aggregation_temporality,
        }
    }
}

impl From<jmetrics::Summary> for pmetrics::Summary {
    fn from(v: jmetrics::Summary) -> Self {
        pmetrics::Summary {
            data_points: vec_conv(v.data_points),
        }
    }
}

impl From<pmetrics::Summary> for jmetrics::Summary {
    fn from(v: pmetrics::Summary) -> Self {
        jmetrics::Summary {
            data_points: vec_conv(v.data_points),
        }
    }
}

impl From<jmetrics::NumberDataPoint> for pmetrics::NumberDataPoint {
    fn from(v: jmetrics::NumberDataPoint) -> Self {
        pmetrics::NumberDataPoint {
            attributes: vec_conv(v.attributes),
            start_time_unix_nano: v.start_time_unix_nano,
            time_unix_nano: v.time_unix_nano,
            exemplars: vec_conv(v.exemplars),
            flags: v.flags,
            value: v.value.map(pmetrics::number_data_point::Value::from),
        }
    }
}

impl From<pmetrics::NumberDataPoint> for jmetrics::NumberDataPoint {
    fn from(v: pmetrics::NumberDataPoint) -> Self {
        jmetrics::NumberDataPoint {
            attributes: vec_conv(v.attributes),
            start_time_unix_nano: v.start_time_unix_nano,
            time_unix_nano: v.time_unix_nano,
            exemplars: vec_conv(v.exemplars),
            flags: v.flags,
            value: v.value.map(jmetrics::PointValue::from),
        }
    }
}

impl From<jmetrics::PointValue> for pmetrics::number_data_point::Value {
    fn from(v: jmetrics::PointValue) -> Self {
        match v {
            jmetrics::PointValue::AsDouble(d) => pmetrics::number_data_point::Value::AsDouble(d),
            jmetrics::PointValue::AsInt(i) => pmetrics::number_data_point::Value::AsInt(i),
        }
    }
}

impl From<pmetrics::number_data_point::Value> for jmetrics::PointValue {
    fn from(v: pmetrics::number_data_point::Value) -> Self {
        match v {
            pmetrics::number_data_point::Value::AsDouble(d) => jmetrics::PointValue::AsDouble(d),
            pmetrics::number_data_point::Value::AsInt(i) => jmetrics::PointValue::AsInt(i),
        }
    }
}

impl From<jmetrics::PointValue> for pmetrics::exemplar::Value {
    fn from(v: jmetrics::PointValue) -> Self {
        match v {
            jmetrics::PointValue::AsDouble(d) => pmetrics::exemplar::Value::AsDouble(d),
            jmetrics::PointValue::AsInt(i) => pmetrics::exemplar::Value::AsInt(i),
        }
    }
}

impl From<pmetrics::exemplar::Value> for jmetrics::PointValue {
    fn from(v: pmetrics::exemplar::Value) -> Self {
        match v {
            pmetrics::exemplar::Value::AsDouble(d) => jmetrics::PointValue::AsDouble(d),
            pmetrics::exemplar::Value::AsInt(i) => jmetrics::PointValue::AsInt(i),
        }
    }
}

impl From<jmetrics::Exemplar> for pmetrics::Exemplar {
    fn from(v: jmetrics::Exemplar) -> Self {
        pmetrics::Exemplar {
            filtered_attributes: vec_conv(v.filtered_attributes),
            time_unix_nano: v.time_unix_nano,
            value: v.value.map(pmetrics::exemplar::Value::from),
            trace_id: v.trace_id,
            span_id: v.span_id,
        }
    }
}

impl From<pmetrics::Exemplar> for jmetrics::Exemplar {
    fn from(v: pmetrics::Exemplar) -> Self {
        jmetrics::Exemplar {
            filtered_attributes: vec_conv(v.filtered_attributes),
            time_unix_nano: v.time_unix_nano,
            value: v.value.map(jmetrics::PointValue::from),
            trace_id: v.trace_id,
            span_id: v.span_id,
        }
    }
}

impl From<jmetrics::HistogramDataPoint> for pmetrics::HistogramDataPoint {
    fn from(v: jmetrics::HistogramDataPoint) -> Self {
        pmetrics::HistogramDataPoint {
            attributes: vec_conv(v.attributes),
            start_time_unix_nano: v.start_time_unix_nano,
            time_unix_nano: v.time_unix_nano,
            count: v.count,
            sum: v.sum,
            bucket_counts: v.bucket_counts,
            explicit_bounds: v.explicit_bounds,
            exemplars: vec_conv(v.exemplars),
            flags: v.flags,
            min: v.min,
            max: v.max,
        }
    }
}

impl From<pmetrics::HistogramDataPoint> for jmetrics::HistogramDataPoint {
    fn from(v: pmetrics::HistogramDataPoint) -> Self {
        jmetrics::HistogramDataPoint {
            attributes: vec_conv(v.attributes),
            start_time_unix_nano: v.start_time_unix_nano,
            time_unix_nano: v.time_unix_nano,
            count: v.count,
            sum: v.sum,
            bucket_counts: v.bucket_counts,
            explicit_bounds: v.explicit_bounds,
            exemplars: vec_conv(v.exemplars),
            flags: v.flags,
            min: v.min,
            max: v.max,
        }
    }
}

impl From<jmetrics::ExponentialHistogramDataPoint> for pmetrics::ExponentialHistogramDataPoint {
    fn from(v: jmetrics::ExponentialHistogramDataPoint) -> Self {
        pmetrics::ExponentialHistogramDataPoint {
            attributes: vec_conv(v.attributes),
            start_time_unix_nano: v.start_time_unix_nano,
            time_unix_nano: v.time_unix_nano,
            count: v.count,
            sum: v.sum,
            scale: v.scale,
            zero_count: v.zero_count,
            positive: v
                .positive
                .map(pmetrics::exponential_histogram_data_point::Buckets::from),
            negative: v
                .negative
                .map(pmetrics::exponential_histogram_data_point::Buckets::from),
            exemplars: vec_conv(v.exemplars),
            flags: v.flags,
            min: v.min,
            max: v.max,
            zero_threshold: v.zero_threshold,
        }
    }
}

impl From<pmetrics::ExponentialHistogramDataPoint> for jmetrics::ExponentialHistogramDataPoint {
    fn from(v: pmetrics::ExponentialHistogramDataPoint) -> Self {
        jmetrics::ExponentialHistogramDataPoint {
            attributes: vec_conv(v.attributes),
            start_time_unix_nano: v.start_time_unix_nano,
            time_unix_nano: v.time_unix_nano,
            count: v.count,
            sum: v.sum,
            scale: v.scale,
            zero_count: v.zero_count,
            positive: v.positive.map(jmetrics::Buckets::from),
            negative: v.negative.map(jmetrics::Buckets::from),
            exemplars: vec_conv(v.exemplars),
            flags: v.flags,
            min: v.min,
            max: v.max,
            zero_threshold: v.zero_threshold,
        }
    }
}

impl From<jmetrics::Buckets> for pmetrics::exponential_histogram_data_point::Buckets {
    fn from(v: jmetrics::Buckets) -> Self {
        pmetrics::exponential_histogram_data_point::Buckets {
            offset: v.offset,
            bucket_counts: v.bucket_counts,
        }
    }
}

impl From<pmetrics::exponential_histogram_data_point::Buckets> for jmetrics::Buckets {
    fn from(v: pmetrics::exponential_histogram_data_point::Buckets) -> Self {
        jmetrics::Buckets {
            offset: v.offset,
            bucket_counts: v.bucket_counts,
        }
    }
}

impl From<jmetrics::SummaryDataPoint> for pmetrics::SummaryDataPoint {
    fn from(v: jmetrics::SummaryDataPoint) -> Self {
        pmetrics::SummaryDataPoint {
            attributes: vec_conv(v.attributes),
            start_time_unix_nano: v.start_time_unix_nano,
            time_unix_nano: v.time_unix_nano,
            count: v.count,
            sum: v.sum,
            quantile_values: vec_conv(v.quantile_values),
            flags: v.flags,
        }
    }
}

impl From<pmetrics::SummaryDataPoint> for jmetrics::SummaryDataPoint {
    fn from(v: pmetrics::SummaryDataPoint) -> Self {
        jmetrics::SummaryDataPoint {
            attributes: vec_conv(v.attributes),
            start_time_unix_nano: v.start_time_unix_nano,
            time_unix_nano: v.time_unix_nano,
            count: v.count,
            sum: v.sum,
            quantile_values: vec_conv(v.quantile_values),
            flags: v.flags,
        }
    }
}

impl From<jmetrics::QuantileValue> for pmetrics::summary_data_point::ValueAtQuantile {
    fn from(v: jmetrics::QuantileValue) -> Self {
        pmetrics::summary_data_point::ValueAtQuantile {
            quantile: v.quantile,
            value: v.value,
        }
    }
}

impl From<pmetrics::summary_data_point::ValueAtQuantile> for jmetrics::QuantileValue {
    fn from(v: pmetrics::summary_data_point::ValueAtQuantile) -> Self {
        jmetrics::QuantileValue {
            quantile: v.quantile,
            value: v.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::common as jcommon;

    fn sample_metric() -> jmetrics::Metric {
        jmetrics::Metric {
            name: "http.server.duration".into(),
            description: "Duration".into(),
            unit: "ms".into(),
            metadata: vec![],
            data: Some(jmetrics::MetricData::Histogram(jmetrics::Histogram {
                aggregation_temporality: 2,
                data_points: vec![jmetrics::HistogramDataPoint {
                    attributes: vec![jcommon::KeyValue {
                        key: "route".into(),
                        value: Some(jcommon::AnyValue::StringValue("/api".into())),
                    }],
                    start_time_unix_nano: 10,
                    time_unix_nano: 20,
                    count: 100,
                    sum: Some(1234.5),
                    bucket_counts: vec![10, 60, 30],
                    explicit_bounds: vec![10.0, 100.0],
                    exemplars: vec![jmetrics::Exemplar {
                        filtered_attributes: vec![],
                        time_unix_nano: 20,
                        value: Some(jmetrics::PointValue::AsInt(12)),
                        trace_id: (0..16).collect(),
                        span_id: (0..8).collect(),
                    }],
                    flags: 0,
                    min: Some(1.0),
                    max: Some(99.0),
                }],
            })),
        }
    }

    #[test]
    fn metric_prost_roundtrip() {
        let jmetric = sample_metric();
        let pmetric: pmetrics::Metric = jmetric.clone().into();
        let back: jmetrics::Metric = pmetric.into();
        assert_eq!(jmetric, back);
    }

    #[test]
    fn metric_json_roundtrip() {
        let jmetric = sample_metric();
        let line = serde_json::to_string(&jmetric).unwrap();
        let parsed: jmetrics::Metric = serde_json::from_str(&line).unwrap();
        assert_eq!(parsed, jmetric);
        assert!(line.contains("bucketCounts"));
        assert!(line.contains("\"count\":\"100\""));
        assert!(line.contains("asInt"));
    }

    #[test]
    fn all_metric_kinds_roundtrip() {
        let kinds = vec![
            jmetrics::MetricData::Gauge(jmetrics::Gauge {
                data_points: vec![jmetrics::NumberDataPoint {
                    value: Some(jmetrics::PointValue::AsDouble(1.5)),
                    time_unix_nano: 5,
                    ..Default::default()
                }],
            }),
            jmetrics::MetricData::Sum(jmetrics::Sum {
                data_points: vec![jmetrics::NumberDataPoint {
                    value: Some(jmetrics::PointValue::AsInt(7)),
                    ..Default::default()
                }],
                aggregation_temporality: 2,
                is_monotonic: true,
            }),
            jmetrics::MetricData::ExponentialHistogram(jmetrics::ExponentialHistogram {
                aggregation_temporality: 2,
                data_points: vec![jmetrics::ExponentialHistogramDataPoint {
                    scale: -2,
                    zero_count: 1,
                    positive: Some(jmetrics::Buckets {
                        offset: 3,
                        bucket_counts: vec![1, 2],
                    }),
                    ..Default::default()
                }],
            }),
            jmetrics::MetricData::Summary(jmetrics::Summary {
                data_points: vec![jmetrics::SummaryDataPoint {
                    quantile_values: vec![jmetrics::QuantileValue {
                        quantile: 0.99,
                        value: 12.3,
                    }],
                    count: 9,
                    sum: 4.2,
                    ..Default::default()
                }],
            }),
        ];
        for data in kinds {
            let m = jmetrics::Metric {
                name: "m".into(),
                data: Some(data),
                ..Default::default()
            };
            let p: pmetrics::Metric = m.clone().into();
            let back: jmetrics::Metric = p.into();
            assert_eq!(m, back);
            let line = serde_json::to_string(&m).unwrap();
            let parsed: jmetrics::Metric = serde_json::from_str(&line).unwrap();
            assert_eq!(m, parsed);
        }
    }
}
