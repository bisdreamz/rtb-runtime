//! Location Types
//!
//! Options to indicate how the geographic information was determined.

use crate::spec_list;

spec_list! {
    /// GPS/Location Services
    GPS_LOCATION_SERVICES = 1 => "GPS/Location Services",

    /// IP Address
    IP_ADDRESS = 2 => "IP Address",

    /// User Provided (e.g., registration data)
    USER_PROVIDED = 3 => "User Provided",
}
