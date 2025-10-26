//! Companion Types
//!
//! Options to indicate markup types allowed for companion ads that apply to video and audio ads.
//! This table is derived from VAST 2.0+ and DAAST 1.0+ specifications.

use crate::spec_list;

spec_list! {
    /// Static Resource
    STATIC_RESOURCE = 1 => "Static Resource",

    /// HTML Resource
    HTML_RESOURCE = 2 => "HTML Resource",

    /// iframe Resource
    IFRAME_RESOURCE = 3 => "iframe Resource",
}
