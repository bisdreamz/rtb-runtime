//! Basic usage example for openrtb-rs
//!
//! This example demonstrates:
//! - Parsing OpenRTB JSON with standard serde_json
//! - Accessing proto-defined fields
//! - Accessing custom extension fields
//! - Serializing back to JSON

use openrtb_rs::BidRequest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sample OpenRTB bid request JSON (with custom extension fields)
    let json = r#"{
        "id": "auction-123",
        "imp": [{
            "id": "imp-1",
            "secure": 1,
            "instl": 0,
            "bidfloor": 0.5,
            "banner": {
                "w": 300,
                "h": 250,
                "topframe": 1
            },
            "ext": {
                "gpid": "/homepage/banner",
                "channel": 42,
                "enabled": true
            }
        }],
        "device": {
            "ua": "Mozilla/5.0...",
            "ip": "192.168.1.1",
            "dnt": 0,
            "lmt": 0
        },
        "test": 0
    }"#;

    println!("=== Parsing OpenRTB JSON ===\n");

    // Parse using standard serde_json - no custom wrappers needed!
    let request: BidRequest = serde_json::from_str(json)?;

    println!("Request ID: {}", request.id);
    println!("Number of impressions: {}", request.imp.len());
    println!("Test mode: {}", request.test);

    println!("\n=== Accessing Impression Fields ===\n");

    if let Some(imp) = request.imp.first() {
        println!("Impression ID: {}", imp.id);
        println!("Secure: {}", imp.secure);
        println!("Interstitial: {}", imp.instl);
        println!("Bid floor: {}", imp.bidfloor);

        if let Some(ref banner) = imp.banner {
            println!("Banner dimensions: {}x{}", banner.w, banner.h);
            println!("Top frame: {}", banner.topframe);
        }

        println!("\n=== Accessing Extension Fields ===\n");

        if let Some(ref ext) = imp.ext {
            // Proto-defined field (with autocomplete!)
            println!("GPID (proto field): {}", ext.gpid);

            // Custom fields (dynamic access)
            if let Some(channel) = ext.custom().get_i64("channel") {
                println!("Channel (custom field): {}", channel);
            }

            if let Some(enabled) = ext.custom().get_bool("enabled") {
                println!("Enabled (custom field): {}", enabled);
            }

            // Check if a field exists
            if ext.custom().contains("channel") {
                println!("✓ Channel field is present");
            }
        }
    }

    println!("\n=== Serializing Back to JSON ===\n");

    // Serialize back using standard serde_json
    let serialized = serde_json::to_string_pretty(&request)?;

    println!("{}", serialized);

    // Verify boolean fields are serialized as 0/1
    if serialized.contains("\"secure\": 1") || serialized.contains("\"secure\":1") {
        println!("\n✓ Boolean fields correctly serialized as integers (0/1)");
    }

    // Verify custom fields are preserved
    if serialized.contains("\"channel\"") {
        println!("✓ Custom extension fields preserved in serialization");
    }

    println!("\n=== Complete! ===");

    Ok(())
}
