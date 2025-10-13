//! # openrtb-rs
//!
//! Rust bindings for OpenRTB 2.x protocol with automatic JSON handling.
//!
//! ## Quick Start
//!
//! Use standard `serde_json` for parsing and serialization. The library automatically
//! handles OpenRTB's quirks (boolean fields as integers, extension fields, etc.):
//!
//! ```ignore
//! use rtb::BidRequest;
//!
//! // Parse OpenRTB JSON
//! let request: BidRequest = serde_json::from_str(json)?;
//!
//! // Serialize back to JSON
//! let json = serde_json::to_string(&request)?;
//! ```
//!
//! ## Working with Extension Fields
//!
//! OpenRTB allows custom fields in `ext` objects. Access both proto-defined
//! and custom fields easily:
//!
//! ```ignore
//! use rtb::BidRequest;
//!
//! let request: BidRequest = serde_json::from_str(json)?;
//!
//! // Proto fields - full autocomplete support
//! let gpid = request.imp[0].ext.gpid;
//! let skadn = request.imp[0].ext.skadn;
//!
//! // Custom fields - dynamic access
//! let channel = request.imp[0].ext.custom().get_i64("channel");
//! let enabled = request.imp[0].ext.custom().get_bool("enabled");
//! ```
//!
//! See the [`extensions`] module for detailed documentation on working with custom fields.
//!
//! ## Features
//!
//! - **Standard serde_json**: No custom serializers needed
//! - **Bool as 0/1**: Handled automatically per OpenRTB spec
//! - **Extension fields**: Type-safe access to custom fields
//! - **gRPC support**: Full protobuf encoding via tonic
//! - **Builder pattern**: Convenient struct construction
//!
//! ## Protocol Support
//!
//! This library supports OpenRTB 2.x protocol. It generates types from the official
//! OpenRTB protobuf definitions maintained by the IAB Tech Lab.

/// OpenRTB protocol types generated from protobuf definitions.
///
/// This module contains all OpenRTB message types (BidRequest, BidResponse, etc.)
/// generated from the official protobuf specification.
pub mod openrtb;

/// Extension field support for custom OpenRTB fields.
///
/// This module provides utilities for accessing both standard proto-defined fields
/// and custom/unknown fields in OpenRTB extension objects.
///
/// See the module documentation for detailed usage examples.
pub mod extensions {
    pub use crate::compat::extensions::*;
}

// Re-export all OpenRTB types at the crate root for convenience
pub use openrtb::*;

// Re-export pbjson_types for working with google.protobuf.Value (with serde support)
pub use pbjson_types;

// Internal compatibility layer (not public)
pub(crate) mod compat;
pub mod server;
