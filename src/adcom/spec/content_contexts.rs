//! Content Contexts
//!
//! Options for indicating the type of content being used or consumed by the user in which ads may appear.
//! Values derived from the TAG Inventory Quality Guidelines (IQG).

use crate::spec_list;

spec_list! {
    /// Video (i.e., video file or stream such as Internet TV broadcasts)
    VIDEO = 1 => "Video",

    /// Game (i.e., an interactive software game)
    GAME = 2 => "Game",

    /// Music (i.e., audio file or stream such as Internet radio broadcasts)
    MUSIC = 3 => "Music",

    /// Application (i.e., an interactive software application)
    APPLICATION = 4 => "Application",

    /// Text (i.e., primarily textual document such as a web page, eBook, or news article)
    TEXT = 5 => "Text",

    /// Other (i.e., none of the other categories applies)
    OTHER = 6 => "Other",

    /// Unknown
    UNKNOWN = 7 => "Unknown",
}
