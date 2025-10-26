//! Start Delay Modes
//!
//! The various options for the video or audio start delay.
//! If the start delay value is greater than 0, then the position is mid-roll and the value indicates the start delay.

use crate::spec_list_i32;

spec_list_i32! {
    /// Pre-Roll
    PRE_ROLL = 0 => "Pre-Roll",

    /// Generic Mid-Roll
    GENERIC_MID_ROLL = -1 => "Generic Mid-Roll",

    /// Generic Post-Roll
    GENERIC_POST_ROLL = -2 => "Generic Post-Roll",
}
