//! Serde helper modules for the OTLP/JSON wire format.
//!
//! Mirrors the standard protobuf-JSON mapping as used by OTLP:
//! - 64-bit integers are emitted as decimal strings (parsers accept strings and numbers)
//! - trace/span IDs are emitted as lowercase hex strings (parsers accept any case)
//! - bytes are emitted as base64 strings

use serde::de::{Deserializer, Error, Unexpected, Visitor};
use serde::ser::Serializer;
use std::fmt;

pub mod hex_bytes {
    use super::*;

    pub fn serialize<S: Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&crate::json::hex_encode(v))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        struct V;

        impl Visitor<'_> for V {
            type Value = Vec<u8>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a hex-encoded ID string")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                crate::json::hex_decode(v).map_err(|e| E::custom(format!("invalid hex ID: {e}")))
            }
        }

        d.deserialize_str(V)
    }
}

fn visit_u64<E: Error>(v: u64) -> Result<u64, E> {
    Ok(v)
}

pub mod u64_str {
    use super::*;

    pub fn serialize<S: Serializer>(v: &u64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&v.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        struct V;

        impl Visitor<'_> for V {
            type Value = u64;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a uint64 number or decimal string")
            }

            fn visit_u64<E: Error>(self, v: u64) -> Result<u64, E> {
                visit_u64(v)
            }

            fn visit_i64<E: Error>(self, v: i64) -> Result<u64, E> {
                u64::try_from(v)
                    .map_err(|_| E::invalid_value(Unexpected::Signed(v), &"a non-negative uint64"))
            }

            fn visit_f64<E: Error>(self, v: f64) -> Result<u64, E> {
                if v.fract() == 0.0 && v >= 0.0 && v <= u64::MAX as f64 {
                    Ok(v as u64)
                } else {
                    Err(E::invalid_value(
                        Unexpected::Float(v),
                        &"an integral uint64",
                    ))
                }
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<u64, E> {
                v.parse::<u64>().map_err(E::custom)
            }
        }

        d.deserialize_any(V)
    }
}

pub mod i64_str {
    use super::*;

    pub fn serialize<S: Serializer>(v: &i64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&v.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
        struct V;

        impl Visitor<'_> for V {
            type Value = i64;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an int64 number or decimal string")
            }

            fn visit_i64<E: Error>(self, v: i64) -> Result<i64, E> {
                Ok(v)
            }

            fn visit_u64<E: Error>(self, v: u64) -> Result<i64, E> {
                i64::try_from(v).map_err(|_| E::invalid_value(Unexpected::Unsigned(v), &"an int64"))
            }

            fn visit_f64<E: Error>(self, v: f64) -> Result<i64, E> {
                if v.fract() == 0.0 && v >= i64::MIN as f64 && v <= i64::MAX as f64 {
                    Ok(v as i64)
                } else {
                    Err(E::invalid_value(Unexpected::Float(v), &"an integral int64"))
                }
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<i64, E> {
                v.parse::<i64>().map_err(E::custom)
            }
        }

        d.deserialize_any(V)
    }
}

pub mod vec_u64_str {
    use super::*;

    pub fn serialize<S: Serializer>(v: &[u64], s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(v.iter().map(|x| x.to_string()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u64>, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = Vec<u64>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a sequence of uint64 numbers or decimal strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(item) = seq.next_element::<serde_json::Value>()? {
                    let v = match item {
                        serde_json::Value::String(s) => {
                            s.parse::<u64>().map_err(A::Error::custom)?
                        }
                        serde_json::Value::Number(n) => n
                            .as_u64()
                            .ok_or_else(|| A::Error::custom("not a non-negative uint64"))?,
                        other => {
                            return Err(A::Error::custom(format!(
                                "expected uint64 or string, got {other}"
                            )))
                        }
                    };
                    out.push(v);
                }
                Ok(out)
            }
        }

        d.deserialize_seq(V)
    }
}

pub mod vec_i64_str {
    use super::*;

    pub fn serialize<S: Serializer>(v: &[i64], s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(v.iter().map(|x| x.to_string()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<i64>, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = Vec<i64>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a sequence of int64 numbers or decimal strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(item) = seq.next_element::<serde_json::Value>()? {
                    let v = match item {
                        serde_json::Value::String(s) => {
                            s.parse::<i64>().map_err(A::Error::custom)?
                        }
                        serde_json::Value::Number(n) => n
                            .as_i64()
                            .or_else(|| {
                                n.as_f64()
                                    .filter(|f| f.fract() == 0.0)
                                    .and_then(|f| i64::try_from(f as i128).ok())
                            })
                            .ok_or_else(|| A::Error::custom("not an int64"))?,
                        other => {
                            return Err(A::Error::custom(format!(
                                "expected int64 or string, got {other}"
                            )))
                        }
                    };
                    out.push(v);
                }
                Ok(out)
            }
        }

        d.deserialize_seq(V)
    }
}
