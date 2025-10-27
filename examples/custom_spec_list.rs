//! Example showing how users can extend the library with their own specification lists
//! using the exported spec_list! and spec_list_i32! macros.

// Import the macros from the rtb crate
use rtb::{spec_list, spec_list_i32};

// Example 1: Custom exchange-specific no-bid reason codes (500+)
pub mod custom_nobid_reasons {
    use rtb::spec_list;

    spec_list! {
        /// Exchange maintenance window
        EXCHANGE_MAINTENANCE = 500 => "Exchange Maintenance",

        /// Advertiser budget exhausted
        BUDGET_EXHAUSTED = 501 => "Advertiser Budget Exhausted",

        /// Creative not approved
        CREATIVE_NOT_APPROVED = 502 => "Creative Not Approved",

        /// Geo-targeting mismatch
        GEO_TARGETING_MISMATCH = 503 => "Geo-targeting Mismatch",

        /// Brand safety violation
        BRAND_SAFETY_VIOLATION = 504 => "Brand Safety Violation",
    }
}

// Example 2: Custom device types for specialized environments
pub mod custom_device_types {
    use rtb::spec_list;

    spec_list! {
        /// Smart Refrigerator
        SMART_REFRIGERATOR = 100 => "Smart Refrigerator",

        /// Smart Mirror
        SMART_MIRROR = 101 => "Smart Mirror",

        /// In-Car Entertainment System
        IN_CAR_ENTERTAINMENT = 102 => "In-Car Entertainment System",

        /// Elevator Display
        ELEVATOR_DISPLAY = 103 => "Elevator Display",
    }
}

// Example 3: Using spec_list_i32 for values with negative numbers
pub mod custom_priority_levels {
    use rtb::spec_list_i32;

    spec_list_i32! {
        /// Low priority (backfill)
        LOW_PRIORITY = -1 => "Low Priority",

        /// Normal priority
        NORMAL_PRIORITY = 0 => "Normal Priority",

        /// High priority (premium)
        HIGH_PRIORITY = 1 => "High Priority",

        /// Critical priority (sponsorship)
        CRITICAL_PRIORITY = 2 => "Critical Priority",
    }
}

fn main() {
    println!("=== Custom Specification Lists Example ===\n");

    // Using custom no-bid reasons
    println!("Custom Exchange No-Bid Reasons (500+):");
    for &value in custom_nobid_reasons::all_values() {
        println!(
            "  {} = {}",
            value,
            custom_nobid_reasons::description(value).unwrap()
        );
    }
    println!();

    // Using custom device types
    println!("Custom Device Types:");
    println!(
        "  Smart Refrigerator: {} = {}",
        custom_device_types::SMART_REFRIGERATOR,
        custom_device_types::description(custom_device_types::SMART_REFRIGERATOR).unwrap()
    );
    println!(
        "  In-Car Entertainment: {} = {}",
        custom_device_types::IN_CAR_ENTERTAINMENT,
        custom_device_types::description(custom_device_types::IN_CAR_ENTERTAINMENT).unwrap()
    );
    println!();

    // Using custom priority levels with negative values
    println!("Custom Priority Levels:");
    for &value in custom_priority_levels::all_values() {
        println!(
            "  {} = {}",
            value,
            custom_priority_levels::description(value).unwrap()
        );
    }
    println!();

    // Demonstrating lookup and validation
    println!("Lookup Examples:");
    println!(
        "  Reason 502: {}",
        custom_nobid_reasons::description(502).unwrap()
    );
    println!("  Is 504 valid? {}", custom_nobid_reasons::is_valid(504));
    println!("  Is 999 valid? {}", custom_nobid_reasons::is_valid(999));
    println!();

    println!("✓ All custom spec lists accessible with helper functions!");
    println!("\nNote: The spec_list! and spec_list_i32! macros are exported");
    println!("      from the rtb crate for users to create their own lists.");
}
