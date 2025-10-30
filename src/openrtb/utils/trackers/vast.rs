use derive_builder::Builder;
use quick_xml::events::{BytesCData, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use anyhow::{bail, Error};

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

/// Injects tracking URLs into a VAST 2.0+ XML document
///
/// Trackers are injected where applicable based on ad structure:
/// - **Impression**: Injected at InLine/Wrapper level (applies to all ad types)
/// - **Error**: Injected at InLine/Wrapper level (applies to all ad types)
/// - **Video events** (start, quartiles, complete, etc.): Injected into Linear <TrackingEvents> if present
/// - **Click tracking**: Injected into NonLinear ads as <NonLinearClickTracking> if present
///
/// If a tracker type doesn't apply to the ad structure (e.g., video events for NonLinear-only ads),
/// it's silently skipped. This allows you to provide a complete set of trackers without knowing
/// the ad type in advance.
///
/// All URLs are wrapped in CDATA sections to prevent entity encoding of special characters like `&`.
///
/// Compatible with VAST 2.0, 3.0, and 4.0 specifications.
///
/// # Errors
/// Returns an error if:
/// - XML parsing fails
/// - No InLine or Wrapper tag is found
pub fn inject_vast_trackers(vast_xml: &str, trackers: &VastTrackers) -> Result<String, Error> {
    let mut reader = Reader::from_str(vast_xml);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();

    let mut found_ad_container = false;
    let mut injected_impression = false;
    let mut injected_error = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                process_start_event(
                    e,
                    &mut writer,
                    trackers,
                    &mut found_ad_container,
                    &mut injected_impression,
                    &mut injected_error,
                )?;
            }
            Event::End(ref e) => {
                writer.write_event(Event::End(e.clone()))?;
            }
            Event::Eof => break,
            e => writer.write_event(e)?,
        }
        buf.clear();
    }

    // Validate injection success
    if !found_ad_container {
        bail!("No InLine or Wrapper tag found in VAST XML");
    }

    // Validate that applicable trackers were injected
    // Impression and Error apply to ALL ad types, so if provided they MUST be injected
    if trackers.impression.is_some() && !injected_impression {
        bail!(
            "Impression tracker was provided but could not be injected - VAST structure may be invalid"
        );
    }
    if trackers.error.is_some() && !injected_error {
        bail!(
            "Error tracker was provided but could not be injected - VAST structure may be invalid"
        );
    }

    // Note: Video events and click tracking are ad-type specific
    // They're injected where applicable, silently skipped if not:
    // - Video events: Injected if TrackingEvents found (Linear ads)
    // - Click tracking: Injected if NonLinear found

    let output = writer.into_inner().into_inner();
    String::from_utf8(output).map_err(|e| e.into())
}

/// Helper to write a simple element with CDATA content (for URLs)
fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    tag: &str,
    content: &str,
) -> Result<(), Error> {
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
) -> Result<(), Error> {
    if let Some(url) = url {
        let mut elem = BytesStart::new("Tracking");
        elem.push_attribute(("event", event_type));
        writer.write_event(Event::Start(elem))?;
        writer.write_event(Event::CData(BytesCData::new(url)))?;
        writer.write_event(Event::End(BytesEnd::new("Tracking")))?;
    }
    Ok(())
}

/// Inject impression and error trackers at InLine/Wrapper level
fn inject_ad_level_trackers<W: std::io::Write>(
    writer: &mut Writer<W>,
    trackers: &VastTrackers,
) -> Result<(bool, bool), Error> {
    let mut injected_impression = false;
    let mut injected_error = false;

    if let Some(url) = &trackers.impression {
        write_element(writer, "Impression", url)?;
        injected_impression = true;
    }

    if let Some(url) = &trackers.error {
        write_element(writer, "Error", url)?;
        injected_error = true;
    }

    Ok((injected_impression, injected_error))
}

/// Inject all video event trackers into Linear TrackingEvents
fn inject_video_events<W: std::io::Write>(
    writer: &mut Writer<W>,
    trackers: &VastTrackers,
) -> Result<(), Error> {
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

/// Process a single XML start event and inject trackers where appropriate
fn process_start_event<W: std::io::Write>(
    event: &BytesStart,
    writer: &mut Writer<W>,
    trackers: &VastTrackers,
    found_ad_container: &mut bool,
    injected_impression: &mut bool,
    injected_error: &mut bool,
) -> Result<(), Error> {
    let name = event.name();

    if name.as_ref() == b"InLine" || name.as_ref() == b"Wrapper" {
        *found_ad_container = true;
        writer.write_event(Event::Start(event.clone()))?;
        let (imp, err) = inject_ad_level_trackers(writer, trackers)?;
        *injected_impression = imp;
        *injected_error = err;
    } else if name.as_ref() == b"TrackingEvents" {
        writer.write_event(Event::Start(event.clone()))?;
        inject_video_events(writer, trackers)?;
    } else if name.as_ref() == b"NonLinear" {
        writer.write_event(Event::Start(event.clone()))?;
        if let Some(url) = &trackers.click_tracking {
            write_element(writer, "NonLinearClickTracking", url)?;
        }
    } else {
        writer.write_event(Event::Start(event.clone()))?;
    }

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

        // Should succeed but not inject anything
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

        // Should be parseable XML - try to read all events without error
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

        // Should contain CDATA with unescaped ampersands
        assert!(result.contains(&format!("<![CDATA[{}]]>", url)));
        // Should NOT contain entity-encoded ampersands
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

        // Impression should be injected at InLine level (works for both Linear and NonLinear)
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

        // Click tracking should be injected into NonLinear element
        assert!(result.contains("<NonLinearClickTracking><![CDATA[https://billing.example.com/click]]></NonLinearClickTracking>"));
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
        // Video events should be silently skipped for NonLinear-only ads
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp".to_string()))
            .start(Some("https://billing.example.com/start".to_string()))
            .complete(Some("https://billing.example.com/complete".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_NONLINEAR, &trackers).unwrap();

        // Impression should be injected (applies to all ad types)
        assert!(
            result.contains("<Impression><![CDATA[https://billing.example.com/imp]]></Impression>")
        );

        // Video events should NOT be injected (no TrackingEvents in NonLinear-only ad)
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
        // Should ADD our trackers to existing ones, not replace
        let trackers = VastTrackersBuilder::default()
            .impression(Some("https://billing.example.com/imp".to_string()))
            .start(Some("https://billing.example.com/start".to_string()))
            .build()
            .unwrap();

        let result = inject_vast_trackers(VAST_WITH_EXISTING_TRACKERS, &trackers).unwrap();

        // Both our new trackers should be present
        assert!(result.contains("<![CDATA[https://billing.example.com/imp]]>"));
        assert!(result.contains("<![CDATA[https://billing.example.com/start]]>"));

        // Existing trackers should still be present
        assert!(result.contains("<![CDATA[https://existing.com/imp]]>"));
        assert!(result.contains("<![CDATA[https://existing.com/start]]>"));
        assert!(result.contains("<![CDATA[https://existing.com/complete]]>"));

        // Should have 2 impression trackers (ours + existing)
        assert_eq!(result.matches("<Impression>").count(), 2);
    }

    #[test]
    fn test_error_no_tracking_events() {
        // This test was updated - we now removed the old validation that errored here
        // Now we just verify the original test case still works
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

        // Should succeed even though there's no TrackingEvents - video events just won't be injected
        let result = inject_vast_trackers(vast_no_tracking, &trackers);
        assert!(result.is_ok());

        // Video events should not be in result
        let output = result.unwrap();
        assert!(!output.contains("event=\"start\""));
    }
}
