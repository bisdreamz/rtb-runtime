//! IP Location Services
//!
//! The services and/or vendors used for resolving IP addresses to geolocations.

use crate::spec_list;

spec_list! {
    /// ip2location
    IP2LOCATION = 1 => "ip2location",

    /// Neustar (Quova)
    NEUSTAR_QUOVA = 2 => "Neustar (Quova)",

    /// MaxMind
    MAXMIND = 3 => "MaxMind",

    /// NetAcuity (Digital Element)
    NETACUITY = 4 => "NetAcuity (Digital Element)",

    /// 51Degrees (High Confidence)
    FIFTYONE_DEGREES_HIGH = 511 => "51Degrees (High Confidence)",

    /// 51Degrees (Medium Confidence)
    FIFTYONE_DEGREES_MEDIUM = 512 => "51Degrees (Medium Confidence)",
}
