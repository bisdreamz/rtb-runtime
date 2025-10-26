//! Display Placement Types
//!
//! General types of display placements; the locations where a native ad may be shown in relationship to the surrounding content.

use crate::spec_list;

spec_list! {
    /// In the feed of content (e.g., as an item inside the organic feed, grid, listing, carousel, etc.).
    IN_FEED = 1 => "In the feed",

    /// In the atomic unit of the content (e.g., in the article page or single image page).
    ATOMIC_UNIT = 2 => "In the atomic unit",

    /// Outside the core content (e.g., in the ads section on the right rail, as a banner-style placement near the content, etc.).
    OUTSIDE_CORE = 3 => "Outside the core content",

    /// Recommendation widget; most commonly presented below article content.
    RECOMMENDATION_WIDGET = 4 => "Recommendation widget",
}
