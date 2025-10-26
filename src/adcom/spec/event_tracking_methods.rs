//! Event Tracking Methods
//!
//! Available methods of tracking of ad events.
//! Vendor specific codes may include custom measurement companies (e.g., Moat, Doubleverify, IAS, etc.).

use crate::spec_list;

spec_list! {
    /// Image-Pixel: URL provided will be inserted as a 1x1 pixel at the time of the event.
    IMAGE_PIXEL = 1 => "Image-Pixel",

    /// JavaScript: URL provided will be inserted as a JavaScript tag at the time of the event.
    JAVASCRIPT = 2 => "JavaScript",
}
