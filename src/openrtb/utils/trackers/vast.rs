use anyhow::{Result, bail};
use derive_builder::Builder;
use quick_xml::events::{BytesCData, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// VAST tracking event URLs to inject into a VAST video document
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Builder)]
pub struct VastTrackers {
    /// Fired when impression occurs
    #[builder(default)]
    pub impression: Option<String>,

    /// Fired when an error occurs
    #[builder(default)]
    pub error: Option<String>,

    /// Fired when video playback starts
    #[builder(default)]
    pub start: Option<String>,

    /// Fired when video reaches 25% completion
    #[builder(default)]
    pub first_quartile: Option<String>,

    /// Fired when video reaches 50% completion
    #[builder(default)]
    pub midpoint: Option<String>,

    /// Fired when video reaches 75% completion
    #[builder(default)]
    pub third_quartile: Option<String>,

    /// Fired when video reaches 100% completion
    #[builder(default)]
    pub complete: Option<String>,

    /// Fired when video is muted
    #[builder(default)]
    pub mute: Option<String>,

    /// Fired when video is unmuted
    #[builder(default)]
    pub unmute: Option<String>,

    /// Fired when video is paused
    #[builder(default)]
    pub pause: Option<String>,

    /// Fired when video resumes from pause
    #[builder(default)]
    pub resume: Option<String>,

    /// Fired when video is rewound
    #[builder(default)]
    pub rewind: Option<String>,

    /// Fired when video is skipped
    #[builder(default)]
    pub skip: Option<String>,

    /// Fired when linear ad is closed
    #[builder(default)]
    pub close_linear: Option<String>,

    /// Fired when user clicks the ad
    #[builder(default)]
    pub click_tracking: Option<String>,
}

/// Injects tracking URLs into a VAST 2.0+ XML document.
/// Applies inline, wrapper, and linear event trackers where the spec allows, wrapping
/// all URLs in CDATA to preserve special characters.
///
/// # Errors
/// Returns an error if:
/// - XML parsing fails
/// - No InLine or Wrapper tag is found

const INLINE_ALLOWED_PREFIX: &[&[u8]] = &[
    b"AdSystem",
    b"AdTitle",
    b"AdServingId",
    b"Category",
    b"Categories",
    b"Description",
    b"Advertiser",
    b"Pricing",
    b"Survey",
    b"Error",
    b"Impression",
];

const WRAPPER_ALLOWED_PREFIX: &[&[u8]] = &[
    b"AdSystem",
    b"VASTAdTagURI",
    b"AdServingId",
    b"Category",
    b"Categories",
    b"Description",
    b"Pricing",
    b"Survey",
    b"Error",
    b"Impression",
];

#[derive(Debug)]
enum AdContainerKind {
    Inline,
    Wrapper,
}

struct AdContainerState<'a> {
    kind: AdContainerKind,
    impression: Option<&'a str>,
    error: Option<&'a str>,
    impression_injected: bool,
    error_injected: bool,
    seen_vast_ad_tag_uri: bool,
}

impl<'a> AdContainerState<'a> {
    fn new(kind: AdContainerKind, trackers: &'a VastTrackers) -> Self {
        Self {
            kind,
            impression: trackers.impression.as_deref(),
            error: trackers.error.as_deref(),
            impression_injected: false,
            error_injected: false,
            seen_vast_ad_tag_uri: false,
        }
    }

    fn has_pending(&self) -> bool {
        (self.impression.is_some() && !self.impression_injected)
            || (self.error.is_some() && !self.error_injected)
    }

    fn inject_if_needed<W: std::io::Write>(
        &mut self,
        writer: &mut Writer<W>,
        impression_injected: &mut bool,
        error_injected: &mut bool,
    ) -> Result<()> {
        if !self.has_pending() {
            return Ok(());
        }

        if let Some(url) = self.impression {
            if !self.impression_injected {
                write_element(writer, "Impression", url)?;
                self.impression_injected = true;
                *impression_injected = true;
            }
        }

        if let Some(url) = self.error {
            if !self.error_injected {
                write_element(writer, "Error", url)?;
                self.error_injected = true;
                *error_injected = true;
            }
        }

        Ok(())
    }

    fn should_skip_child(&self, child_name: &[u8]) -> bool {
        let allowed = match self.kind {
            AdContainerKind::Inline => INLINE_ALLOWED_PREFIX,
            AdContainerKind::Wrapper => WRAPPER_ALLOWED_PREFIX,
        };

        allowed.iter().any(|tag| *tag == child_name)
    }

    fn maybe_inject_before_child<W: std::io::Write>(
        &mut self,
        child_name: &[u8],
        current_depth: usize,
        writer: &mut Writer<W>,
        impression_injected: &mut bool,
        error_injected: &mut bool,
    ) -> Result<()> {
        if current_depth != 0 || !self.has_pending() {
            return Ok(());
        }

        if matches!(self.kind, AdContainerKind::Wrapper) && !self.seen_vast_ad_tag_uri {
            return Ok(());
        }

        if self.should_skip_child(child_name) {
            return Ok(());
        }

        self.inject_if_needed(writer, impression_injected, error_injected)
    }

    fn on_direct_child_end(&mut self, child_name: &[u8]) {
        if matches!(self.kind, AdContainerKind::Wrapper) && child_name == b"VASTAdTagURI" {
            self.seen_vast_ad_tag_uri = true;
        }
    }

    fn finalize<W: std::io::Write>(
        &mut self,
        writer: &mut Writer<W>,
        impression_injected: &mut bool,
        error_injected: &mut bool,
    ) -> Result<()> {
        self.inject_if_needed(writer, impression_injected, error_injected)
    }
}

pub fn inject_vast_trackers(vast_xml: &str, trackers: &VastTrackers) -> Result<String> {
    let mut reader = Reader::from_str(vast_xml);
    reader.config_mut().trim_text(true);
    reader.config_mut().expand_empty_elements = true;

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut found_ad_container = false;
    let mut impression_injected = false;
    let mut error_injected = false;

    let mut ad_state: Option<AdContainerState> = None;
    let mut ad_direct_depth: usize = 0;
    let mut non_linear_depth: usize = 0;
    let click_tracking_url = trackers.click_tracking.as_deref();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                let name = e.name();
                let name_slice = name.as_ref();

                let is_ad_container = name_slice == b"InLine" || name_slice == b"Wrapper";
                if is_ad_container {
                    found_ad_container = true;
                    writer.write_event(Event::Start(e.clone()))?;

                    let kind = if name_slice == b"InLine" {
                        AdContainerKind::Inline
                    } else {
                        AdContainerKind::Wrapper
                    };

                    ad_state = Some(AdContainerState::new(kind, trackers));
                    ad_direct_depth = 0;
                    continue;
                }

                if let Some(ref mut state) = ad_state {
                    state.maybe_inject_before_child(
                        name_slice,
                        ad_direct_depth,
                        &mut writer,
                        &mut impression_injected,
                        &mut error_injected,
                    )?;
                    ad_direct_depth += 1;
                }

                if name_slice == b"TrackingEvents" {
                    writer.write_event(Event::Start(e.clone()))?;
                    inject_video_events(&mut writer, trackers)?;
                } else if name_slice == b"NonLinear" {
                    writer.write_event(Event::Start(e.clone()))?;
                    if click_tracking_url.is_some() {
                        non_linear_depth += 1;
                    }
                } else {
                    writer.write_event(Event::Start(e.clone()))?;
                }
            }
            Event::End(ref e) => {
                let name = e.name();
                let name_slice = name.as_ref();

                let is_ad_container = name_slice == b"InLine" || name_slice == b"Wrapper";
                if is_ad_container {
                    if let Some(ref mut state) = ad_state {
                        state.finalize(
                            &mut writer,
                            &mut impression_injected,
                            &mut error_injected,
                        )?;
                    }

                    writer.write_event(Event::End(e.clone()))?;
                    ad_state = None;
                    ad_direct_depth = 0;
                    non_linear_depth = 0;
                    continue;
                }

                if let Some(ref mut state) = ad_state {
                    if ad_direct_depth == 1 {
                        state.on_direct_child_end(name_slice);
                    }

                    if ad_direct_depth > 0 {
                        ad_direct_depth -= 1;
                    }
                }

                if name_slice == b"NonLinear" {
                    if let Some(url) = click_tracking_url {
                        if non_linear_depth > 0 {
                            non_linear_depth -= 1;
                            write_element(&mut writer, "NonLinearClickTracking", url)?;
                        }
                    }
                }

                writer.write_event(Event::End(e.clone()))?;
            }
            Event::Eof => break,
            e => writer.write_event(e)?,
        }
        buf.clear();
    }

    if !found_ad_container {
        bail!("No InLine or Wrapper tag found in VAST XML");
    }

    if trackers.impression.is_some() && !impression_injected {
        bail!(
            "Impression tracker was provided but could not be injected - VAST structure may be invalid"
        );
    }
    if trackers.error.is_some() && !error_injected {
        bail!(
            "Error tracker was provided but could not be injected - VAST structure may be invalid"
        );
    }

    let output = writer.into_inner().into_inner();
    String::from_utf8(output).map_err(|e| e.into())
}

/// Helper to write a simple element with CDATA content (for URLs)
fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    content: &str,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::CData(BytesCData::new(content)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

/// Helper to write a tracking event with event attribute
fn inject_tracking_event<W: std::io::Write>(
    writer: &mut Writer<W>,
    event_type: &str,
    url: &Option<String>,
) -> Result<()> {
    if let Some(url) = url {
        let mut elem = BytesStart::new("Tracking");
        elem.push_attribute(("event", event_type));
        writer.write_event(Event::Start(elem))?;
        writer.write_event(Event::CData(BytesCData::new(url)))?;
        writer.write_event(Event::End(BytesEnd::new("Tracking")))?;
    }
    Ok(())
}

/// Inject all video event trackers into Linear TrackingEvents
fn inject_video_events<W: std::io::Write>(
    writer: &mut Writer<W>,
    trackers: &VastTrackers,
) -> Result<()> {
    inject_tracking_event(writer, "start", &trackers.start)?;
    inject_tracking_event(writer, "firstQuartile", &trackers.first_quartile)?;
    inject_tracking_event(writer, "midpoint", &trackers.midpoint)?;
    inject_tracking_event(writer, "thirdQuartile", &trackers.third_quartile)?;
    inject_tracking_event(writer, "complete", &trackers.complete)?;
    inject_tracking_event(writer, "mute", &trackers.mute)?;
    inject_tracking_event(writer, "unmute", &trackers.unmute)?;
    inject_tracking_event(writer, "pause", &trackers.pause)?;
    inject_tracking_event(writer, "resume", &trackers.resume)?;
    inject_tracking_event(writer, "rewind", &trackers.rewind)?;
    inject_tracking_event(writer, "skip", &trackers.skip)?;
    inject_tracking_event(writer, "closeLinear", &trackers.close_linear)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VAST_INLINE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<VAST version="4.0">
  <Ad id="12345">
    <InLine>
      <AdSystem>Test Ad System</AdSystem>
      <AdTitle>Test Ad</AdTitle>
      <Creatives>
        <Creative>
          <Linear>
            <Duration>00:00:15</Duration>
            <TrackingEvents>
            </TrackingEvents>
            <MediaFiles>
              <MediaFile>https://example.com/video.mp4</MediaFile>
            </MediaFiles>
          </Linear>
        </Creative>
      </Creatives>
    </InLine>
  </Ad>
</VAST>"#;

    const VAST_INLINE_EMPTY_TRACKING: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<VAST version="4.0">
  <Ad id="emptyTracking">
    <InLine>
      <AdSystem>Test Ad System</AdSystem>
      <AdTitle>Test Ad</AdTitle>
      <Creatives>
        <Creative>
          <Linear>
            <Duration>00:00:15</Duration>
            <TrackingEvents/>
            <MediaFiles>
              <MediaFile>https://example.com/video.mp4</MediaFile>
            </MediaFiles>
          </Linear>
        </Creative>
      </Creatives>
    </InLine>
  </Ad>
</VAST>"#;

    const VAST_WRAPPER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<VAST version="4.0">
  <Ad id="wrapper123">
    <Wrapper>
      <AdSystem>Wrapper System</AdSystem>
      <VASTAdTagURI>https://example.com/vast.xml</VASTAdTagURI>
      <Creatives>
        <Creative>
          <Linear>
            <TrackingEvents>
            </TrackingEvents>
          </Linear>
        </Creative>
      </Creatives>
    </Wrapper>
  </Ad>
</VAST>"#;

    #[test]
    fn test_inject_impression_inline() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp?id=123".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        assert!(result.contains(
            "<Impression><![CDATA[https://billing.example.com/imp?id=123]]></Impression>"
        ));
        assert!(result.contains("<InLine>"));
    }

    #[test]
    fn test_impression_preserves_header_order() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp?id=order".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        let ad_system_pos = result.find("<AdSystem>").unwrap();
        let impression_pos = result
            .find("<Impression><![CDATA[https://billing.example.com/imp?id=order]]></Impression>")
            .unwrap();
        let creatives_pos = result.find("<Creatives>").unwrap();

        assert!(ad_system_pos < impression_pos);
        assert!(impression_pos < creatives_pos);
    }

    #[test]
    fn test_inject_impression_wrapper() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp?id=456".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_WRAPPER, &trackers).unwrap();

        assert!(result.contains(
            "<Impression><![CDATA[https://billing.example.com/imp?id=456]]></Impression>"
        ));
        assert!(result.contains("<Wrapper>"));
    }

    #[test]
    fn test_wrapper_impression_after_vast_ad_tag_uri() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some(
                "https://billing.example.com/imp?id=wrapper-order".to_string(),
            ))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_WRAPPER, &trackers).unwrap();

        let vast_uri_pos = result.find("<VASTAdTagURI>").unwrap();
        let impression_pos = result
            .find("<Impression><![CDATA[https://billing.example.com/imp?id=wrapper-order]]></Impression>")
            .unwrap();

        assert!(vast_uri_pos < impression_pos);
    }

    #[test]
    fn test_inject_error_tracker() {
        let trackers = VastTrackersBuilder::default()
            .error(Some("https://billing.example.com/error?id=123".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        assert!(
            result.contains("<Error><![CDATA[https://billing.example.com/error?id=123]]></Error>")
        );
    }

    #[test]
    fn test_inject_tracking_events() {
        let trackers = VastTrackersBuilder::default()
            .start(Some("https://billing.example.com/start".to_string()))
            .first_quartile(Some("https://billing.example.com/q1".to_string()))
            .midpoint(Some("https://billing.example.com/mid".to_string()))
            .third_quartile(Some("https://billing.example.com/q3".to_string()))
            .complete(Some("https://billing.example.com/complete".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        assert!(result.contains(
            r#"<Tracking event="start"><![CDATA[https://billing.example.com/start]]></Tracking>"#
        ));
        assert!(result.contains(r#"<Tracking event="firstQuartile"><![CDATA[https://billing.example.com/q1]]></Tracking>"#));
        assert!(result.contains(
            r#"<Tracking event="midpoint"><![CDATA[https://billing.example.com/mid]]></Tracking>"#
        ));
        assert!(result.contains(r#"<Tracking event="thirdQuartile"><![CDATA[https://billing.example.com/q3]]></Tracking>"#));
        assert!(result.contains(r#"<Tracking event="complete"><![CDATA[https://billing.example.com/complete]]></Tracking>"#));
    }

    #[test]
    fn test_tracking_events_self_closing_injects() {
        let trackers = VastTrackersBuilder::default()
            .start(Some("https://billing.example.com/start".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE_EMPTY_TRACKING, &trackers).unwrap();

        assert!(result.contains(
            r#"<Tracking event="start"><![CDATA[https://billing.example.com/start]]></Tracking>"#
        ));
    }

    #[test]
    fn test_inject_interaction_events() {
        let trackers = VastTrackersBuilder::default()
            .mute(Some("https://billing.example.com/mute".to_string()))
            .pause(Some("https://billing.example.com/pause".to_string()))
            .skip(Some("https://billing.example.com/skip".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        assert!(result.contains(
            r#"<Tracking event="mute"><![CDATA[https://billing.example.com/mute]]></Tracking>"#
        ));
        assert!(result.contains(
            r#"<Tracking event="pause"><![CDATA[https://billing.example.com/pause]]></Tracking>"#
        ));
        assert!(result.contains(
            r#"<Tracking event="skip"><![CDATA[https://billing.example.com/skip]]></Tracking>"#
        ));
    }

    #[test]
    fn test_inject_all_trackers() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp".to_string()))
            .error(Some("https://billing.example.com/error".to_string()))
            .start(Some("https://billing.example.com/start".to_string()))
            .complete(Some("https://billing.example.com/complete".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        assert!(
            result.contains("<Impression><![CDATA[https://billing.example.com/imp]]></Impression>")
        );
        assert!(result.contains("<Error><![CDATA[https://billing.example.com/error]]></Error>"));
        assert!(result.contains(
            r#"<Tracking event="start"><![CDATA[https://billing.example.com/start]]></Tracking>"#
        ));
        assert!(result.contains(r#"<Tracking event="complete"><![CDATA[https://billing.example.com/complete]]></Tracking>"#));
    }

    #[test]
    fn test_empty_trackers() {
        let trackers = VastTrackersBuilder::default().build().unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        assert!(result.contains("<InLine>"));
        assert!(!result.contains("<Impression>"));
    }

    #[test]
    fn test_error_no_inline_or_wrapper() {
        let invalid_vast = r#"<?xml version="1.0"?>
<VAST version="4.0">
  <Ad id="123">
    <InvalidTag></InvalidTag>
  </Ad>
</VAST>"#;

        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://example.com/imp".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(invalid_vast, &trackers);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No InLine or Wrapper tag found")
        );
    }

    #[test]
    fn test_valid_xml_output() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        let mut reader = Reader::from_str(&result);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("XML parsing error: {}", e),
            }
            buf.clear();
        }
    }

    #[test]
    fn test_url_with_ampersands_uses_cdata() {
        // URLs with & and other special chars should be in CDATA, not entity-encoded
        let url = "https://billing.example.com/imp?id=123&pub=456&cat=video";
        let trackers = VastTrackersBuilder::default()
            .impression(Some(url.to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_INLINE, &trackers).unwrap();

        assert!(result.contains(&format!("<![CDATA[{}]]>", url)));
        assert!(!result.contains("&amp;"));
    }

    const VAST_NONLINEAR: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<VAST version="4.0">
  <Ad id="nonlinear1">
    <InLine>
      <AdSystem>Test Ad System</AdSystem>
      <AdTitle>NonLinear Overlay Ad</AdTitle>
      <Creatives>
        <Creative>
          <NonLinearAds>
            <NonLinear>
              <StaticResource>https://example.com/overlay.jpg</StaticResource>
              <NonLinearClickThrough>https://example.com/click</NonLinearClickThrough>
            </NonLinear>
          </NonLinearAds>
        </Creative>
      </Creatives>
    </InLine>
  </Ad>
</VAST>"#;

    #[test]
    fn test_nonlinear_impression() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_NONLINEAR, &trackers).unwrap();

        assert!(
            result.contains("<Impression><![CDATA[https://billing.example.com/imp]]></Impression>")
        );
    }

    #[test]
    fn test_nonlinear_click_tracking() {
        let trackers = VastTrackersBuilder::default()
            .click_tracking(Some("https://billing.example.com/click".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_NONLINEAR, &trackers).unwrap();

        let click_through_pos = result.find("<NonLinearClickThrough>").unwrap();
        let click_tracking_markup = "<NonLinearClickTracking><![CDATA[https://billing.example.com/click]]></NonLinearClickTracking>";
        let click_tracking_pos = result.find(click_tracking_markup).unwrap();

        assert!(click_through_pos < click_tracking_pos);
        assert!(result.contains(click_tracking_markup));
    }

    #[test]
    fn test_nonlinear_with_impression_and_click() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp".to_string()))
            .click_tracking(Some("https://billing.example.com/click".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_NONLINEAR, &trackers).unwrap();

        assert!(
            result.contains("<Impression><![CDATA[https://billing.example.com/imp]]></Impression>")
        );
        assert!(result.contains("<NonLinearClickTracking><![CDATA[https://billing.example.com/click]]></NonLinearClickTracking>"));
    }

    #[test]
    fn test_video_events_skipped_for_nonlinear() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp".to_string()))
            .start(Some("https://billing.example.com/start".to_string()))
            .complete(Some("https://billing.example.com/complete".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_NONLINEAR, &trackers).unwrap();

        assert!(
            result.contains("<Impression><![CDATA[https://billing.example.com/imp]]></Impression>")
        );

        assert!(!result.contains("event=\"start\""));
        assert!(!result.contains("event=\"complete\""));
    }

    const VAST_WITH_EXISTING_TRACKERS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<VAST version="4.0">
  <Ad id="12345">
    <InLine>
      <AdSystem>Test Ad System</AdSystem>
      <Impression><![CDATA[https://existing.com/imp]]></Impression>
      <Creatives>
        <Creative>
          <Linear>
            <TrackingEvents>
              <Tracking event="start"><![CDATA[https://existing.com/start]]></Tracking>
              <Tracking event="complete"><![CDATA[https://existing.com/complete]]></Tracking>
            </TrackingEvents>
          </Linear>
        </Creative>
      </Creatives>
    </InLine>
  </Ad>
</VAST>"#;

    #[test]
    fn test_adds_to_existing_trackers() {
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp".to_string()))
            .start(Some("https://billing.example.com/start".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_WITH_EXISTING_TRACKERS, &trackers).unwrap();

        assert!(result.contains("<![CDATA[https://billing.example.com/imp]]>"));
        assert!(result.contains("<![CDATA[https://billing.example.com/start]]>"));

        assert!(result.contains("<![CDATA[https://existing.com/imp]]>"));
        assert!(result.contains("<![CDATA[https://existing.com/start]]>"));
        assert!(result.contains("<![CDATA[https://existing.com/complete]]>"));

        assert_eq!(result.matches("<Impression>").count(), 2);
    }

    #[test]
    fn test_error_no_tracking_events() {
        let vast_no_tracking = r#"<?xml version="1.0"?>
<VAST version="4.0">
  <Ad id="123">
    <InLine>
      <AdSystem>Test</AdSystem>
      <Creatives>
        <Creative>
          <Linear>
            <MediaFiles>
              <MediaFile>https://example.com/video.mp4</MediaFile>
            </MediaFiles>
          </Linear>
        </Creative>
      </Creatives>
    </InLine>
  </Ad>
</VAST>"#;

        let trackers = VastTrackersBuilder::default()
            .start(Some("https://example.com/start".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(vast_no_tracking, &trackers);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.contains("event=\"start\""));
    }
}
