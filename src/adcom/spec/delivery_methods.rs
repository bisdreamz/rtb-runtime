//! Delivery Methods
//!
//! The various options for the delivery of video or audio content.

use crate::spec_list;

spec_list! {
    /// Streaming
    STREAMING = 1 => "Streaming",

    /// Progressive
    PROGRESSIVE = 2 => "Progressive",

    /// Download
    DOWNLOAD = 3 => "Download",
}
