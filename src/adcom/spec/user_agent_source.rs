//! User-Agent Source
//!
//! The possible sources for User-Agent metadata.

use crate::spec_list;

spec_list! {
    /// Unspecified/unknown
    UNSPECIFIED = 0 => "Unspecified/unknown",

    /// User-Agent Client Hints (only low-entropy headers were available)
    CLIENT_HINTS_LOW_ENTROPY = 1 => "User-Agent Client Hints (only low-entropy headers were available)",

    /// User-Agent Client Hints (with high-entropy headers available)
    CLIENT_HINTS_HIGH_ENTROPY = 2 => "User-Agent Client Hints (with high-entropy headers available)",

    /// Parsed from User-Agent header (the same string carried by the ua field)
    PARSED_FROM_UA_HEADER = 3 => "Parsed from User-Agent header",
}
