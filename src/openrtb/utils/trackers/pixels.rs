/// Error type for pixel URL validation
#[derive(Debug, Clone, PartialEq)]
pub enum PixelError {
    EmptyUrl,
    InvalidScheme,
}

impl std::fmt::Display for PixelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PixelError::EmptyUrl => write!(f, "URL cannot be empty"),
            PixelError::InvalidScheme => write!(f, "URL must start with http:// or https://"),
        }
    }
}

impl std::error::Error for PixelError {}

/// Validates a URL for use in a tracking pixel
fn validate_url(url: &str) -> Result<(), PixelError> {
    let trimmed = url.trim();

    if trimmed.is_empty() {
        return Err(PixelError::EmptyUrl);
    }

    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(PixelError::InvalidScheme);
    }

    Ok(())
}

/// Generates HTML for a 1x1 transparent tracking pixel
///
/// # Arguments
/// * `url` - The tracking URL to use as the image source
///
/// # Returns
/// * `Ok(String)` - The HTML string for the tracking pixel
/// * `Err(PixelError)` - If the URL is invalid
///
/// # Example
/// ```
/// use rtb::openrtb::utils::trackers::html_pixel;
///
/// let html = html_pixel("https://example.com/track?id=123").unwrap();
/// assert!(html.contains("https://example.com/track?id=123"));
/// ```
pub fn html_pixel(url: impl AsRef<str>) -> Result<String, PixelError> {
    let url = url.as_ref();
    validate_url(url)?;

    Ok(format!(
        r#"<img src="{}" width="1" height="1" style="border:0;display:none" alt="" />"#,
        html_escape(url)
    ))
}

/// Escapes HTML special characters in a string
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_pixel_with_https() {
        let result = html_pixel("https://example.com/track");
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains(r#"src="https://example.com/track""#));
        assert!(html.contains(r#"width="1""#));
        assert!(html.contains(r#"height="1""#));
        assert!(html.contains(r#"border:0"#));
    }

    #[test]
    fn test_html_pixel_with_http() {
        let result = html_pixel("http://example.com/track");
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains(r#"src="http://example.com/track""#));
    }

    #[test]
    fn test_html_pixel_with_query_params() {
        let result = html_pixel("https://track.example.com/pixel?id=123&type=impression");
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("id=123"));
        assert!(html.contains("type=impression"));
    }

    #[test]
    fn test_html_pixel_empty_url() {
        let result = html_pixel("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PixelError::EmptyUrl);
    }

    #[test]
    fn test_html_pixel_whitespace_only() {
        let result = html_pixel("   ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PixelError::EmptyUrl);
    }

    #[test]
    fn test_html_pixel_invalid_scheme() {
        let result = html_pixel("ftp://example.com/track");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PixelError::InvalidScheme);
    }

    #[test]
    fn test_html_pixel_no_scheme() {
        let result = html_pixel("example.com/track");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), PixelError::InvalidScheme);
    }

    #[test]
    fn test_html_escape() {
        let escaped = html_escape("https://example.com?a=1&b=2");
        assert_eq!(escaped, "https://example.com?a=1&amp;b=2");
    }

    #[test]
    fn test_html_escape_all_chars() {
        let escaped = html_escape(r#"<script>alert('xss')"</script>"#);
        assert_eq!(
            escaped,
            r#"&lt;script&gt;alert(&#x27;xss&#x27;)&quot;&lt;/script&gt;"#
        );
    }

    #[test]
    fn test_html_pixel_escapes_special_chars() {
        let result = html_pixel(r#"https://example.com/track?param="value""#);
        assert!(result.is_ok());
        let html = result.unwrap();
        // The URL should have quotes escaped as &quot;
        assert!(html.contains("param=&quot;value&quot;"));
        // Should still have proper HTML structure with quotes in attributes
        assert!(html.starts_with(r#"<img src=""#));
    }
}
