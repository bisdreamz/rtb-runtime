//! Connection Types
//!
//! The options for the type of device connectivity.

use crate::spec_list;

spec_list! {
    /// Ethernet; Wired Connection
    ETHERNET = 1 => "Ethernet; Wired Connection",

    /// WIFI
    WIFI = 2 => "WIFI",

    /// Cellular Network - Unknown Generation
    CELLULAR_UNKNOWN = 3 => "Cellular Network - Unknown Generation",

    /// Cellular Network - 2G
    CELLULAR_2G = 4 => "Cellular Network - 2G",

    /// Cellular Network - 3G
    CELLULAR_3G = 5 => "Cellular Network - 3G",

    /// Cellular Network - 4G
    CELLULAR_4G = 6 => "Cellular Network - 4G",

    /// Cellular Network - 5G
    CELLULAR_5G = 7 => "Cellular Network - 5G",
}
