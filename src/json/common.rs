//! common/v1 + resource/v1 JSON model.

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValue {
    #[serde(default)]
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<AnyValue>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayValue {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<AnyValue>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KvlistValue {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<KeyValue>,
}

/// Oneof `any_value::Value`, serialized as a single-key object
/// (protobuf JSON oneof mapping).
#[derive(Clone, Debug, PartialEq)]
pub enum AnyValue {
    StringValue(String),
    BoolValue(bool),
    IntValue(i64),
    DoubleValue(f64),
    ArrayValue(ArrayValue),
    KvlistValue(KvlistValue),
    BytesValue(Vec<u8>),
}

impl Serialize for AnyValue {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(Some(1))?;
        match self {
            AnyValue::StringValue(v) => map.serialize_entry("stringValue", v)?,
            AnyValue::BoolValue(v) => map.serialize_entry("boolValue", v)?,
            AnyValue::IntValue(v) => map.serialize_entry("intValue", &v.to_string())?,
            AnyValue::DoubleValue(v) => map.serialize_entry("doubleValue", v)?,
            AnyValue::ArrayValue(v) => map.serialize_entry("arrayValue", v)?,
            AnyValue::KvlistValue(v) => map.serialize_entry("kvlistValue", v)?,
            AnyValue::BytesValue(v) => {
                map.serialize_entry("bytesValue", &crate::json::b64_encode(v))?
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for AnyValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = AnyValue;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a single-key JSON object for AnyValue")
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut found: Option<AnyValue> = None;
                while let Some(key) = map.next_key::<String>()? {
                    let value = match key.as_str() {
                        "stringValue" | "string_value" => {
                            let v: Option<String> = map.next_value()?;
                            v.map(AnyValue::StringValue)
                        }
                        "boolValue" | "bool_value" => {
                            let v: Option<bool> = map.next_value()?;
                            v.map(AnyValue::BoolValue)
                        }
                        "intValue" | "int_value" => {
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
                                Some(Num::S(s)) => Some(AnyValue::IntValue(
                                    s.parse().map_err(serde::de::Error::custom)?,
                                )),
                                Some(Num::I(i)) => Some(AnyValue::IntValue(i)),
                                Some(Num::U(u)) => Some(AnyValue::IntValue(
                                    i64::try_from(u).map_err(serde::de::Error::custom)?,
                                )),
                                Some(Num::F(f)) => Some(AnyValue::IntValue(
                                    i64::try_from(f as i128).map_err(serde::de::Error::custom)?,
                                )),
                                None => None,
                            }
                        }
                        "doubleValue" | "double_value" => {
                            let v: Option<f64> = map.next_value()?;
                            v.map(AnyValue::DoubleValue)
                        }
                        "arrayValue" | "array_value" => {
                            let v: Option<ArrayValue> = map.next_value()?;
                            v.map(AnyValue::ArrayValue)
                        }
                        "kvlistValue" | "kvlist_value" => {
                            let v: Option<KvlistValue> = map.next_value()?;
                            v.map(AnyValue::KvlistValue)
                        }
                        "bytesValue" | "bytes_value" => {
                            let v: Option<String> = map.next_value()?;
                            match v {
                                Some(s) => Some(AnyValue::BytesValue(
                                    crate::json::b64_decode(&s)
                                        .map_err(serde::de::Error::custom)?,
                                )),
                                None => None,
                            }
                        }
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &[
                                    "stringValue",
                                    "boolValue",
                                    "intValue",
                                    "doubleValue",
                                    "arrayValue",
                                    "kvlistValue",
                                    "bytesValue",
                                ],
                            ))
                        }
                    };
                    if let Some(v) = value {
                        found = Some(v);
                    }
                }
                found.ok_or_else(|| {
                    serde::de::Error::custom("AnyValue object has no known value key")
                })
            }
        }

        d.deserialize_map(V)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentationScope {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(default, skip_serializing_if = "crate::json::is_zero")]
    pub dropped_attributes_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<KeyValue>,
    #[serde(default, skip_serializing_if = "crate::json::is_zero")]
    pub dropped_attributes_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entity_refs: Vec<EntityRef>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRef {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub schema_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub id_keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub description_keys: Vec<String>,
}
