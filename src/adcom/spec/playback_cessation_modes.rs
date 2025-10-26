//! Playback Cessation Modes
//!
//! Various modes for when media playback terminates.

use crate::spec_list;

spec_list! {
    /// On Video Completion or when Terminated by User
    ON_COMPLETION = 1 => "On Video Completion or when Terminated by User",

    /// On Leaving Viewport or when Terminated by User
    ON_LEAVING_VIEWPORT = 2 => "On Leaving Viewport or when Terminated by User",

    /// On Leaving Viewport Continues as a Floating/Slider Unit until Video Completion or when Terminated by User
    FLOATING_UNTIL_COMPLETION = 3 => "On Leaving Viewport Continues as a Floating/Slider Unit until Video Completion or when Terminated by User",
}
