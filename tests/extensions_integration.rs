//! Integration tests for OpenRTB extension field handling.
//!
//! These tests verify that the extension system correctly handles both
//! proto-defined fields and custom fields in real OpenRTB JSON.

use openrtb_rs::BidRequest;
use prost::Message;

/// Test parsing a real OpenRTB bid request with custom extension fields
#[test]
fn test_parse_bid_request_with_extensions() {
    // Real bid request JSON from test_data/
    let json = include_str!("../test_data/sample_bid_request.json");

    // Parse with standard serde_json
    let request: BidRequest = serde_json::from_str(json).expect("Failed to parse bid request");

    // Verify basic fields
    assert_eq!(request.id, "68e3c2bd_0b23dea1f5ebac8487bca87e9d673710_0-a5");
    assert_eq!(request.imp.len(), 1);

    // Verify boolean fields are parsed correctly (they're 0/1 in JSON)
    assert_eq!(request.test, false); // "test": 0
    if let Some(ref source) = request.source {
        assert_eq!(source.fd, true); // "fd": 1
    }
}

#[test]
fn test_extension_fields_custom_access() {
    let json = include_str!("../test_data/sample_bid_request.json");
    let request: BidRequest = serde_json::from_str(json).expect("Failed to parse");

    // Access custom extension field from imp.ext
    if let Some(ref imp) = request.imp.first() {
        if let Some(ref ext) = imp.ext {
            // Custom field: "channel": 546
            let channel = ext.custom().get_i64("channel");
            assert_eq!(channel, Some(546), "Expected channel field with value 546");
        }
    }
}

#[test]
fn test_extension_fields_proto_access() {
    // Create a simple bid request with known proto ext fields
    let json = r#"{
        "id": "test-123",
        "imp": [{
            "id": "imp-1",
            "ext": {
                "gpid": "test-gpid-value",
                "custom_field": 42
            }
        }]
    }"#;

    let request: BidRequest = serde_json::from_str(json).expect("Failed to parse");

    let imp = &request.imp[0];
    if let Some(ref ext) = imp.ext {
        // Proto field access via Deref
        assert_eq!(ext.gpid, "test-gpid-value");

        // Custom field access via .custom()
        let custom_value = ext.custom().get_i64("custom_field");
        assert_eq!(custom_value, Some(42));
    }
}

#[test]
fn test_serialization_roundtrip() {
    let json = include_str!("../test_data/sample_bid_request.json");

    // Parse
    let request: BidRequest = serde_json::from_str(json).expect("Failed to parse");

    // Serialize back
    let serialized = serde_json::to_string(&request).expect("Failed to serialize");

    // Parse again
    let roundtrip: BidRequest =
        serde_json::from_str(&serialized).expect("Failed to parse after roundtrip");

    // Verify key fields match
    assert_eq!(request.id, roundtrip.id);
    assert_eq!(request.imp.len(), roundtrip.imp.len());

    // Verify custom extension field is preserved
    if let Some(ref imp) = roundtrip.imp.first() {
        if let Some(ref ext) = imp.ext {
            let channel = ext.custom().get_i64("channel");
            assert_eq!(
                channel,
                Some(546),
                "Custom field should be preserved in roundtrip"
            );
        }
    }
}

#[test]
fn test_bool_fields_serialize_as_integers() {
    // Test with non-default boolean values (1/true)
    // Note: Default values (0/false) are omitted per protobuf JSON spec
    let json = r#"{
        "id": "test",
        "imp": [{
            "id": "imp-1",
            "secure": 1,
            "instl": 1,
            "banner": {
                "topframe": 1
            }
        }],
        "test": 1
    }"#;

    let request: BidRequest = serde_json::from_str(json).expect("Failed to parse");

    // Serialize
    let serialized = serde_json::to_string(&request).expect("Failed to serialize");

    println!("Serialized JSON: {}", serialized);

    // Verify non-default booleans (true) are serialized as integer 1, not "true"
    assert!(
        serialized.contains("\"test\":1") || serialized.contains("\"test\": 1"),
        "test field should serialize as integer 1, not boolean true. Got: {}",
        serialized
    );

    assert!(
        serialized.contains("\"secure\":1") || serialized.contains("\"secure\": 1"),
        "secure field should serialize as integer 1, not boolean true. Got: {}",
        serialized
    );

    assert!(
        serialized.contains("\"instl\":1") || serialized.contains("\"instl\": 1"),
        "instl field should serialize as integer 1, not boolean true. Got: {}",
        serialized
    );

    assert!(
        serialized.contains("\"topframe\":1") || serialized.contains("\"topframe\": 1"),
        "topframe field should serialize as integer 1, not boolean true. Got: {}",
        serialized
    );

    // Should NOT contain true/false literals
    assert!(
        !serialized.contains("true") && !serialized.contains("false"),
        "Should not contain boolean literals (true/false). Got: {}",
        serialized
    );
}

#[test]
fn test_nested_extension_objects() {
    let json = r#"{
        "id": "test",
        "imp": [{
            "id": "imp-1",
            "ext": {
                "metadata": {
                    "version": "1.0",
                    "count": 5
                }
            }
        }]
    }"#;

    let request: BidRequest = serde_json::from_str(json).expect("Failed to parse");

    let imp = &request.imp[0];
    if let Some(ref ext) = imp.ext {
        // Access nested object
        let metadata = ext
            .custom()
            .get_nested("metadata")
            .expect("Should have metadata field");

        assert_eq!(metadata.get_str("version"), Some("1.0"));
        assert_eq!(metadata.get_i64("count"), Some(5));
    }
}

#[test]
fn test_typed_extension_deserialization() {
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct CustomImpExt {
        channel: i64,
        enabled: bool,
    }

    let json = r#"{
        "id": "test",
        "imp": [{
            "id": "imp-1",
            "ext": {
                "channel": 42,
                "enabled": true
            }
        }]
    }"#;

    let request: BidRequest = serde_json::from_str(json).expect("Failed to parse");

    let imp = &request.imp[0];
    if let Some(ref ext) = imp.ext {
        // Deserialize entire ext object into typed struct
        let typed: CustomImpExt = ext
            .custom()
            .as_typed()
            .expect("Should deserialize to CustomImpExt");

        assert_eq!(typed.channel, 42);
        assert_eq!(typed.enabled, true);
    }
}

/// Test protobuf encoding/decoding roundtrip for extension fields.
///
/// This test verifies that:
/// 1. Proto-defined fields in ext objects are preserved through protobuf encoding
/// 2. Custom fields are NOT preserved (expected behavior - they don't exist in proto schema)
/// 3. The same BidRequest type works with both JSON and protobuf
#[test]
fn test_protobuf_roundtrip_with_extensions() {
    // Create a bid request with both proto-defined and custom ext fields
    let json = r#"{
        "id": "proto-test-123",
        "imp": [{
            "id": "imp-1",
            "bidfloor": 1.5,
            "ext": {
                "gpid": "/homepage/banner",
                "channel": 42,
                "custom_field": "should-not-survive-protobuf"
            }
        }],
        "test": 0
    }"#;

    // Parse from JSON (captures both proto and custom fields)
    let original: BidRequest = serde_json::from_str(json).expect("Failed to parse JSON");

    // Verify custom field exists in original
    if let Some(ref imp) = original.imp.first() {
        if let Some(ref ext) = imp.ext {
            assert_eq!(ext.gpid, "/homepage/banner", "Proto field should exist");
            assert_eq!(
                ext.custom().get_i64("channel"),
                Some(42),
                "Custom field should exist in JSON-parsed request"
            );
        }
    }

    // Encode to protobuf bytes
    let mut buf = Vec::new();
    original
        .encode(&mut buf)
        .expect("Failed to encode to protobuf");

    println!("Encoded protobuf size: {} bytes", buf.len());

    // Decode from protobuf bytes
    let decoded: BidRequest = BidRequest::decode(&buf[..]).expect("Failed to decode from protobuf");

    // Verify basic fields are preserved
    assert_eq!(
        decoded.id, "proto-test-123",
        "Request ID should be preserved"
    );
    assert_eq!(decoded.imp.len(), 1, "Impression count should be preserved");
    assert_eq!(decoded.test, false, "Test flag should be preserved");

    // Verify impression fields are preserved
    if let Some(ref imp) = decoded.imp.first() {
        assert_eq!(imp.id, "imp-1", "Impression ID should be preserved");
        assert_eq!(imp.bidfloor, 1.5, "Bid floor should be preserved");

        // Verify extension behavior
        if let Some(ref ext) = imp.ext {
            // Proto-defined field SHOULD be preserved
            assert_eq!(
                ext.gpid, "/homepage/banner",
                "Proto-defined field (gpid) should be preserved through protobuf"
            );

            // Custom fields should NOT be preserved (expected behavior)
            assert_eq!(
                ext.custom().get_i64("channel"),
                None,
                "Custom field should NOT be preserved through protobuf encoding"
            );
            assert_eq!(
                ext.custom().len(),
                0,
                "No custom fields should exist after protobuf roundtrip"
            );
        }
    }
}

/// Test that the same BidRequest type works seamlessly with both JSON and protobuf.
///
/// This demonstrates the unified API where users don't need to care about the
/// transport layer - the same field access patterns work regardless.
#[test]
fn test_unified_api_json_and_protobuf() {
    // Helper function that works with BidRequest regardless of source
    fn extract_channel(request: &BidRequest) -> Option<i64> {
        request
            .imp
            .first()?
            .ext
            .as_ref()?
            .custom()
            .get_i64("channel")
    }

    // Same function for proto-defined fields
    fn extract_gpid(request: &BidRequest) -> Option<String> {
        request.imp.first()?.ext.as_ref().map(|e| e.gpid.clone())
    }

    let json = r#"{
        "id": "unified-test",
        "imp": [{
            "id": "imp-1",
            "ext": {
                "gpid": "/app/video",
                "channel": 99
            }
        }]
    }"#;

    // From JSON - both proto and custom fields work
    let json_request: BidRequest = serde_json::from_str(json).unwrap();
    assert_eq!(extract_channel(&json_request), Some(99));
    assert_eq!(extract_gpid(&json_request), Some("/app/video".to_string()));

    // From protobuf - only proto fields work
    let mut buf = Vec::new();
    json_request.encode(&mut buf).unwrap();
    let proto_request: BidRequest = BidRequest::decode(&buf[..]).unwrap();

    assert_eq!(extract_channel(&proto_request), None); // Custom field lost
    assert_eq!(extract_gpid(&proto_request), Some("/app/video".to_string())); // Proto field preserved

    // The API is exactly the same for both! Users just check the Option result.
}
