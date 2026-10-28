//! Conversions between prost-generated OTLP types and the JSON model.

pub mod logs;
pub mod metrics;
pub mod trace;

use crate::json::common as jcommon;
use opentelemetry_proto::tonic::common::v1 as pcommon;
use opentelemetry_proto::tonic::resource::v1 as presource;

/// Maps a vector element-wise between mirrored types.
pub(super) fn vec_conv<I, O: From<I>>(v: Vec<I>) -> Vec<O> {
    v.into_iter().map(Into::into).collect()
}

impl From<jcommon::KeyValue> for pcommon::KeyValue {
    fn from(v: jcommon::KeyValue) -> Self {
        pcommon::KeyValue {
            key: v.key,
            value: v.value.map(pcommon::AnyValue::from),
            key_strindex: 0,
        }
    }
}

impl From<&jcommon::KeyValue> for pcommon::KeyValue {
    fn from(v: &jcommon::KeyValue) -> Self {
        v.clone().into()
    }
}

impl From<pcommon::KeyValue> for jcommon::KeyValue {
    fn from(v: pcommon::KeyValue) -> Self {
        jcommon::KeyValue {
            key: v.key,
            value: v.value.map(jcommon::AnyValue::from),
        }
    }
}

impl From<&pcommon::KeyValue> for jcommon::KeyValue {
    fn from(v: &pcommon::KeyValue) -> Self {
        v.clone().into()
    }
}

impl From<jcommon::AnyValue> for pcommon::AnyValue {
    fn from(v: jcommon::AnyValue) -> Self {
        let value = match v {
            jcommon::AnyValue::StringValue(s) => pcommon::any_value::Value::StringValue(s),
            jcommon::AnyValue::BoolValue(b) => pcommon::any_value::Value::BoolValue(b),
            jcommon::AnyValue::IntValue(i) => pcommon::any_value::Value::IntValue(i),
            jcommon::AnyValue::DoubleValue(d) => pcommon::any_value::Value::DoubleValue(d),
            jcommon::AnyValue::ArrayValue(a) => {
                pcommon::any_value::Value::ArrayValue(pcommon::ArrayValue {
                    values: vec_conv(a.values),
                })
            }
            jcommon::AnyValue::KvlistValue(k) => {
                pcommon::any_value::Value::KvlistValue(pcommon::KeyValueList {
                    values: vec_conv(k.values),
                })
            }
            jcommon::AnyValue::BytesValue(b) => pcommon::any_value::Value::BytesValue(b),
        };
        pcommon::AnyValue { value: Some(value) }
    }
}

impl From<pcommon::AnyValue> for jcommon::AnyValue {
    fn from(v: pcommon::AnyValue) -> Self {
        match v.value {
            Some(pcommon::any_value::Value::StringValue(s)) => jcommon::AnyValue::StringValue(s),
            Some(pcommon::any_value::Value::BoolValue(b)) => jcommon::AnyValue::BoolValue(b),
            Some(pcommon::any_value::Value::IntValue(i)) => jcommon::AnyValue::IntValue(i),
            Some(pcommon::any_value::Value::DoubleValue(d)) => jcommon::AnyValue::DoubleValue(d),
            Some(pcommon::any_value::Value::ArrayValue(a)) => {
                jcommon::AnyValue::ArrayValue(jcommon::ArrayValue {
                    values: vec_conv(a.values),
                })
            }
            Some(pcommon::any_value::Value::KvlistValue(k)) => {
                jcommon::AnyValue::KvlistValue(jcommon::KvlistValue {
                    values: vec_conv(k.values),
                })
            }
            Some(pcommon::any_value::Value::BytesValue(b)) => jcommon::AnyValue::BytesValue(b),
            // Internal string-table optimization (profiles); the table lives in the
            // profiles payload, which otelcat does not parse — render as empty.
            Some(pcommon::any_value::Value::StringValueStrindex(_)) => {
                jcommon::AnyValue::StringValue(String::new())
            }
            None => jcommon::AnyValue::StringValue(String::new()),
        }
    }
}

impl From<jcommon::Resource> for presource::Resource {
    fn from(v: jcommon::Resource) -> Self {
        presource::Resource {
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
            entity_refs: vec_conv(v.entity_refs),
        }
    }
}

impl From<jcommon::EntityRef> for pcommon::EntityRef {
    fn from(v: jcommon::EntityRef) -> Self {
        pcommon::EntityRef {
            schema_url: v.schema_url,
            r#type: v.r#type,
            id_keys: v.id_keys,
            description_keys: v.description_keys,
        }
    }
}

impl From<pcommon::EntityRef> for jcommon::EntityRef {
    fn from(v: pcommon::EntityRef) -> Self {
        jcommon::EntityRef {
            schema_url: v.schema_url,
            r#type: v.r#type,
            id_keys: v.id_keys,
            description_keys: v.description_keys,
        }
    }
}

impl From<presource::Resource> for jcommon::Resource {
    fn from(v: presource::Resource) -> Self {
        jcommon::Resource {
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
            entity_refs: vec_conv(v.entity_refs),
        }
    }
}

impl From<jcommon::InstrumentationScope> for pcommon::InstrumentationScope {
    fn from(v: jcommon::InstrumentationScope) -> Self {
        pcommon::InstrumentationScope {
            name: v.name,
            version: v.version,
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
        }
    }
}

impl From<pcommon::InstrumentationScope> for jcommon::InstrumentationScope {
    fn from(v: pcommon::InstrumentationScope) -> Self {
        jcommon::InstrumentationScope {
            name: v.name,
            version: v.version,
            attributes: vec_conv(v.attributes),
            dropped_attributes_count: v.dropped_attributes_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::common as j;
    use pcommon::any_value::Value as V;

    #[test]
    fn any_value_roundtrip_all_variants() {
        let cases = vec![
            j::AnyValue::StringValue("hello".into()),
            j::AnyValue::BoolValue(true),
            j::AnyValue::IntValue(-42),
            j::AnyValue::DoubleValue(3.25),
            j::AnyValue::ArrayValue(j::ArrayValue {
                values: vec![j::AnyValue::IntValue(1), j::AnyValue::BoolValue(false)],
            }),
            j::AnyValue::KvlistValue(j::KvlistValue {
                values: vec![j::KeyValue {
                    key: "k".into(),
                    value: Some(j::AnyValue::StringValue("v".into())),
                }],
            }),
            j::AnyValue::BytesValue(vec![0x01, 0xff]),
        ];
        for case in cases {
            let p: pcommon::AnyValue = case.clone().into();
            let back: j::AnyValue = p.into();
            assert_eq!(case, back);
        }
        // empty inner value maps to empty string on the json side
        let empty = pcommon::AnyValue { value: None };
        let jv: j::AnyValue = empty.into();
        assert_eq!(jv, j::AnyValue::StringValue(String::new()));
        // nested kvlist through prost
        let kv = j::KeyValue {
            key: "x".into(),
            value: Some(j::AnyValue::KvlistValue(j::KvlistValue {
                values: vec![j::KeyValue {
                    key: "deep".into(),
                    value: Some(j::AnyValue::IntValue(7)),
                }],
            })),
        };
        let pk: pcommon::KeyValue = kv.clone().into();
        assert!(matches!(
            pk.value.as_ref().unwrap().value,
            Some(V::KvlistValue(_))
        ));
        let back: j::KeyValue = pk.into();
        assert_eq!(kv, back);
    }
}
