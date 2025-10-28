use crate::bid_response::bid::AdmOneof;
use crate::bid_response::Bid;

#[derive(Debug, Clone, PartialEq)]
pub enum AdFormat {
    Banner,
    Video,
    Native
}

fn is_vast(adm: &str) -> bool {
    use memchr::memchr;

    let mut s = adm;

    // jump to first tag start to ignore leading comments/garbage if present
    if let Some(i) = memchr(b'<', s.as_bytes()) {
        s = &s[i..];
    }

    if s.starts_with("<?xml") {
        if let Some(end) = s.find("?>") {
            s = s[end + 2..].trim_start();
        }
    }

    if let Some(rest) = s.strip_prefix('<') {
        let tag_end = rest
            .find(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/')
            .unwrap_or(rest.len());
        let tag = &rest[..tag_end];
        return tag.eq_ignore_ascii_case("vast");
    }

    false
}

fn classify_adm(adm: &AdmOneof) -> Option<AdFormat> {
    match adm {
        AdmOneof::Adm(s) => {
            let trim_adm = s.trim_start_matches('\u{feff}').trim_start();

            if trim_adm.is_empty() {
                return None;
            }

            if is_vast(trim_adm) {
                Some(AdFormat::Video)
            } else if trim_adm.starts_with("{") && trim_adm.contains("native") {
                Some(AdFormat::Native)
            } else {
                Some(AdFormat::Banner)
            }
        }
        AdmOneof::AdmNative(_) => Some(AdFormat::Native),
    }
}

/// Detect the type of ['AdFormat'] the bid markup is
pub fn detect_ad_format(bid: &'_ Bid) -> Option<AdFormat> {
    let adm = match &bid.adm_oneof {
        Some(adm) => adm,
        None => return None
    };

    classify_adm(adm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NativeResponse;
    use crate::native_response::Link;

    #[test]
    fn test_detect_banner_html() {
        let bid = Bid {
            id: "banner-bid".to_string(),
            impid: "imp-1".to_string(),
            price: 1.0,
            adm_oneof: Some(AdmOneof::Adm("<div>Banner Ad</div>".to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Banner)));
    }

    #[test]
    fn test_detect_banner_html_with_attributes() {
        let bid = Bid {
            id: "banner-bid-2".to_string(),
            impid: "imp-1".to_string(),
            price: 1.0,
            adm_oneof: Some(AdmOneof::Adm(r#"<a href="https://example.com"><img src="banner.jpg"></a>"#.to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Banner)));
    }

    #[test]
    fn test_detect_video_vast_with_xml_declaration() {
        let vast_xml = r#"<?xml version="1.0"?>
<VAST version="3.0">
  <Ad>
    <InLine>...</InLine>
  </Ad>
</VAST>"#;

        let bid = Bid {
            id: "video-bid".to_string(),
            impid: "imp-1".to_string(),
            price: 2.0,
            adm_oneof: Some(AdmOneof::Adm(vast_xml.to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Video)));
    }

    #[test]
    fn test_detect_video_vast_without_xml_declaration() {
        let vast_xml = r#"<VAST version="4.0">
  <Ad>
    <InLine>...</InLine>
  </Ad>
</VAST>"#;

        let bid = Bid {
            id: "video-bid-2".to_string(),
            impid: "imp-1".to_string(),
            price: 2.0,
            adm_oneof: Some(AdmOneof::Adm(vast_xml.to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Video)));
    }

    #[test]
    fn test_detect_video_vast_lowercase() {
        let vast_xml = "<vast version=\"2.0\"><Ad></Ad></vast>";

        let bid = Bid {
            id: "video-bid-3".to_string(),
            impid: "imp-1".to_string(),
            price: 2.0,
            adm_oneof: Some(AdmOneof::Adm(vast_xml.to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Video)));
    }

    #[test]
    fn test_detect_native_from_adm_native() {
        let native_response = NativeResponse {
            ver: "1.2".to_string(),
            link: Some(Link {
                url: "https://example.com".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let bid = Bid {
            id: "native-bid".to_string(),
            impid: "imp-1".to_string(),
            price: 1.5,
            adm_oneof: Some(AdmOneof::AdmNative(native_response)),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Native)));
    }

    #[test]
    fn test_detect_native_from_json_string() {
        let native_json = r#"{"native":{"ver":"1.2","link":{"url":"https://example.com"}}}"#;

        let bid = Bid {
            id: "native-json-bid".to_string(),
            impid: "imp-1".to_string(),
            price: 1.5,
            adm_oneof: Some(AdmOneof::Adm(native_json.to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Native)));
    }

    #[test]
    fn test_detect_native_from_json_string_with_whitespace() {
        let native_json = r#"  {"native": {"ver": "1.2"}}"#;

        let bid = Bid {
            id: "native-json-bid-2".to_string(),
            impid: "imp-1".to_string(),
            price: 1.5,
            adm_oneof: Some(AdmOneof::Adm(native_json.to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Native)));
    }

    #[test]
    fn test_no_adm_returns_none() {
        let bid = Bid {
            id: "no-adm-bid".to_string(),
            impid: "imp-1".to_string(),
            price: 1.0,
            adm_oneof: None,
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(format.is_none());
    }

    #[test]
    fn test_empty_adm_returns_none() {
        let bid = Bid {
            id: "empty-adm-bid".to_string(),
            impid: "imp-1".to_string(),
            price: 1.0,
            adm_oneof: Some(AdmOneof::Adm("".to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(format.is_none());
    }

    #[test]
    fn test_whitespace_only_adm_returns_none() {
        let bid = Bid {
            id: "whitespace-bid".to_string(),
            impid: "imp-1".to_string(),
            price: 1.0,
            adm_oneof: Some(AdmOneof::Adm("   \n\t  ".to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(format.is_none());
    }

    #[test]
    fn test_bom_handling() {
        let html_with_bom = "\u{feff}<div>Banner</div>";

        let bid = Bid {
            id: "bom-bid".to_string(),
            impid: "imp-1".to_string(),
            price: 1.0,
            adm_oneof: Some(AdmOneof::Adm(html_with_bom.to_string())),
            ..Default::default()
        };

        let format = detect_ad_format(&bid);
        assert!(matches!(format, Some(AdFormat::Banner)));
    }
}