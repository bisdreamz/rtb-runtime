/// Simple OpenRTB JSON serialization that handles bool as 0/1
///
/// This is a solution that post-processes JSON rather than
/// trying to intercept serde at multiple levels, since the proto
/// definition has them defined as bools for some reason
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use smallvec::SmallVec;

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
    fn walk(value: &mut Value, path: &mut SmallVec<[Option<Key>; 8]>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map.iter_mut() {
                    let key_id = key_id(key);
                    path.push(key_id);

                    if let Value::Bool(flag) = val {
                        if is_bool_int_path(path) {
                            let as_int = if *flag { 1 } else { 0 };
                            *val = Value::Number(Number::from(as_int));
                        }
                    } else {
                        walk(val, path);
                    }

                    path.pop();
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    walk(item, path);
                }
            }
            _ => {}
        }
    }

    let mut path = SmallVec::<[Option<Key>; 8]>::new();
    walk(value, &mut path);
}

fn convert_ints_to_bools(value: &mut Value) {
    fn walk(value: &mut Value, path: &mut SmallVec<[Option<Key>; 8]>) {
        match value {
            Value::Object(map) => {
                for (key, val) in map.iter_mut() {
                    let key_id = key_id(key);
                    path.push(key_id);

                    if let Value::Number(n) = val {
                        if is_bool_int_path(path) {
                            if let Some(b) = number_to_bool(n) {
                                *val = Value::Bool(b);
                            }
                        }
                    } else {
                        walk(val, path);
                    }

                    path.pop();
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    walk(item, path);
                }
            }
            _ => {}
        }
    }

    let mut path = SmallVec::<[Option<Key>; 8]>::new();
    walk(value, &mut path);
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

fn is_bool_int_path(path: &[Option<Key>]) -> bool {
    if path.iter().any(|segment| segment.is_none()) {
        return false;
    }

    match path {
        [Some(Key::Test)]
        | [Some(Key::AllImps)]
        | [Some(Key::Source), Some(Key::Fd)]
        | [Some(Key::Source), Some(Key::Schain), Some(Key::Complete)]
        | [
            Some(Key::Source),
            Some(Key::Schain),
            Some(Key::Nodes),
            Some(Key::Hp),
        ]
        | [Some(Key::Regs), Some(Key::Coppa)]
        | [Some(Key::Regs), Some(Key::Gdpr)]
        | [Some(Key::Regs), Some(Key::Ext), Some(Key::Gdpr)]
        | [Some(Key::Regs), Some(Key::Ext), Some(Key::S22580)]
        | [Some(Key::Imp), Some(Key::Instl)]
        | [Some(Key::Imp), Some(Key::ClickBrowser)]
        | [Some(Key::Imp), Some(Key::Secure)]
        | [Some(Key::Imp), Some(Key::Rwdd)]
        | [Some(Key::Imp), Some(Key::Pmp), Some(Key::PrivateAuction)]
        | [Some(Key::Imp), Some(Key::Ext), Some(Key::Intrinsic)]
        | [
            Some(Key::Imp),
            Some(Key::Ext),
            Some(Key::Igs),
            Some(Key::Biddable),
        ]
        | [
            Some(Key::Imp),
            Some(Key::Ext),
            Some(Key::Skadn),
            Some(Key::ProductPage),
        ]
        | [
            Some(Key::Imp),
            Some(Key::Ext),
            Some(Key::Skadn),
            Some(Key::Skoverlay),
        ]
        | [Some(Key::Imp), Some(Key::Banner), Some(Key::Topframe)]
        | [Some(Key::Imp), Some(Key::Banner), Some(Key::Vcm)]
        | [Some(Key::Imp), Some(Key::Video), Some(Key::Skip)]
        | [Some(Key::Imp), Some(Key::Video), Some(Key::BoxingAllowed)]
        | [Some(Key::Imp), Some(Key::Video), Some(Key::Stitched)]
        | [Some(Key::Imp), Some(Key::Native), Some(Key::AurlSupport)]
        | [Some(Key::Imp), Some(Key::Native), Some(Key::DurlSupport)]
        | [Some(Key::Imp), Some(Key::Native), Some(Key::Privacy)]
        | [
            Some(Key::Imp),
            Some(Key::Native),
            Some(Key::Assets),
            Some(Key::Required),
        ]
        | [Some(Key::Device), Some(Key::Dnt)]
        | [Some(Key::Device), Some(Key::Lmt)]
        | [Some(Key::Device), Some(Key::Js)]
        | [Some(Key::Device), Some(Key::GeoFetch)]
        | [Some(Key::Device), Some(Key::Sua), Some(Key::Mobile)]
        | [Some(Key::Site), Some(Key::Mobile)]
        | [Some(Key::Site), Some(Key::PrivacyPolicy)]
        | [Some(Key::Site), Some(Key::Content), Some(Key::LiveStream)]
        | [
            Some(Key::Site),
            Some(Key::Content),
            Some(Key::SourceRelationship),
        ]
        | [Some(Key::Site), Some(Key::Content), Some(Key::Embeddable)]
        | [Some(Key::App), Some(Key::PrivacyPolicy)]
        | [Some(Key::App), Some(Key::Paid)]
        | [Some(Key::App), Some(Key::Content), Some(Key::LiveStream)]
        | [
            Some(Key::App),
            Some(Key::Content),
            Some(Key::SourceRelationship),
        ]
        | [Some(Key::App), Some(Key::Content), Some(Key::Embeddable)]
        | [
            Some(Key::SeatBid),
            Some(Key::Bid),
            Some(Key::Ext),
            Some(Key::Dsa),
            Some(Key::AdRender),
        ]
        | [
            Some(Key::SeatBid),
            Some(Key::Bid),
            Some(Key::Ext),
            Some(Key::Skadn),
            Some(Key::Skoverlay),
            Some(Key::Dismissible),
        ]
        | [Some(Key::SeatBid), Some(Key::Group)] => true,
        _ => false,
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Key {
    Test,
    AllImps,
    Source,
    Fd,
    Schain,
    Complete,
    Nodes,
    Hp,
    Regs,
    Coppa,
    Gdpr,
    Ext,
    S22580,
    Imp,
    Instl,
    ClickBrowser,
    Secure,
    Rwdd,
    Pmp,
    PrivateAuction,
    Intrinsic,
    Igs,
    Biddable,
    Skadn,
    ProductPage,
    Skoverlay,
    Banner,
    Topframe,
    Vcm,
    Video,
    Skip,
    BoxingAllowed,
    Stitched,
    Native,
    AurlSupport,
    DurlSupport,
    Privacy,
    Assets,
    Required,
    Device,
    Dnt,
    Lmt,
    Js,
    GeoFetch,
    Sua,
    Mobile,
    Site,
    PrivacyPolicy,
    Content,
    LiveStream,
    SourceRelationship,
    Embeddable,
    App,
    Paid,
    SeatBid,
    Bid,
    Dsa,
    AdRender,
    Dismissible,
    Group,
}

fn key_id(key: &str) -> Option<Key> {
    match key {
        "test" => Some(Key::Test),
        "allimps" => Some(Key::AllImps),
        "source" => Some(Key::Source),
        "fd" => Some(Key::Fd),
        "schain" => Some(Key::Schain),
        "complete" => Some(Key::Complete),
        "nodes" => Some(Key::Nodes),
        "hp" => Some(Key::Hp),
        "regs" => Some(Key::Regs),
        "coppa" => Some(Key::Coppa),
        "gdpr" => Some(Key::Gdpr),
        "ext" => Some(Key::Ext),
        "s22580" => Some(Key::S22580),
        "imp" => Some(Key::Imp),
        "instl" => Some(Key::Instl),
        "clickbrowser" => Some(Key::ClickBrowser),
        "secure" => Some(Key::Secure),
        "rwdd" => Some(Key::Rwdd),
        "pmp" => Some(Key::Pmp),
        "private_auction" => Some(Key::PrivateAuction),
        "intrinsic" => Some(Key::Intrinsic),
        "igs" => Some(Key::Igs),
        "biddable" => Some(Key::Biddable),
        "skadn" => Some(Key::Skadn),
        "productpage" => Some(Key::ProductPage),
        "skoverlay" => Some(Key::Skoverlay),
        "banner" => Some(Key::Banner),
        "topframe" => Some(Key::Topframe),
        "vcm" => Some(Key::Vcm),
        "video" => Some(Key::Video),
        "skip" => Some(Key::Skip),
        "boxingallowed" => Some(Key::BoxingAllowed),
        "stitched" => Some(Key::Stitched),
        "native" => Some(Key::Native),
        "aurlsupport" => Some(Key::AurlSupport),
        "durlsupport" => Some(Key::DurlSupport),
        "privacy" => Some(Key::Privacy),
        "assets" => Some(Key::Assets),
        "required" => Some(Key::Required),
        "device" => Some(Key::Device),
        "dnt" => Some(Key::Dnt),
        "lmt" => Some(Key::Lmt),
        "js" => Some(Key::Js),
        "geofetch" => Some(Key::GeoFetch),
        "sua" => Some(Key::Sua),
        "mobile" => Some(Key::Mobile),
        "site" => Some(Key::Site),
        "privacypolicy" => Some(Key::PrivacyPolicy),
        "content" => Some(Key::Content),
        "livestream" => Some(Key::LiveStream),
        "sourcerelationship" => Some(Key::SourceRelationship),
        "embeddable" => Some(Key::Embeddable),
        "app" => Some(Key::App),
        "paid" => Some(Key::Paid),
        "seatbid" => Some(Key::SeatBid),
        "bid" => Some(Key::Bid),
        "dsa" => Some(Key::Dsa),
        "adrender" => Some(Key::AdRender),
        "dismissible" => Some(Key::Dismissible),
        "group" => Some(Key::Group),
        _ => None,
    }
}
