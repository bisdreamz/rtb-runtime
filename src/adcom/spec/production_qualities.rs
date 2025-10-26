//! Production Qualities
//!
//! Options for content quality. These values are defined by the IAB.

use crate::spec_list;

spec_list! {
    /// Unknown
    UNKNOWN = 0 => "Unknown",

    /// Professionally Produced
    PROFESSIONALLY_PRODUCED = 1 => "Professionally Produced",

    /// Prosumer
    PROSUMER = 2 => "Prosumer",

    /// User Generated (UGC)
    USER_GENERATED = 3 => "User Generated (UGC)",
}
