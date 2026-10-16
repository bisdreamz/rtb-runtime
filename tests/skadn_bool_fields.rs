//! SKAdNetwork flags use OpenRTB's 0/1 encoding like every other bool field.

use rtb::{BidRequest, BidResponse};
use serde_json::{Value, json};

#[test]
fn skadn_request_flags_accept_and_emit_integers() {
    let request: BidRequest = serde_json::from_value(json!({
        "id": "skadn",
        "imp": [{
            "id": "1",
            "ext": { "skadn": { "versions": ["4.0"], "productpage": 1, "skoverlay": 0 } }
        }]
    }))
    .expect("skadn 0/1 flags should deserialize");

    let skadn: Value = serde_json::to_value(&request).unwrap()["imp"][0]["ext"]["skadn"].clone();
    assert_eq!(skadn["productpage"], json!(1));
    assert!(skadn.get("skoverlay").is_none_or(|v| v == &json!(0)));
}

#[test]
fn skoverlay_dismissible_accepts_and_emits_integers() {
    let response: BidResponse = serde_json::from_value(json!({
        "id": "skadn",
        "seatbid": [{
            "bid": [{
                "id": "1",
                "impid": "1",
                "price": 1.0,
                "ext": { "skadn": { "version": "4.0", "skoverlay": { "dismissible": 1 } } }
            }]
        }]
    }))
    .expect("skoverlay 0/1 flag should deserialize");

    let skoverlay: Value = serde_json::to_value(&response).unwrap()["seatbid"][0]["bid"][0]["ext"]
        ["skadn"]["skoverlay"]
        .clone();
    assert_eq!(skoverlay["dismissible"], json!(1));
}
