use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use std::fmt;

/// Wrapper that serializes a bool as 0/1 integer according to OpenRTB definitions
pub struct Ser<'a>(pub &'a bool);

impl<'a> Serialize for Ser<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *self.0 {
            serializer.serialize_u8(1)
        } else {
            serializer.serialize_u8(0)
        }
    }
}

/// Wrapper that deserializes OpenRTB bool values encoded as 0/1 integers (or regular booleans)
pub struct De(pub bool);

impl<'de> Deserialize<'de> for De {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = bool;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a boolean or 0/1 integer")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(v)
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v != 0)
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v != 0)
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v != 0.0)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    "0" => Ok(false),
                    "1" => Ok(true),
                    "true" | "True" => Ok(true),
                    "false" | "False" => Ok(false),
                    _ => v.parse::<i64>().map(|i| i != 0).map_err(E::custom),
                }
            }
        }

        deserializer.deserialize_any(Visitor).map(De)
    }
}
