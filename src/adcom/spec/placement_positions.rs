//! Placement Positions
//!
//! Placement positions as a relative measure of visibility or prominence.
//! Values derived from the TAG Inventory Quality Guidelines (IQG).

use crate::spec_list;

spec_list! {
    /// Unknown
    UNKNOWN = 0 => "Unknown",

    /// Above The Fold
    ABOVE_THE_FOLD = 1 => "Above The Fold",

    /// Locked (i.e., fixed position)
    LOCKED = 2 => "Locked (i.e., fixed position)",

    /// Below The Fold
    BELOW_THE_FOLD = 3 => "Below The Fold",

    /// Header
    HEADER = 4 => "Header",

    /// Footer
    FOOTER = 5 => "Footer",

    /// Sidebar
    SIDEBAR = 6 => "Sidebar",

    /// Fullscreen
    FULLSCREEN = 7 => "Fullscreen",
}
