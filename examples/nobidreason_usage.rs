//! Example demonstrating usage of OpenRTB No-Bid Reason Codes

use rtb::spec::openrtb;

fn main() {
    println!("=== OpenRTB No-Bid Reason Codes ===\n");

    println!("All No-Bid Reason Codes:");
    for &value in openrtb::nobidreason::all_values() {
        println!(
            "  {} = {} ({})",
            value,
            openrtb::nobidreason::name(value).unwrap(),
            openrtb::nobidreason::description(value).unwrap()
        );
    }
    println!();

    // Example: Looking up a reason code
    let reason = 12; // Ads.txt Authorization Violation
    println!("Lookup example:");
    println!(
        "  Reason code {}: {}",
        reason,
        openrtb::nobidreason::description(reason).unwrap()
    );
    println!();

    // Example: Common reasons
    println!("Common No-Bid Scenarios:");
    println!(
        "  Invalid Request: {}",
        openrtb::nobidreason::INVALID_REQUEST
    );
    println!(
        "  Blocked Publisher: {}",
        openrtb::nobidreason::BLOCKED_PUBLISHER_OR_SITE
    );
    println!(
        "  Daily User Cap: {}",
        openrtb::nobidreason::DAILY_USER_CAP_MET
    );
    println!(
        "  Ads.txt Violation: {}",
        openrtb::nobidreason::ADS_TXT_AUTHORIZATION_VIOLATION
    );
    println!(
        "  Insufficient Time: {}",
        openrtb::nobidreason::INSUFFICIENT_AUCTION_TIME
    );
    println!();

    // Validation
    println!("Validation:");
    println!("  Is 12 valid? {}", openrtb::nobidreason::is_valid(12));
    println!("  Is 99 valid? {}", openrtb::nobidreason::is_valid(99));
    println!();

    println!(
        "Note: Values 500+ are exchange-specific and should be communicated with buyers beforehand."
    );
}
