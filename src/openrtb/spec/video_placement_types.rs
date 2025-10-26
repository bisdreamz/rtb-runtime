//! Video Placement Types
//!
//! **DEPRECATED:** This list is from OpenRTB 2.5 and is deprecated in favor of the AdCom 1.0
//! `plcmt` field. Use `rtb::spec::adcom::video_plcmt_subtypes` instead.
//!
//! The various types of video placements derived largely from the IAB Digital Video Guidelines.

use crate::spec_list;

spec_list! {
    /// In-Stream: Played before, during or after the streaming video content that the consumer has requested (e.g., Pre-roll, Mid-roll, Post-roll).
    #[deprecated(note = "Use rtb::spec::adcom::video_plcmt_subtypes instead")]
    IN_STREAM = 1 => "In-Stream",

    /// In-Banner: Exists within a web banner that leverages the banner space to deliver a video experience as opposed to another static or rich media format. The format relies on the existence of display ad inventory on the page for its delivery.
    #[deprecated(note = "Use rtb::spec::adcom::video_plcmt_subtypes instead")]
    IN_BANNER = 2 => "In-Banner",

    /// In-Article: Loads and plays dynamically between paragraphs of editorial content; existing as a standalone branded message.
    #[deprecated(note = "Use rtb::spec::adcom::video_plcmt_subtypes instead")]
    IN_ARTICLE = 3 => "In-Article",

    /// In-Feed: Found in content, social, or product feeds.
    #[deprecated(note = "Use rtb::spec::adcom::video_plcmt_subtypes instead")]
    IN_FEED = 4 => "In-Feed",

    /// Interstitial/Slider/Floating: Covers the entire or a portion of screen area, but is always on screen while displayed (i.e. cannot be scrolled out of view). Note that a full-screen interstitial (e.g., in mobile) can be distinguished from a floating/slider unit by the imp.instl field.
    #[deprecated(note = "Use rtb::spec::adcom::video_plcmt_subtypes instead")]
    INTERSTITIAL_SLIDER_FLOATING = 5 => "Interstitial/Slider/Floating",
}
