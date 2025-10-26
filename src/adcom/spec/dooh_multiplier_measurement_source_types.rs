//! DOOH Multiplier Measurement Source Types
//!
//! The types of entities that provide quantity measurement for impression multipliers, which are common in Out of Home advertising.

use crate::spec_list;

spec_list! {
    /// Unknown
    UNKNOWN = 0 => "Unknown",

    /// Measurement Vendor Provided
    MEASUREMENT_VENDOR_PROVIDED = 1 => "Measurement Vendor Provided",

    /// Publisher Provided
    PUBLISHER_PROVIDED = 2 => "Publisher Provided",

    /// Exchange Provided
    EXCHANGE_PROVIDED = 3 => "Exchange Provided",
}
