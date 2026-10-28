//! metrics/v1 JSON model.

use super::common::{InstrumentationScope, KeyValue, Resource};
use super::wire;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMetricsServiceRequest {
    #[serde(
        alias = "resource_metrics",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub resource_metrics: Vec<ResourceMetrics>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetrics {
    #[serde(alias = "resource", default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<Resource>,
    #[serde(
        alias = "scope_metrics",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub scope_metrics: Vec<ScopeMetrics>,
    #[serde(
        alias = "schema_url",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub schema_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeMetrics {
    #[serde(alias = "scope", default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<InstrumentationScope>,
    #[serde(alias = "metrics", default, skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<Metric>,
    #[serde(
        alias = "schema_url",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub schema_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    #[serde(alias = "name", default)]
    pub name: String,
    #[serde(
        alias = "description",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub description: String,
    #[serde(alias = "unit", default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
    #[serde(alias = "metadata", default, skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<KeyValue>,
    #[serde(alias = "data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<MetricData>,
}

/// Oneof `metric::Data`, serialized as a single-key object
/// (protobuf JSON oneof mapping).
#[derive(Clone, Debug, PartialEq)]
pub enum MetricData {
    Gauge(Gauge),
    Sum(Sum),
    Histogram(Histogram),
    ExponentialHistogram(ExponentialHistogram),
    Summary(Summary),
}

impl Serialize for MetricData {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(1))?;
        match self {
            MetricData::Gauge(v) => map.serialize_entry("gauge", v)?,
            MetricData::Sum(v) => map.serialize_entry("sum", v)?,
            MetricData::Histogram(v) => map.serialize_entry("histogram", v)?,
            MetricData::ExponentialHistogram(v) => {
                map.serialize_entry("exponentialHistogram", v)?
            }
            MetricData::Summary(v) => map.serialize_entry("summary", v)?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for MetricData {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = MetricData;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a single-key JSON object for metric data")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut found: Option<MetricData> = None;
                while let Some(key) = map.next_key::<String>()? {
                    let value = match key.as_str() {
                        "gauge" => map.next_value::<Option<Gauge>>()?.map(MetricData::Gauge),
                        "sum" => map.next_value::<Option<Sum>>()?.map(MetricData::Sum),
                        "histogram" => map
                            .next_value::<Option<Histogram>>()?
                            .map(MetricData::Histogram),
                        "exponentialHistogram" | "exponential_histogram" => map
                            .next_value::<Option<ExponentialHistogram>>()?
                            .map(MetricData::ExponentialHistogram),
                        "summary" => map
                            .next_value::<Option<Summary>>()?
                            .map(MetricData::Summary),
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &[
                                    "gauge",
                                    "sum",
                                    "histogram",
                                    "exponentialHistogram",
                                    "summary",
                                ],
                            ))
                        }
                    };
                    if let Some(v) = value {
                        found = Some(v);
                    }
                }
                found.ok_or_else(|| {
                    serde::de::Error::custom("metric data object has no known data key")
                })
            }
        }

        d.deserialize_map(V)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gauge {
    #[serde(alias = "data_points", default, skip_serializing_if = "Vec::is_empty")]
    pub data_points: Vec<NumberDataPoint>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sum {
    #[serde(alias = "data_points", default, skip_serializing_if = "Vec::is_empty")]
    pub data_points: Vec<NumberDataPoint>,
    #[serde(
        alias = "aggregation_temporality",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub aggregation_temporality: i32,
    #[serde(
        alias = "is_monotonic",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub is_monotonic: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Histogram {
    #[serde(alias = "data_points", default, skip_serializing_if = "Vec::is_empty")]
    pub data_points: Vec<HistogramDataPoint>,
    #[serde(
        alias = "aggregation_temporality",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub aggregation_temporality: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExponentialHistogram {
    #[serde(alias = "data_points", default, skip_serializing_if = "Vec::is_empty")]
    pub data_points: Vec<ExponentialHistogramDataPoint>,
    #[serde(
        alias = "aggregation_temporality",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub aggregation_temporality: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    #[serde(alias = "data_points", default, skip_serializing_if = "Vec::is_empty")]
    pub data_points: Vec<SummaryDataPoint>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberDataPoint {
    #[serde(alias = "attributes", default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(
        alias = "start_time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub start_time_unix_nano: u64,
    #[serde(
        alias = "time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub time_unix_nano: u64,
    #[serde(alias = "exemplars", default, skip_serializing_if = "Vec::is_empty")]
    pub exemplars: Vec<Exemplar>,
    #[serde(alias = "flags", default, skip_serializing_if = "crate::json::is_zero")]
    pub flags: u32,
    #[serde(alias = "value", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<PointValue>,
}

/// Oneof `number_data_point::Value` (`as_double` / `as_int`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointValue {
    AsDouble(f64),
    AsInt(i64),
}

impl Serialize for PointValue {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(1))?;
        match self {
            PointValue::AsDouble(v) => map.serialize_entry("asDouble", v)?,
            PointValue::AsInt(v) => map.serialize_entry("asInt", &v.to_string())?,
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for PointValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = PointValue;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a single-key JSON object with asDouble or asInt")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut found: Option<PointValue> = None;
                while let Some(key) = map.next_key::<String>()? {
                    let value = match key.as_str() {
                        "asDouble" | "as_double" => {
                            Some(PointValue::AsDouble(map.next_value::<f64>()?))
                        }
                        "asInt" | "as_int" => {
                            #[derive(Deserialize)]
                            #[serde(untagged)]
                            enum Num {
                                S(String),
                                I(i64),
                                U(u64),
                                F(f64),
                            }
                            let n: Option<Num> = map.next_value()?;
                            match n {
                                Some(Num::S(s)) => Some(PointValue::AsInt(
                                    s.parse().map_err(serde::de::Error::custom)?,
                                )),
                                Some(Num::I(i)) => Some(PointValue::AsInt(i)),
                                Some(Num::U(u)) => Some(PointValue::AsInt(
                                    i64::try_from(u).map_err(serde::de::Error::custom)?,
                                )),
                                Some(Num::F(f)) => Some(PointValue::AsInt(
                                    i64::try_from(f as i128).map_err(serde::de::Error::custom)?,
                                )),
                                None => None,
                            }
                        }
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &["asDouble", "asInt"],
                            ))
                        }
                    };
                    if let Some(v) = value {
                        found = Some(v);
                    }
                }
                found.ok_or_else(|| serde::de::Error::custom("data point value object is empty"))
            }
        }

        d.deserialize_map(V)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exemplar {
    #[serde(
        alias = "filtered_attributes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub filtered_attributes: Vec<KeyValue>,
    #[serde(
        alias = "time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub time_unix_nano: u64,
    #[serde(alias = "value", default, skip_serializing_if = "Option::is_none")]
    pub value: Option<PointValue>,
    #[serde(
        alias = "trace_id",
        with = "wire::hex_bytes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub trace_id: Vec<u8>,
    #[serde(
        alias = "span_id",
        with = "wire::hex_bytes",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub span_id: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistogramDataPoint {
    #[serde(alias = "attributes", default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(
        alias = "start_time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub start_time_unix_nano: u64,
    #[serde(
        alias = "time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub time_unix_nano: u64,
    #[serde(
        alias = "count",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub count: u64,
    #[serde(alias = "sum", default, skip_serializing_if = "Option::is_none")]
    pub sum: Option<f64>,
    #[serde(
        alias = "bucket_counts",
        with = "wire::vec_u64_str",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub bucket_counts: Vec<u64>,
    #[serde(
        alias = "explicit_bounds",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub explicit_bounds: Vec<f64>,
    #[serde(alias = "exemplars", default, skip_serializing_if = "Vec::is_empty")]
    pub exemplars: Vec<Exemplar>,
    #[serde(alias = "flags", default, skip_serializing_if = "crate::json::is_zero")]
    pub flags: u32,
    #[serde(alias = "min", default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(alias = "max", default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExponentialHistogramDataPoint {
    #[serde(alias = "attributes", default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(
        alias = "start_time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub start_time_unix_nano: u64,
    #[serde(
        alias = "time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub time_unix_nano: u64,
    #[serde(
        alias = "count",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub count: u64,
    #[serde(alias = "sum", default, skip_serializing_if = "Option::is_none")]
    pub sum: Option<f64>,
    #[serde(alias = "scale", default, skip_serializing_if = "crate::json::is_zero")]
    pub scale: i32,
    #[serde(
        alias = "zero_count",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub zero_count: u64,
    #[serde(alias = "positive", default, skip_serializing_if = "Option::is_none")]
    pub positive: Option<Buckets>,
    #[serde(alias = "negative", default, skip_serializing_if = "Option::is_none")]
    pub negative: Option<Buckets>,
    #[serde(alias = "exemplars", default, skip_serializing_if = "Vec::is_empty")]
    pub exemplars: Vec<Exemplar>,
    #[serde(alias = "flags", default, skip_serializing_if = "crate::json::is_zero")]
    pub flags: u32,
    #[serde(alias = "min", default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(alias = "max", default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(
        alias = "zero_threshold",
        default,
        skip_serializing_if = "crate::json::is_zero_f64"
    )]
    pub zero_threshold: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Buckets {
    #[serde(
        alias = "offset",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub offset: i32,
    #[serde(
        alias = "bucket_counts",
        with = "wire::vec_u64_str",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub bucket_counts: Vec<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDataPoint {
    #[serde(alias = "attributes", default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(
        alias = "start_time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub start_time_unix_nano: u64,
    #[serde(
        alias = "time_unix_nano",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub time_unix_nano: u64,
    #[serde(
        alias = "count",
        with = "wire::u64_str",
        default,
        skip_serializing_if = "crate::json::is_zero"
    )]
    pub count: u64,
    #[serde(
        alias = "sum",
        default,
        skip_serializing_if = "crate::json::is_zero_f64"
    )]
    pub sum: f64,
    #[serde(
        alias = "quantile_values",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub quantile_values: Vec<QuantileValue>,
    #[serde(alias = "flags", default, skip_serializing_if = "crate::json::is_zero")]
    pub flags: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuantileValue {
    #[serde(alias = "quantile", default)]
    pub quantile: f64,
    #[serde(alias = "value", default)]
    pub value: f64,
}
