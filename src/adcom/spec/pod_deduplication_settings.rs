//! Pod Deduplication Settings
//!
//! Various pod deduplication settings.

use crate::spec_list;

spec_list! {
    /// Deduplicated on adomain
    ADOMAIN = 1 => "Deduplicated on adomain",

    /// Deduplicated on IAB Tech Lab Content Taxonomy
    IAB_CONTENT_TAXONOMY = 2 => "Deduplicated on IAB Tech Lab Content Taxonomy",

    /// Deduplicated on creative ID
    CREATIVE_ID = 3 => "Deduplicated on creative ID",

    /// Deduplicated on mediafile URL
    MEDIAFILE_URL = 4 => "Deduplicated on mediafile URL",

    /// No deduplication
    NO_DEDUP = 5 => "No deduplication",
}
