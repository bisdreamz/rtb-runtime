//! Agent Types
//!
//! User agent types a user identifier is from.

use crate::spec_list;

spec_list! {
    /// An ID which is tied to a specific web browser or device (cookie-based, probabilistic, or other).
    WEB_OR_DEVICE = 1 => "Web browser or device ID",

    /// In-app impressions, which will typically contain a type of device ID (or rather, the privacy-compliant versions of device IDs).
    IN_APP = 2 => "In-app device ID",

    /// A person-based ID, i.e., that is the same across devices.
    PERSON_BASED = 3 => "Person-based ID",
}
