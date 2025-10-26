//! Example showing migration from deprecated OpenRTB 2.5 Video Placement Types
//! to the current AdCom 1.0 Video Plcmt Subtypes

use rtb::spec::{adcom, openrtb};

fn main() {
    println!("=== Video Placement Type Migration Example ===\n");

    println!("DEPRECATED OpenRTB 2.5 Video Placement Types:");
    println!("  In-Stream: {}", openrtb::video_placement_types::IN_STREAM);
    println!("  In-Banner: {}", openrtb::video_placement_types::IN_BANNER);
    println!(
        "  In-Article: {}",
        openrtb::video_placement_types::IN_ARTICLE
    );
    println!("  In-Feed: {}", openrtb::video_placement_types::IN_FEED);
    println!(
        "  Interstitial: {}",
        openrtb::video_placement_types::INTERSTITIAL_SLIDER_FLOATING
    );
    println!();

    println!("✓ Current AdCom 1.0 Video Plcmt Subtypes (USE THESE):");
    println!(
        "  Instream: {} = {}",
        adcom::video_plcmt_subtypes::INSTREAM,
        adcom::video_plcmt_subtypes::description(adcom::video_plcmt_subtypes::INSTREAM).unwrap()
    );
    println!(
        "  Accompanying Content: {} = {}",
        adcom::video_plcmt_subtypes::ACCOMPANYING_CONTENT,
        adcom::video_plcmt_subtypes::description(adcom::video_plcmt_subtypes::ACCOMPANYING_CONTENT)
            .unwrap()
    );
    println!(
        "  Interstitial: {} = {}",
        adcom::video_plcmt_subtypes::INTERSTITIAL,
        adcom::video_plcmt_subtypes::description(adcom::video_plcmt_subtypes::INTERSTITIAL)
            .unwrap()
    );
    println!(
        "  No Content/Standalone: {} = {}",
        adcom::video_plcmt_subtypes::NO_CONTENT_STANDALONE,
        adcom::video_plcmt_subtypes::description(
            adcom::video_plcmt_subtypes::NO_CONTENT_STANDALONE
        )
        .unwrap()
    );
    println!();

    println!("Migration Notes:");
    println!("  - OpenRTB 2.5 'placement' field → AdCom 1.0 'plcmt' field");
    println!("  - The AdCom values provide more precise categorization");
    println!("  - Use rtb::spec::adcom::video_plcmt_subtypes for new code");
}
