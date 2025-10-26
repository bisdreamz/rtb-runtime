//! Media Ratings
//!
//! Media ratings used in describing content based on the TAG Inventory Quality Guidelines (IQG) v2.1 categorization.

use crate::spec_list;

spec_list! {
    /// All Audiences
    ALL_AUDIENCES = 1 => "All Audiences",

    /// Everyone Over Age 12
    OVER_AGE_12 = 2 => "Everyone Over Age 12",

    /// Mature Audiences
    MATURE = 3 => "Mature Audiences",
}
