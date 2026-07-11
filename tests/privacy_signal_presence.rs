//! Contract tests for privacy flags whose omission has distinct OpenRTB meaning.

use prost::Message;
use rtb::BidRequest;

#[test]
#[allow(deprecated)]
fn explicit_zero_is_distinct_from_omission() {
    let explicit: BidRequest = serde_json::from_str(
        r#"{
            "id": "explicit-zero",
            "device": { "dnt": 0, "lmt": 0 },
            "regs": { "gdpr": 0, "ext": { "gdpr": 0 } }
        }"#,
    )
    .expect("explicit privacy zeroes should deserialize");

    let device = explicit.device.as_ref().expect("device should exist");
    assert_eq!(device.dnt, Some(false));
    assert_eq!(device.lmt, Some(false));

    let regs = explicit.regs.as_ref().expect("regs should exist");
    assert_eq!(regs.gdpr, Some(false));
    assert_eq!(
        regs.ext.as_ref().expect("regs.ext should exist").gdpr,
        Some(false)
    );

    let omitted: BidRequest = serde_json::from_str(
        r#"{
            "id": "omitted",
            "device": {},
            "regs": { "ext": {} }
        }"#,
    )
    .expect("omitted privacy fields should deserialize");

    let device = omitted.device.as_ref().expect("device should exist");
    assert_eq!(device.dnt, None);
    assert_eq!(device.lmt, None);

    let regs = omitted.regs.as_ref().expect("regs should exist");
    assert_eq!(regs.gdpr, None);
    assert_eq!(regs.ext.as_ref().expect("regs.ext should exist").gdpr, None);
}

#[test]
#[allow(deprecated)]
fn json_roundtrip_preserves_explicit_zero_and_one() {
    let request: BidRequest = serde_json::from_str(
        r#"{
            "id": "json-roundtrip",
            "device": { "dnt": 0, "lmt": 1 },
            "regs": { "gdpr": 0, "ext": { "gdpr": 1 } }
        }"#,
    )
    .expect("privacy flags should deserialize");

    let json = serde_json::to_value(request).expect("privacy flags should serialize");
    assert_eq!(
        json.pointer("/device/dnt").and_then(|v| v.as_i64()),
        Some(0)
    );
    assert_eq!(
        json.pointer("/device/lmt").and_then(|v| v.as_i64()),
        Some(1)
    );
    assert_eq!(json.pointer("/regs/gdpr").and_then(|v| v.as_i64()), Some(0));
    assert_eq!(
        json.pointer("/regs/ext/gdpr").and_then(|v| v.as_i64()),
        Some(1)
    );
}

#[test]
fn omitted_fields_stay_omitted_when_serialized() {
    let request: BidRequest = serde_json::from_str(
        r#"{
            "id": "omitted-roundtrip",
            "device": {},
            "regs": {}
        }"#,
    )
    .expect("request should deserialize");

    let json = serde_json::to_value(request).expect("request should serialize");
    assert!(json.pointer("/device/dnt").is_none());
    assert!(json.pointer("/device/lmt").is_none());
    assert!(json.pointer("/regs/gdpr").is_none());
}

#[test]
#[allow(deprecated)]
fn protobuf_roundtrip_preserves_presence() {
    let request: BidRequest = serde_json::from_str(
        r#"{
            "id": "protobuf-roundtrip",
            "device": { "dnt": 0 },
            "regs": { "gdpr": 1 }
        }"#,
    )
    .expect("request should deserialize");

    let encoded = request.encode_to_vec();
    let decoded = BidRequest::decode(encoded.as_slice()).expect("protobuf should decode");

    assert_eq!(
        decoded.device.expect("device should exist").dnt,
        Some(false)
    );
    assert_eq!(decoded.regs.expect("regs should exist").gdpr, Some(true));
}
