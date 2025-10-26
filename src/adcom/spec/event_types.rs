//! Event Types
//!
//! Types of ad events available for tracking.
//! These types refer to the actual event, timing, etc.; not the method of firing.
//! Scripts that are performing measurement should be deployed at the "loaded" event.

use crate::spec_list;

spec_list! {
    /// loaded: Delivered as a part of the creative markup. Creative may be pre-cached or pre-loaded; prior to initial rendering.
    LOADED = 1 => "loaded",

    /// impression: Ad impression per IAB/MRC Ad Impression Measurement Guidelines.
    IMPRESSION = 2 => "impression",

    /// viewable-mrc50: Visible impression using MRC definition of 50% in view for 1 second.
    VIEWABLE_MRC50 = 3 => "viewable-mrc50",

    /// viewable-mrc100: 100% in view for 1 second (i.e., the GroupM standard).
    VIEWABLE_MRC100 = 4 => "viewable-mrc100",

    /// viewable-video50: Visible impression for video using MRC definition of 50% in view for 2 seconds.
    VIEWABLE_VIDEO50 = 5 => "viewable-video50",
}
