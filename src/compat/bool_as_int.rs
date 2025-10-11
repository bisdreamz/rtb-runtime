//! Boolean-to-Integer Conversion for OpenRTB Compliance
//!
//! This module provides serde wrappers that handle the OpenRTB specification's
//! requirement that certain boolean fields be serialized as integers (0 or 1)
//! instead of JSON boolean literals (true/false).
//!
//! ## The Problem
//!
//! The OpenRTB protobuf definitions define many fields as `bool` types, which
//! would naturally serialize to JSON as `true` or `false`. However, the OpenRTB
//! JSON specification explicitly requires these same fields to use integer values
//! `0` (false) or `1` (true).
//!
//! Fields affected include: test, allimps, secure, instl, skip, coppa, gdpr, dnt,
//! lmt, and many others. See the build.rs patching logic for the complete list.
//!
//! ## The Solution
//!
//! At build time, our code generator patches the generated serde implementations
//! to wrap these specific boolean fields with our conversion helpers. This happens
//! transparently - users of the library never need to know about this module.
//!
//! ## Usage
//!
//! **End users never call these types directly.** They are automatically applied
//! by the patched serde implementations.
//!
//! The build system modifies generated code from:
//! ```ignore
//! struct_ser.serialize_field("skip", &self.skip)?;
//! ```
//!
//! To:
//! ```ignore
//! struct_ser.serialize_field("skip", &crate::compat::bool_as_int::Ser(&self.skip))?;
//! ```
//!
//! This ensures that boolean fields serialize as 0/1 while maintaining the natural
//! `bool` type in Rust code for type safety and ergonomics.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use std::fmt;

/// Serialization wrapper that converts a boolean to 0 or 1.
///
/// Used internally by generated serde implementations for OpenRTB boolean fields.
/// Serializes `false` as `0` and `true` as `1`.
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

/// Deserialization wrapper that converts 0/1 integers (or boolean literals) to bool.
///
/// Used internally by generated serde implementations for OpenRTB boolean fields.
/// Accepts multiple input formats for maximum compatibility:
/// - Integers: 0 (false), 1 (true), or any non-zero value (true)
/// - Booleans: true, false
/// - Strings: "0", "1", "true", "false" (case-insensitive)
///
/// This flexible parsing ensures compatibility with various OpenRTB implementations
/// that may deviate slightly from the strict specification.
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
