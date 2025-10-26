//! Example demonstrating usage of AdCom specification constants

use rtb::spec::adcom;

fn main() {
    println!("=== AdCom Specification Constants Usage ===\n");

    // Device Types
    println!("Device Types:");
    println!("  Value: {}", adcom::devicetype::MOBILE_TABLET_GENERAL);
    println!("  Name: {}", adcom::devicetype::name(1).unwrap());
    println!(
        "  Description: {}",
        adcom::devicetype::description(1).unwrap()
    );
    println!("  Valid: {}\n", adcom::devicetype::is_valid(1));

    // API Frameworks
    println!("API Frameworks:");
    for &value in adcom::api_frameworks::all_values() {
        println!(
            "  {} = {}",
            value,
            adcom::api_frameworks::description(value).unwrap()
        );
    }
    println!();

    // Operating Systems
    println!("Operating Systems (sample):");
    println!("  Android: {}", adcom::operating_systems::ANDROID);
    println!("  iOS: {}", adcom::operating_systems::IOS);
    println!("  Windows: {}", adcom::operating_systems::WINDOWS);
    println!();

    // Connection Types
    println!("Connection Types:");
    println!(
        "  WiFi: {} = {}",
        adcom::connection_types::WIFI,
        adcom::connection_types::description(adcom::connection_types::WIFI).unwrap()
    );
    println!(
        "  5G: {} = {}",
        adcom::connection_types::CELLULAR_5G,
        adcom::connection_types::description(adcom::connection_types::CELLULAR_5G).unwrap()
    );
    println!();

    // Creative Subtypes
    println!("Creative Subtypes - Video:");
    println!(
        "  VAST 4.3: {}",
        adcom::creative_subtypes_audio_video::VAST_4_3
    );
    println!(
        "  VAST 4.3 Wrapper: {}",
        adcom::creative_subtypes_audio_video::VAST_4_3_WRAPPER
    );
    println!();

    // DOOH
    println!("DOOH Venue Taxonomies:");
    for &value in adcom::dooh_venue_taxonomies::all_values() {
        println!(
            "  {} = {}",
            value,
            adcom::dooh_venue_taxonomies::description(value).unwrap()
        );
    }
    println!();

    // Signed integer lists
    println!("Pod Sequence (with negative values):");
    println!("  Last pod: {}", adcom::pod_sequence::LAST);
    println!("  Any pod: {}", adcom::pod_sequence::ANY);
    println!("  First pod: {}", adcom::pod_sequence::FIRST);

    println!("\n✓ All AdCom specifications accessible and functional!");
}
