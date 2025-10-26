//! Creative Attributes
//!
//! Standard list of creative attributes that can describe an actual ad or restrictions relative to a given placement.

use crate::spec_list;

spec_list! {
    /// Audio Ad (Autoplay)
    AUDIO_AD_AUTOPLAY = 1 => "Audio Ad (Autoplay)",

    /// Audio Ad (User Initiated)
    AUDIO_AD_USER_INITIATED = 2 => "Audio Ad (User Initiated)",

    /// Expandable (Automatic)
    EXPANDABLE_AUTOMATIC = 3 => "Expandable (Automatic)",

    /// Expandable (User Initiated - Click)
    EXPANDABLE_USER_CLICK = 4 => "Expandable (User Initiated - Click)",

    /// Expandable (User Initiated - Rollover)
    EXPANDABLE_USER_ROLLOVER = 5 => "Expandable (User Initiated - Rollover)",

    /// In-Banner Video Ad (Autoplay)
    IN_BANNER_VIDEO_AUTOPLAY = 6 => "In-Banner Video Ad (Autoplay)",

    /// In-Banner Video Ad (User Initiated)
    IN_BANNER_VIDEO_USER_INITIATED = 7 => "In-Banner Video Ad (User Initiated)",

    /// Pop (e.g., Over, Under, or Upon Exit)
    POP = 8 => "Pop (e.g., Over, Under, or Upon Exit)",

    /// Provocative or Suggestive Imagery
    PROVOCATIVE_SUGGESTIVE = 9 => "Provocative or Suggestive Imagery",

    /// Shaky, Flashing, Flickering, Extreme Animation, Smileys
    SHAKY_FLASHING = 10 => "Shaky, Flashing, Flickering, Extreme Animation, Smileys",

    /// Surveys
    SURVEYS = 11 => "Surveys",

    /// Text Only
    TEXT_ONLY = 12 => "Text Only",

    /// User Interactive (e.g., Embedded Games)
    USER_INTERACTIVE = 13 => "User Interactive (e.g., Embedded Games)",

    /// Windows Dialog or Alert Style
    WINDOWS_DIALOG = 14 => "Windows Dialog or Alert Style",

    /// Has Audio On/Off Button
    HAS_AUDIO_ON_OFF = 15 => "Has Audio On/Off Button",

    /// Ad Provides Skip Button (e.g. VPAID-rendered skip button on pre-roll video)
    HAS_SKIP_BUTTON = 16 => "Ad Provides Skip Button",

    /// Adobe Flash
    ADOBE_FLASH = 17 => "Adobe Flash",

    /// Responsive; Sizeless; Fluid (i.e., creatives that dynamically resize to environment)
    RESPONSIVE = 18 => "Responsive; Sizeless; Fluid",
}
