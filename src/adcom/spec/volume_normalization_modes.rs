//! Volume Normalization Modes
//!
//! The types of volume normalization modes, typically for audio.

use crate::spec_list;

spec_list! {
    /// None
    NONE = 0 => "None",

    /// Ad Volume Average Normalized to Content
    AD_VOLUME_AVERAGE = 1 => "Ad Volume Average Normalized to Content",

    /// Ad Volume Peak Normalized to Content
    AD_VOLUME_PEAK = 2 => "Ad Volume Peak Normalized to Content",

    /// Ad Loudness Normalized to Content
    AD_LOUDNESS = 3 => "Ad Loudness Normalized to Content",

    /// Custom Volume Normalization
    CUSTOM = 4 => "Custom Volume Normalization",
}
