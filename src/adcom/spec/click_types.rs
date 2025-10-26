//! Click Types
//!
//! Types of creative activation (i.e., click) behavior types.

use crate::spec_list;

spec_list! {
    /// Non-Clickable
    NON_CLICKABLE = 0 => "Non-Clickable",

    /// Clickable - Details Unknown
    CLICKABLE_UNKNOWN = 1 => "Clickable - Details Unknown",

    /// Clickable - Embedded Browser/Webview
    CLICKABLE_EMBEDDED = 2 => "Clickable - Embedded Browser/Webview",

    /// Clickable - Native Browser
    CLICKABLE_NATIVE = 3 => "Clickable - Native Browser",
}
