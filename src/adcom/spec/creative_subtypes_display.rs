//! Creative Subtypes - Display
//!
//! The various subtypes of display ad creatives.

use crate::spec_list;

spec_list! {
    /// HTML
    HTML = 1 => "HTML",

    /// AMPHTML
    AMPHTML = 2 => "AMPHTML",

    /// Structured Image Object
    STRUCTURED_IMAGE_OBJECT = 3 => "Structured Image Object",

    /// Structured Native Object
    STRUCTURED_NATIVE_OBJECT = 4 => "Structured Native Object",
}
