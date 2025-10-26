//! Video Placement Subtypes
//!
//! Types of video placements in accordance with updated IAB Digital Video Guidelines.
//! To be sent using `plcmt` attribute in Object:Video.

use crate::spec_list;

spec_list! {
    /// Instream: Pre-roll, mid-roll, and post-roll ads that are played before, during or after the streaming video content
    INSTREAM = 1 => "Instream",

    /// Accompanying Content: Pre-roll, mid-roll, and post-roll ads that are played before, during, or after streaming video content
    ACCOMPANYING_CONTENT = 2 => "Accompanying Content",

    /// Interstitial: Video ads that are played without video content
    INTERSTITIAL = 3 => "Interstitial",

    /// No Content/Standalone: Video ads that are played without streaming video content
    NO_CONTENT_STANDALONE = 4 => "No Content/Standalone",
}
