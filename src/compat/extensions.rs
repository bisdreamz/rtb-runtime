//! Extension Field Support for OpenRTB Custom Fields
//!
//! This module provides type-safe access to both standard protobuf-defined fields
//! and custom extension fields in OpenRTB objects.
//!
//! ## The Problem
//!
//! OpenRTB uses protobuf "extensions" to allow advertisers, publishers, and exchanges
//! to add custom fields beyond the standard specification. The proto definition includes
//! `extensions 500 to max;` declarations to signal this flexibility.
//!
//! However, prost (the Rust protobuf library) does not yet support extensions
//! (see <https://github.com/tokio-rs/prost/issues/674>). Without special handling,
//! any custom fields in the JSON would be silently dropped during parsing, or worse,
//! cause deserialization to fail.
//!
//! ## The Solution
//!
//! We wrap OpenRTB extension objects with [`ExtWithCustom<T>`], which:
//!
//! 1. Preserves all standard protobuf fields in their original types (with IDE autocomplete)
//! 2. Captures any unknown/custom fields in a [`DynamicExt`] HashMap
//! 3. Provides convenient accessor methods for common data types
//! 4. Allows type-safe deserialization of custom fields when structure is known
//! 5. Implements `prost::Message` for protobuf/gRPC compatibility
//!
//! At build time, the code generator patches all `ext` field declarations to use
//! `ExtWithCustom<T>` instead of the raw proto type. This happens transparently.
//!
//! ## Protobuf Compatibility
//!
//! `ExtWithCustom<T>` implements `prost::Message` to work seamlessly with gRPC/tonic.
//! When encoding to or decoding from protobuf:
//! - Proto-defined fields are fully supported
//! - Custom fields are ignored (they don't exist in the proto schema)
//!
//! This allows the same types to work with both JSON (HTTP) and protobuf (gRPC):
//!
//! ```ignore
//! // Works with actix-web JSON
//! let request: BidRequest = serde_json::from_str(json)?;
//! request.imp[0].ext.as_ref()?.custom().get_i64("channel"); // Some(42)
//!
//! // Works with tonic protobuf
//! let request: BidRequest = decode_from_grpc(bytes)?;
//! request.imp[0].ext.as_ref()?.custom().get_i64("channel"); // None
//! ```
//!
//! ## Usage
//!
//! ### Accessing Standard Proto Fields
//!
//! Proto fields are accessible directly via Deref, with full IDE support:
//!
//! ```ignore
//! // Autocomplete works! These are real proto fields
//! let gpid = request.imp[0].ext.gpid;
//! let skadn = request.imp[0].ext.skadn;
//! ```
//!
//! ### Accessing Custom Fields
//!
//! Custom/unknown fields are accessed via the `.custom()` method:
//!
//! ```ignore
//! let custom = request.imp[0].ext.custom();
//!
//! // Simple primitives
//! if let Some(channel) = custom.get_i64("channel") {
//!     println!("Channel ID: {}", channel);
//! }
//!
//! // With defaults
//! let enabled = custom.get_bool_or("enabled", true);
//!
//! // Nested objects
//! if let Some(metadata) = custom.get_nested("metadata") {
//!     let version = metadata.get_str("version");
//! }
//!
//! // Type-safe deserialization when structure is known
//! #[derive(Deserialize)]
//! struct VideoMetadata {
//!     channel: i64,
//!     rewarded: bool,
//! }
//! let meta: VideoMetadata = custom.as_typed()?;
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

// Prost imports for protobuf support
use prost::bytes::{Buf, BufMut};
use prost::encoding::{DecodeContext, WireType};
use prost::{DecodeError, Message};

/// Dynamic extension field storage with convenient accessor methods.
///
/// This type wraps a HashMap of JSON values and provides type-safe accessor
/// methods for common data types. All methods return `Option<T>` since custom
/// fields may or may not be present in any given request.
///
/// Unknown fields are automatically captured during deserialization via serde's
/// `#[serde(flatten)]` attribute.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DynamicExt {
    #[serde(flatten)]
    inner: HashMap<String, Value>,
}

impl DynamicExt {
    /// Creates an empty extension object.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    // ===== Raw Access =====

    /// Get raw JSON value for manual handling.
    ///
    /// Returns `None` if the field doesn't exist.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.inner.get(key)
    }

    /// Check if a field exists.
    pub fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Returns the number of custom fields.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if there are no custom fields.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    // ===== Primitive Types =====

    /// Get boolean value.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.inner.get(key)?.as_bool()
    }

    /// Get i64 integer value (handles integers up to 2^63-1).
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.inner.get(key)?.as_i64()
    }

    /// Get u64 integer value (handles integers up to 2^64-1).
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.inner.get(key)?.as_u64()
    }

    /// Get f64 floating point value.
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.inner.get(key)?.as_f64()
    }

    /// Get string value as a borrowed slice.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.inner.get(key)?.as_str()
    }

    /// Get string value as an owned String.
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.inner.get(key)?.as_str().map(String::from)
    }

    // ===== Collections =====

    /// Get array of JSON values.
    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.inner.get(key)?.as_array()
    }

    /// Get object as a JSON map.
    pub fn get_object(&self, key: &str) -> Option<&Map<String, Value>> {
        self.inner.get(key)?.as_object()
    }

    /// Get nested object as a DynamicExt for chaining.
    ///
    /// This allows accessing nested custom fields:
    ///
    /// ```ignore
    /// if let Some(metadata) = ext.custom().get_nested("metadata") {
    ///     let version = metadata.get_str("version");
    /// }
    /// ```
    pub fn get_nested(&self, key: &str) -> Option<DynamicExt> {
        let obj = self.inner.get(key)?.as_object()?;
        Some(DynamicExt {
            inner: obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        })
    }

    // ===== Typed Deserialization =====

    /// Deserialize a single field into any serde-compatible type.
    ///
    /// Returns `Ok(None)` if the field doesn't exist, or `Err` if deserialization fails.
    ///
    /// ```ignore
    /// #[derive(Deserialize)]
    /// struct Deal {
    ///     id: String,
    ///     price: f64,
    /// }
    ///
    /// if let Some(deal) = ext.custom().get_as::<Deal>("deal")? {
    ///     println!("Deal: {} @ ${}", deal.id, deal.price);
    /// }
    /// ```
    pub fn get_as<T>(&self, key: &str) -> Result<Option<T>, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.inner
            .get(key)
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
    }

    /// Deserialize an array field into a vector of typed elements.
    ///
    /// Returns `Ok(None)` if the field doesn't exist, or `Err` if deserialization fails.
    ///
    /// ```ignore
    /// let categories: Vec<String> = ext.custom()
    ///     .get_array_as("categories")?
    ///     .unwrap_or_default();
    /// ```
    pub fn get_array_as<T>(&self, key: &str) -> Result<Option<Vec<T>>, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.inner
            .get(key)
            .map(|v| serde_json::from_value(v.clone()))
            .transpose()
    }

    /// Deserialize the entire extension object into a typed struct.
    ///
    /// This converts all custom fields into a user-defined struct when you know
    /// the expected structure.
    ///
    /// ```ignore
    /// #[derive(Deserialize)]
    /// struct VideoExt {
    ///     channel: i64,
    ///     rewarded: bool,
    ///     categories: Vec<String>,
    /// }
    ///
    /// let typed: VideoExt = ext.custom().as_typed()?;
    /// ```
    pub fn as_typed<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let obj: Map<String, Value> = self
            .inner
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        serde_json::from_value(Value::Object(obj))
    }

    // ===== Convenience: With Defaults =====

    /// Get boolean with a default fallback.
    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.get_bool(key).unwrap_or(default)
    }

    /// Get i64 with a default fallback.
    pub fn get_i64_or(&self, key: &str, default: i64) -> i64 {
        self.get_i64(key).unwrap_or(default)
    }

    /// Get u64 with a default fallback.
    pub fn get_u64_or(&self, key: &str, default: u64) -> u64 {
        self.get_u64(key).unwrap_or(default)
    }

    /// Get f64 with a default fallback.
    pub fn get_f64_or(&self, key: &str, default: f64) -> f64 {
        self.get_f64(key).unwrap_or(default)
    }

    /// Get string with a default fallback.
    pub fn get_string_or(&self, key: &str, default: String) -> String {
        self.get_string(key).unwrap_or(default)
    }

    // ===== Mutation: Insert/Update Fields =====

    /// Insert or update a custom field with a raw JSON value.
    ///
    /// Returns the previous value if the key existed.
    ///
    /// ```ignore
    /// use serde_json::json;
    ///
    /// let mut ext = DynamicExt::new();
    /// ext.insert("channel".to_string(), json!(42));
    /// ext.insert("metadata".to_string(), json!({"version": "1.0"}));
    /// ```
    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.inner.insert(key, value)
    }

    /// Insert a boolean value.
    pub fn insert_bool(&mut self, key: String, value: bool) {
        self.inner.insert(key, Value::Bool(value));
    }

    /// Insert an i64 integer value.
    pub fn insert_i64(&mut self, key: String, value: i64) {
        self.inner.insert(key, Value::Number(value.into()));
    }

    /// Insert a u64 integer value.
    pub fn insert_u64(&mut self, key: String, value: u64) {
        self.inner.insert(key, Value::Number(value.into()));
    }

    /// Insert an f64 floating point value.
    pub fn insert_f64(&mut self, key: String, value: f64) {
        if let Some(num) = serde_json::Number::from_f64(value) {
            self.inner.insert(key, Value::Number(num));
        }
    }

    /// Insert a string value.
    pub fn insert_string(&mut self, key: String, value: String) {
        self.inner.insert(key, Value::String(value));
    }

    /// Insert an array value.
    pub fn insert_array(&mut self, key: String, value: Vec<Value>) {
        self.inner.insert(key, Value::Array(value));
    }

    /// Insert a nested object.
    pub fn insert_object(&mut self, key: String, value: Map<String, Value>) {
        self.inner.insert(key, Value::Object(value));
    }

    /// Remove a custom field.
    ///
    /// Returns the removed value if the key existed.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.inner.remove(key)
    }
}

/// Wrapper that combines proto-defined fields with custom extension fields.
///
/// This type is automatically used for all OpenRTB `ext` fields. It preserves
/// the original proto type `T` while adding a `custom` field for unknown fields.
///
/// The Deref implementation allows transparent access to proto fields:
///
/// ```ignore
/// // These work via Deref - no .proto needed
/// let gpid = video.ext.gpid;
/// let skadn = video.ext.skadn;
///
/// // Custom fields via explicit method
/// let channel = video.ext.custom().get_i64("channel");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtWithCustom<T> {
    /// Standard protobuf-defined fields. Accessed via Deref.
    #[serde(flatten)]
    proto: T,

    /// Custom/unknown extension fields captured during deserialization.
    #[serde(flatten)]
    custom: DynamicExt,
}

impl<T> ExtWithCustom<T> {
    /// Create a new extension wrapper with proto fields and empty custom fields.
    pub fn new(proto: T) -> Self {
        Self {
            proto,
            custom: DynamicExt::new(),
        }
    }

    /// Create a new extension wrapper with both proto and custom fields.
    pub fn with_custom(proto: T, custom: DynamicExt) -> Self {
        Self { proto, custom }
    }

    /// Builder-style method to add a custom field with a raw JSON value.
    ///
    /// ```ignore
    /// use serde_json::json;
    ///
    /// let ext = ExtWithCustom::new(proto)
    ///     .with_field("force_bid".to_string(), json!(true))
    ///     .with_field("channel".to_string(), json!(42));
    /// ```
    pub fn with_field(mut self, key: String, value: Value) -> Self {
        self.custom.insert(key, value);
        self
    }

    /// Builder-style method to add a boolean custom field.
    pub fn with_bool(mut self, key: String, value: bool) -> Self {
        self.custom.insert_bool(key, value);
        self
    }

    /// Builder-style method to add an i64 custom field.
    pub fn with_i64(mut self, key: String, value: i64) -> Self {
        self.custom.insert_i64(key, value);
        self
    }

    /// Builder-style method to add a string custom field.
    pub fn with_string(mut self, key: String, value: String) -> Self {
        self.custom.insert_string(key, value);
        self
    }

    /// Access custom/unknown extension fields.
    ///
    /// Returns a reference to the DynamicExt that contains all non-proto fields.
    pub fn custom(&self) -> &DynamicExt {
        &self.custom
    }

    /// Mutable access to custom extension fields.
    pub fn custom_mut(&mut self) -> &mut DynamicExt {
        &mut self.custom
    }

    /// Get a reference to the underlying proto fields.
    pub fn proto(&self) -> &T {
        &self.proto
    }

    /// Get a mutable reference to the underlying proto fields.
    pub fn proto_mut(&mut self) -> &mut T {
        &mut self.proto
    }

    /// Consume self and return proto and custom fields separately.
    pub fn into_parts(self) -> (T, DynamicExt) {
        (self.proto, self.custom)
    }
}

/// Deref to the proto type for transparent field access.
///
/// This allows users to access proto fields directly without needing to know
/// about the ExtWithCustom wrapper:
///
/// ```ignore
/// let gpid = video.ext.gpid;  // Works via Deref, not video.ext.proto.gpid
/// ```
impl<T> Deref for ExtWithCustom<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.proto
    }
}

impl<T> DerefMut for ExtWithCustom<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.proto
    }
}

/// Implement Eq when the proto type implements PartialEq and Eq.
/// This is required for protobuf types that derive Eq.
impl<T> Eq for ExtWithCustom<T> where T: PartialEq + Eq {}

/// Implement Hash by hashing both proto and custom fields.
/// This allows ExtWithCustom to be used in hash-based collections.
impl<T> Hash for ExtWithCustom<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.proto.hash(state);
        // Hash custom fields in a deterministic order
        let mut entries: Vec<_> = self.custom.inner.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (key, value) in entries {
            key.hash(state);
            // Hash the JSON value as a string for simplicity
            // (serde_json::Value doesn't implement Hash)
            if let Ok(s) = serde_json::to_string(value) {
                s.hash(state);
            }
        }
    }
}

impl<T: Default> Default for ExtWithCustom<T> {
    fn default() -> Self {
        Self {
            proto: T::default(),
            custom: DynamicExt::default(),
        }
    }
}

/// Implements prost::Message for protobuf/gRPC compatibility.
///
/// All protobuf operations are delegated to the inner `proto` field.
/// Custom fields are ignored during protobuf encoding/decoding - they only
/// exist when deserializing from JSON.
///
/// This allows the same `ExtWithCustom<T>` type to work with both:
/// - JSON via serde (captures custom fields)
/// - Protobuf via prost (ignores custom fields)
impl<T> Message for ExtWithCustom<T>
where
    T: Message + Default,
{
    fn encode_raw(&self, buf: &mut impl BufMut) {
        // Only encode proto-defined fields for protobuf
        // Custom fields don't exist in the proto schema, so they're ignored
        self.proto.encode_raw(buf);
    }

    fn merge_field(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut impl Buf,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError> {
        // Delegate all field decoding to the inner proto type
        // Custom fields will remain empty (expected behavior for protobuf)
        self.proto.merge_field(tag, wire_type, buf, ctx)
    }

    fn encoded_len(&self) -> usize {
        // Only proto fields contribute to encoded size
        self.proto.encoded_len()
    }

    fn clear(&mut self) {
        // Clear proto fields; custom fields remain unchanged
        self.proto.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_ext_primitives() {
        let json = r#"{
            "channel": 42,
            "enabled": true,
            "price": 1.5,
            "name": "test"
        }"#;

        let ext: DynamicExt = serde_json::from_str(json).unwrap();

        assert_eq!(ext.get_i64("channel"), Some(42));
        assert_eq!(ext.get_bool("enabled"), Some(true));
        assert_eq!(ext.get_f64("price"), Some(1.5));
        assert_eq!(ext.get_str("name"), Some("test"));
        assert_eq!(ext.get_i64("missing"), None);
    }

    #[test]
    fn test_dynamic_ext_with_defaults() {
        let ext = DynamicExt::new();

        assert_eq!(ext.get_i64_or("channel", 99), 99);
        assert_eq!(ext.get_bool_or("enabled", true), true);
        assert_eq!(ext.get_string_or("name", "default".to_string()), "default");
    }

    #[test]
    fn test_dynamic_ext_nested() {
        let json = r#"{
            "metadata": {
                "version": "1.0",
                "count": 5
            }
        }"#;

        let ext: DynamicExt = serde_json::from_str(json).unwrap();

        let metadata = ext.get_nested("metadata").unwrap();
        assert_eq!(metadata.get_str("version"), Some("1.0"));
        assert_eq!(metadata.get_i64("count"), Some(5));
    }

    #[test]
    fn test_dynamic_ext_typed() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Custom {
            channel: i64,
            enabled: bool,
        }

        let json = r#"{
            "channel": 42,
            "enabled": true
        }"#;

        let ext: DynamicExt = serde_json::from_str(json).unwrap();
        let typed: Custom = ext.as_typed().unwrap();

        assert_eq!(typed.channel, 42);
        assert_eq!(typed.enabled, true);
    }

    #[test]
    fn test_ext_with_custom_deref() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
        struct ProtoExt {
            gpid: String,
            intrinsic: bool,
        }

        let json = r#"{
            "gpid": "abc123",
            "intrinsic": true,
            "channel": 42
        }"#;

        let ext: ExtWithCustom<ProtoExt> = serde_json::from_str(json).unwrap();

        // Proto fields via Deref
        assert_eq!(ext.gpid, "abc123");
        assert_eq!(ext.intrinsic, true);

        // Custom fields via .custom()
        assert_eq!(ext.custom().get_i64("channel"), Some(42));
    }

    #[test]
    fn test_ext_with_custom_serialization() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
        struct ProtoExt {
            gpid: String,
        }

        let json = r#"{"gpid":"test","custom_field":123}"#;
        let ext: ExtWithCustom<ProtoExt> = serde_json::from_str(json).unwrap();

        let serialized = serde_json::to_string(&ext).unwrap();

        assert!(serialized.contains("\"gpid\""));
        assert!(serialized.contains("\"custom_field\""));

        let deserialized: ExtWithCustom<ProtoExt> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ext, deserialized);
    }

    #[test]
    fn test_dynamic_ext_insert_methods() {
        let mut ext = DynamicExt::new();

        ext.insert_bool("force_bid".to_string(), true);
        ext.insert_i64("channel".to_string(), 42);
        ext.insert_string("name".to_string(), "test".to_string());
        ext.insert_f64("price".to_string(), 1.5);

        assert_eq!(ext.get_bool("force_bid"), Some(true));
        assert_eq!(ext.get_i64("channel"), Some(42));
        assert_eq!(ext.get_str("name"), Some("test"));
        assert_eq!(ext.get_f64("price"), Some(1.5));

        let removed = ext.remove("channel");
        assert_eq!(removed, Some(Value::Number(42.into())));
        assert_eq!(ext.get_i64("channel"), None);
    }

    #[test]
    fn test_ext_with_custom_builder_methods() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
        struct ProtoExt {
            gpid: String,
        }

        let proto = ProtoExt {
            gpid: "/homepage/banner".to_string(),
        };

        let ext = ExtWithCustom::new(proto)
            .with_bool("force_bid".to_string(), true)
            .with_i64("channel".to_string(), 42)
            .with_string("name".to_string(), "test".to_string());

        assert_eq!(ext.gpid, "/homepage/banner");
        assert_eq!(ext.custom().get_bool("force_bid"), Some(true));
        assert_eq!(ext.custom().get_i64("channel"), Some(42));
        assert_eq!(ext.custom().get_str("name"), Some("test"));
    }

    #[test]
    fn test_ext_with_custom_mutable_access() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
        struct ProtoExt {
            gpid: String,
        }

        let proto = ProtoExt {
            gpid: "/app/video".to_string(),
        };

        let mut ext = ExtWithCustom::new(proto);

        ext.custom_mut().insert_bool("rewarded".to_string(), true);
        ext.custom_mut().insert_i64("duration".to_string(), 30);

        assert_eq!(ext.custom().get_bool("rewarded"), Some(true));
        assert_eq!(ext.custom().get_i64("duration"), Some(30));
    }
}
