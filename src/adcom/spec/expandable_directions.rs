//! Expandable Directions
//!
//! Directions in which an expandable ad may expand, given the positioning of the ad unit on the page and constraints imposed by the content.

use crate::spec_list;

spec_list! {
    /// Left
    LEFT = 1 => "Left",

    /// Right
    RIGHT = 2 => "Right",

    /// Up
    UP = 3 => "Up",

    /// Down
    DOWN = 4 => "Down",

    /// Full Screen
    FULL_SCREEN = 5 => "Full Screen",

    /// Resize/Minimize (make smaller)
    RESIZE_MINIMIZE = 6 => "Resize/Minimize",
}
