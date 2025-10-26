//! Native Image Asset Types
//!
//! Common image asset types. This list is non-exhaustive and is intended to be expanded over time.
//! Size recommendations are noted as "maximum height or width of at least".

use crate::spec_list;

spec_list! {
    /// Icon: Icon image. Maximum height at least 50 device independent pixels (DIPS); aspect ratio 1:1.
    ICON = 1 => "Icon",

    /// Main: Large image preview for the ad. At least one of 2 size variants required.
    MAIN = 3 => "Main",
}
