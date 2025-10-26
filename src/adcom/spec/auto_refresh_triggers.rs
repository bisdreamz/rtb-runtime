//! Auto Refresh Triggers
//!
//! Triggers that result in an ad slot refreshing.

use crate::spec_list;

spec_list! {
    /// UNKNOWN
    UNKNOWN = 0 => "Unknown",

    /// User Action: Refresh triggered by user-initiated action such as scrolling.
    USER_ACTION = 1 => "User Action",

    /// Event: Event-driven content change. For example, ads refresh when the football game score changes on the page.
    EVENT = 2 => "Event",

    /// Time: Time-based refresh. Ads refresh on a predefined time interval even without user activity.
    TIME = 3 => "Time",
}
