//! Feed Types
//!
//! Types of feeds for audio.

use crate::spec_list;

spec_list! {
    /// Music streaming service
    MUSIC_STREAMING = 1 => "Music streaming service",

    /// FM/AM broadcast (live content broadcast over the air but also available via online streaming)
    FM_AM_BROADCAST = 2 => "FM/AM broadcast",

    /// Podcast (original, pre-recorded content distributed as episodes in a series)
    PODCAST = 3 => "Podcast",

    /// Catch-up radio (recorded segment of a radio show that was originally broadcast live)
    CATCH_UP_RADIO = 4 => "Catch-up radio",

    /// Web radio (live content only available via online streaming, not as AM/FM broadcast)
    WEB_RADIO = 5 => "Web radio",

    /// Video game (background audio in video games)
    VIDEO_GAME = 6 => "Video game",

    /// Text to speech (audio books, website plugin that can read article)
    TEXT_TO_SPEECH = 7 => "Text to speech",
}
