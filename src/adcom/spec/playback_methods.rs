//! Playback Methods
//!
//! The various media playback methods.

use crate::spec_list;

spec_list! {
    /// Initiates on Page Load with Sound On
    PAGE_LOAD_SOUND_ON = 1 => "Initiates on Page Load with Sound On",

    /// Initiates on Page Load with Sound Off by Default
    PAGE_LOAD_SOUND_OFF = 2 => "Initiates on Page Load with Sound Off by Default",

    /// Initiates on Click with Sound On
    CLICK_SOUND_ON = 3 => "Initiates on Click with Sound On",

    /// Initiates on Mouse-Over with Sound On
    MOUSE_OVER_SOUND_ON = 4 => "Initiates on Mouse-Over with Sound On",

    /// Initiates on Entering Viewport with Sound On
    VIEWPORT_SOUND_ON = 5 => "Initiates on Entering Viewport with Sound On",

    /// Initiates on Entering Viewport with Sound Off by Default
    VIEWPORT_SOUND_OFF = 6 => "Initiates on Entering Viewport with Sound Off by Default",

    /// Continuous Playback - Media playback is set to play additional media automatically without user interaction
    CONTINUOUS = 7 => "Continuous Playback",
}
