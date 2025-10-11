//! Advanced extension handling with custom types
//!
//! This example demonstrates:
//! - Defining custom extension structs
//! - Type-safe deserialization of extension fields
//! - Working with nested objects and arrays
//! - Combining proto and custom fields

use openrtb_rs::BidRequest;
use serde::Deserialize;

/// Custom extension struct for impression-level extensions
#[derive(Debug, Deserialize)]
struct ImpExtCustom {
    channel: i64,
    rewarded: bool,
    categories: Vec<String>,
}

/// Custom extension struct for device-level extensions
#[derive(Debug, Deserialize)]
struct DeviceExtCustom {
    device_id: String,
    fingerprint: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{
        "id": "auction-456",
        "imp": [{
            "id": "imp-1",
            "instl": 1,
            "ext": {
                "gpid": "/app/rewarded-video",
                "channel": 57,
                "rewarded": true,
                "categories": ["gaming", "casual", "puzzle"]
            }
        }],
        "device": {
            "ua": "Mozilla/5.0...",
            "ip": "10.0.0.1",
            "ext": {
                "device_id": "ABC123",
                "fingerprint": "fp_xyz789"
            }
        }
    }"#;

    println!("=== Parsing OpenRTB with Custom Extension Types ===\n");

    let request: BidRequest = serde_json::from_str(json)?;

    println!("Request ID: {}\n", request.id);

    // ===== Impression Extensions =====
    if let Some(imp) = request.imp.first() {
        if let Some(ref ext) = imp.ext {
            println!("=== Impression Extension Fields ===\n");

            // Access proto field
            println!("GPID (proto): {}", ext.gpid);

            // Convert custom fields to typed struct
            match ext.custom().as_typed::<ImpExtCustom>() {
                Ok(custom) => {
                    println!("\nCustom fields (typed struct):");
                    println!("  Channel: {}", custom.channel);
                    println!("  Rewarded: {}", custom.rewarded);
                    println!("  Categories: {:?}", custom.categories);

                    // Business logic with type safety
                    if custom.rewarded && custom.channel == 57 {
                        println!("\n✓ Special handling for rewarded video on channel 57");
                    }
                }
                Err(e) => println!("Failed to parse custom ext: {}", e),
            }

            // Alternative: Access individual fields without full deserialization
            println!("\n=== Individual Field Access ===\n");

            let channel = ext.custom().get_i64_or("channel", 0);
            println!("Channel (with default): {}", channel);

            if let Some(categories) = ext.custom().get_array_as::<String>("categories")? {
                println!("Categories count: {}", categories.len());
                for (i, cat) in categories.iter().enumerate() {
                    println!("  {}. {}", i + 1, cat);
                }
            }
        }
    }

    // ===== Device Extensions =====
    if let Some(ref device) = request.device {
        if let Some(ref ext) = device.ext {
            println!("\n=== Device Extension Fields ===\n");

            match ext.custom().as_typed::<DeviceExtCustom>() {
                Ok(custom) => {
                    println!("Device ID: {}", custom.device_id);
                    println!("Fingerprint: {}", custom.fingerprint);

                    // Use in business logic
                    if custom.device_id.starts_with("ABC") {
                        println!("\n✓ Known device identifier prefix");
                    }
                }
                Err(e) => println!("Failed to parse device ext: {}", e),
            }
        }
    }

    // ===== Nested Objects =====
    println!("\n=== Working with Nested Extension Objects ===\n");

    let nested_json = r#"{
        "id": "auction-789",
        "imp": [{
            "id": "imp-1",
            "ext": {
                "metadata": {
                    "version": "2.0",
                    "provider": "exchange-x",
                    "features": {
                        "video_enhancement": true,
                        "viewability_tracking": true
                    }
                }
            }
        }]
    }"#;

    let nested_request: BidRequest = serde_json::from_str(nested_json)?;

    if let Some(imp) = nested_request.imp.first() {
        if let Some(ref ext) = imp.ext {
            // Access nested object
            if let Some(metadata) = ext.custom().get_nested("metadata") {
                println!("Metadata version: {:?}", metadata.get_str("version"));
                println!("Provider: {:?}", metadata.get_str("provider"));

                // Access deeply nested field
                if let Some(features) = metadata.get_nested("features") {
                    println!("\nFeatures:");
                    println!(
                        "  Video enhancement: {}",
                        features.get_bool_or("video_enhancement", false)
                    );
                    println!(
                        "  Viewability tracking: {}",
                        features.get_bool_or("viewability_tracking", false)
                    );
                }
            }
        }
    }

    println!("\n=== Complete! ===");

    Ok(())
}
