//! OpenRTB Compatibility Layer
//!
//! This module contains internal utilities for adapting proto-generated types
//! to work with the OpenRTB JSON specification, which has quirks that don't map
//! cleanly to protobuf semantics.
//!
//! Most of this module is internal implementation details. End users primarily
//! interact with the public `extensions` submodule exported at the crate root.

pub mod bool_as_int;
pub mod extensions;
