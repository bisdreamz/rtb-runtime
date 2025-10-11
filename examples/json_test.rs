use openrtb_rs::bid_request;
use openrtb_rs::openrtb::*;
use openrtb_rs::openrtb_json;

fn main() {
    // Create a BidRequest
    let request = BidRequest {
        id: "test-123".to_string(),
        imp: vec![bid_request::Imp {
            id: "imp-1".to_string(),
            secure: true,
            bidfloor: 1.5,
            ..Default::default()
        }],
        test: false,
        at: 2,
        tmax: 100,
        ..Default::default()
    };

    println!("=== Using OpenRTB-compliant serialization ===\n");

    // Serialize to JSON using OpenRTB wrapper (bools as 0/1)
    match openrtb_json::to_json_pretty(&request) {
        Ok(json) => {
            println!("Serialized BidRequest to JSON:");
            println!("{}", json);

            // Check the secure field format
            if json.contains("\"secure\": 1") || json.contains("\"secure\":1") {
                println!("\n✅ Good: 'secure' field is serialized as integer (0/1)");
            } else {
                println!("\n⚠️ WARNING: 'secure' field is NOT serialized as integer");
            }

            // Try to deserialize back using OpenRTB wrapper
            match openrtb_json::from_str::<BidRequest>(&json) {
                Ok(deserialized) => {
                    println!("✅ Successfully deserialized back from JSON");
                    println!("Request ID: {}", deserialized.id);
                    println!("Test mode: {}", deserialized.test);
                    if !deserialized.imp.is_empty() {
                        println!("First impression secure: {}", deserialized.imp[0].secure);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to deserialize: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to serialize: {}", e);
        }
    }

    // Test with real OpenRTB JSON (with integer values)
    println!("\n--- Testing with OpenRTB-style JSON (integers for bools) ---");
    let openrtb_json = r#"{
        "id": "auction-456",
        "test": 0,
        "at": 2,
        "tmax": 120,
        "imp": [
            {
                "id": "imp-001",
                "secure": 1,
                "bidfloor": 2.0
            }
        ]
    }"#;

    match openrtb_json::from_str::<BidRequest>(openrtb_json) {
        Ok(deserialized) => {
            println!("✅ Successfully parsed OpenRTB JSON with integer fields");
            println!("Request ID: {}", deserialized.id);
            println!("Test mode: {}", deserialized.test);
            if !deserialized.imp.is_empty() {
                println!("First impression secure: {}", deserialized.imp[0].secure);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse OpenRTB JSON: {}", e);
        }
    }

    // Test with mixed format JSON (some bools, some ints)
    println!("\n--- Testing with mixed format JSON ---");
    let mixed_json = r#"{
        "id": "auction-789",
        "test": true,
        "at": 2,
        "tmax": 150,
        "imp": [
            {
                "id": "imp-002",
                "secure": 0,
                "instl": true,
                "bidfloor": 3.0
            }
        ]
    }"#;

    match openrtb_json::from_str::<BidRequest>(mixed_json) {
        Ok(deserialized) => {
            println!("✅ Successfully parsed mixed format JSON (accepts both true/false and 0/1)");
            println!("Request ID: {}", deserialized.id);
            println!("Test mode: {}", deserialized.test);
            if !deserialized.imp.is_empty() {
                println!("First impression secure: {}", deserialized.imp[0].secure);
                println!("First impression instl: {}", deserialized.imp[0].instl);
            }

            // Re-serialize to show it always outputs 0/1
            if let Ok(json) = openrtb_json::to_json_pretty(&deserialized) {
                println!("\nRe-serialized (always outputs 0/1 for bools):");
                println!("{}", json);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse mixed format JSON: {}", e);
        }
    }
}
