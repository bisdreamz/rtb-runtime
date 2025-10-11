/// Simple OpenRTB JSON serialization that handles bool as 0/1
///
/// This is a solution that post-processes JSON rather than
/// trying to intercept serde at multiple levels, since the proto
/// definition has them defined as bools for some reason
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

/// Serialize to OpenRTB JSON
pub fn to_json<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let mut json_value = serde_json::to_value(value)?;
    convert_bools_to_ints(&mut json_value);
    serde_json::to_string(&json_value)
}

/// Serialize to pretty OpenRTB JSON
pub fn to_json_pretty<T: Serialize>(value: &T) -> serde_json::Result<String> {
    let mut json_value = serde_json::to_value(value)?;
    convert_bools_to_ints(&mut json_value);
    serde_json::to_string_pretty(&json_value)
}

/// Deserialize from OpenRTB JSON (accepts both bool and 0/1 for affected fields)
pub fn from_str<T: for<'de> Deserialize<'de>>(s: &str) -> serde_json::Result<T> {
    let mut json_value: Value = serde_json::from_str(s)?;
    convert_ints_to_bools(&mut json_value);
    serde_json::from_value(json_value)
}

/// Convert specific OpenRTB boolean fields to 0/1 integers
///
/// These fields are defined as `bool` in the protobuf but specified as
/// integers with 0/1 values in the OpenRTB JSON specification.
fn convert_bools_to_ints(value: &mut Value) {
    fn walk(value: &mut Value, path: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map.iter_mut() {
                    path.push(key.clone());
                    if is_bool_int_path(path) {
                        if let Value::Bool(flag) = val {
                            let as_int = if *flag { 1 } else { 0 };
                            *val = Value::Number(Number::from(as_int));
                        }
                    }
                    walk(val, path);
                    path.pop();
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    walk(item, path);
                }
            }
            Value::Bool(flag) => {
                if is_bool_int_path(path) {
                    *value = Value::Number(Number::from(if *flag { 1 } else { 0 }));
                }
            }
            _ => {}
        }
    }

    walk(value, &mut Vec::new());
}

fn convert_ints_to_bools(value: &mut Value) {
    fn walk(value: &mut Value, path: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map.iter_mut() {
                    path.push(key.clone());
                    walk(val, path);
                    path.pop();
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    walk(item, path);
                }
            }
            Value::Number(n) => {
                if is_bool_int_path(path) {
                    if let Some(b) = number_to_bool(n) {
                        *value = Value::Bool(b);
                    }
                }
            }
            _ => {}
        }
    }

    walk(value, &mut Vec::new());
}

fn number_to_bool(number: &Number) -> Option<bool> {
    if let Some(i) = number.as_i64() {
        return Some(i != 0);
    }
    if let Some(u) = number.as_u64() {
        return Some(u != 0);
    }
    number.as_f64().map(|f| f != 0.0)
}

fn is_bool_int_path(path: &[String]) -> bool {
    BOOL_AS_INT_PATHS
        .iter()
        .any(|candidate| path_matches(path, candidate))
}

fn path_matches(path: &[String], candidate: &[&str]) -> bool {
    path.len() == candidate.len()
        && path
            .iter()
            .zip(candidate.iter())
            .all(|(actual, expected)| actual == expected)
}

const BOOL_AS_INT_PATHS: &[&[&str]] = &[
    // BidRequest fields
    &["test"],
    &["allimps"],
    // Source fields
    &["source", "fd"],
    &["source", "schain", "complete"],
    &["source", "schain", "nodes", "hp"],
    // Regs fields
    &["regs", "coppa"],
    &["regs", "gdpr"],
    &["regs", "ext", "gdpr"],
    &["regs", "ext", "s22580"],
    // Imp fields
    &["imp", "instl"],
    &["imp", "clickbrowser"],
    &["imp", "secure"],
    &["imp", "rwdd"],
    &["imp", "pmp", "private_auction"],
    &["imp", "ext", "intrinsic"],
    &["imp", "ext", "igs", "biddable"],
    &["imp", "ext", "skadn", "productpage"],
    &["imp", "ext", "skadn", "skoverlay"],
    // Banner fields
    &["imp", "banner", "topframe"],
    &["imp", "banner", "vcm"],
    // Video fields
    &["imp", "video", "skip"],
    &["imp", "video", "boxingallowed"],
    &["imp", "video", "stitched"],
    // Native fields
    &["imp", "native", "aurlsupport"],
    &["imp", "native", "durlsupport"],
    &["imp", "native", "privacy"],
    &["imp", "native", "assets", "required"],
    // Device fields
    &["device", "dnt"],
    &["device", "lmt"],
    &["device", "js"],
    &["device", "geofetch"],
    &["device", "sua", "mobile"],
    // Site fields
    &["site", "mobile"],
    &["site", "privacypolicy"],
    &["site", "content", "livestream"],
    &["site", "content", "sourcerelationship"],
    &["site", "content", "embeddable"],
    // App fields
    &["app", "privacypolicy"],
    &["app", "paid"],
    &["app", "content", "livestream"],
    &["app", "content", "sourcerelationship"],
    &["app", "content", "embeddable"],
    // Content SKAdNetwork fields
    &["seatbid", "bid", "ext", "dsa", "adrender"],
    &["seatbid", "bid", "ext", "skadn", "skoverlay", "dismissible"],
    // SeatBid
    &["seatbid", "group"],
];
