/// Internal helpers shared by generated code (e.g. bool-as-int serde wrappers).
pub mod json;

pub mod openrtb;
pub use openrtb::*;

// Re-export pbjson_types for working with google.protobuf.Value (with serde support).
pub use pbjson_types;
