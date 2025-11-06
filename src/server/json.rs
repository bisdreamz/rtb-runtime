use crate::BidResponse;
use crate::common::bidresponsestate::BidResponseState;
use actix_web::body::BoxBody;
#[cfg(feature = "simd-json")]
use actix_web::dev::Payload;
#[cfg(feature = "simd-json")]
use actix_web::web::BytesMut;
#[cfg(feature = "simd-json")]
use actix_web::{FromRequest, ResponseError};
use actix_web::{HttpRequest, HttpResponse, Responder};
#[cfg(feature = "simd-json")]
use futures_util::StreamExt;
#[cfg(feature = "simd-json")]
use futures_util::future::LocalBoxFuture;
use std::fmt;
use std::ops::Deref;

#[cfg(feature = "simd-json")]
const MAX_SIZE: usize = 262_144;

#[cfg(feature = "simd-json")]
use std::cell::RefCell;

#[cfg(feature = "simd-json")]
thread_local! {
    static DECOMPRESSOR: RefCell<libdeflater::Decompressor> =
        RefCell::new(libdeflater::Decompressor::new());
}

#[cfg(feature = "simd-json")]
/// Extract the ISIZE field from a gzip trailer (last 4 bytes, little-endian)
/// Returns the uncompressed size modulo 2^32
pub(crate) fn extract_gzip_isize(compressed: &[u8]) -> Result<usize, FastJsonError> {
    if compressed.len() < 18 {
        // Minimum gzip file is 18 bytes (10 header + 8 trailer)
        return Err(FastJsonError::Decompression(
            "Invalid gzip: too small".to_string(),
        ));
    }

    // ISIZE is the last 4 bytes, little-endian
    let isize_bytes = &compressed[compressed.len() - 4..];
    let isize = u32::from_le_bytes([
        isize_bytes[0],
        isize_bytes[1],
        isize_bytes[2],
        isize_bytes[3],
    ]) as usize;

    // Clamp to MAX_SIZE to prevent zip bombs
    if isize > MAX_SIZE {
        return Err(FastJsonError::Overflow);
    }

    // If ISIZE is 0, it means the size is a multiple of 2^32 or unknown
    // Use a reasonable default
    if isize == 0 { Ok(MAX_SIZE) } else { Ok(isize) }
}

#[cfg(feature = "simd-json")]
pub(crate) fn decompress_gzip(compressed: BytesMut) -> Result<BytesMut, FastJsonError> {
    let isize = extract_gzip_isize(&compressed)?;

    DECOMPRESSOR.with(|d| {
        let mut decompressor = d.borrow_mut();
        let mut decompressed = BytesMut::zeroed(isize);

        let actual_size = decompressor
            .gzip_decompress(&compressed, &mut decompressed)
            .map_err(|e| FastJsonError::Decompression(format!("libdeflater error: {:?}", e)))?;

        decompressed.truncate(actual_size);
        Ok(decompressed)
    })
}

pub struct FastJson<T>(pub T);

impl<T> FastJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for FastJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub enum FastJsonError {
    Overflow,
    #[cfg(feature = "simd-json")]
    Parse(simd_json::Error),
    #[cfg(feature = "simd-json")]
    Payload(actix_web::error::PayloadError),
    #[cfg(not(feature = "simd-json"))]
    Payload(actix_web::Error),
    #[cfg(feature = "simd-json")]
    Decompression(String),
}

impl fmt::Display for FastJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FastJsonError::Overflow => write!(f, "Payload too large"),
            #[cfg(feature = "simd-json")]
            FastJsonError::Parse(e) => write!(f, "JSON parse error: {}", e),
            FastJsonError::Payload(e) => write!(f, "Payload error: {}", e),
            #[cfg(feature = "simd-json")]
            FastJsonError::Decompression(e) => write!(f, "Decompression error: {}", e),
        }
    }
}

impl std::error::Error for FastJsonError {}

#[cfg(feature = "simd-json")]
impl ResponseError for FastJsonError {
    fn error_response(&self) -> HttpResponse {
        match self {
            FastJsonError::Overflow => HttpResponse::PayloadTooLarge().finish(),
            FastJsonError::Parse(_) => HttpResponse::BadRequest().finish(),
            FastJsonError::Payload(_) => HttpResponse::BadRequest().finish(),
            FastJsonError::Decompression(_) => HttpResponse::BadRequest().finish(),
        }
    }
}

#[cfg(feature = "simd-json")]
impl<T> FromRequest for FastJson<T>
where
    T: serde::de::DeserializeOwned + 'static,
{
    type Error = FastJsonError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let mut payload = payload.take();

        // Check if the request is gzip-compressed
        let is_gzip = req
            .headers()
            .get("content-encoding")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.eq_ignore_ascii_case("gzip"))
            .unwrap_or(false);

        Box::pin(async move {
            let mut body = BytesMut::new();

            while let Some(chunk) = payload.next().await {
                let chunk = chunk.map_err(FastJsonError::Payload)?;

                if (body.len() + chunk.len()) > MAX_SIZE {
                    return Err(FastJsonError::Overflow);
                }

                body.extend_from_slice(&chunk);
            }

            // Decompress if needed
            let mut final_body = if is_gzip {
                decompress_gzip(body)?
            } else {
                body
            };

            let value = simd_json::from_slice(final_body.as_mut()).map_err(FastJsonError::Parse)?;
            Ok(FastJson(value))
        })
    }
}

pub struct JsonBidResponseState(pub BidResponseState);

impl Responder for JsonBidResponseState {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self.0 {
            BidResponseState::Bid(bidresponse) => HttpResponse::Ok().json(bidresponse),
            BidResponseState::NoBidReason { reqid, nbr, desc } => HttpResponse::Ok()
                .reason(desc.unwrap_or("No Bid"))
                .json(BidResponse {
                    id: reqid,
                    nbr: nbr as i32,
                    ..Default::default()
                }),
            BidResponseState::NoBid { desc } => {
                let response = HttpResponse::NoContent()
                    .reason(desc.unwrap_or("No Bid"))
                    .finish();
                response
            }
        }
    }
}

#[cfg(all(test, feature = "simd-json"))]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_extract_gzip_isize() {
        let data = b"Hello, World!";
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        let compressed = encoder.finish().unwrap();

        let isize = extract_gzip_isize(&compressed).unwrap();
        assert_eq!(isize, data.len());
    }

    #[test]
    fn test_extract_gzip_isize_too_small() {
        let compressed = vec![0u8; 10];
        let result = extract_gzip_isize(&compressed);
        assert!(matches!(result, Err(FastJsonError::Decompression(_))));
    }

    #[test]
    fn test_extract_gzip_isize_overflow() {
        let mut fake_gzip = vec![0u8; 18];
        fake_gzip[0] = 0x1f;
        fake_gzip[1] = 0x8b;
        let large_size = (MAX_SIZE + 1) as u32;
        fake_gzip[14..18].copy_from_slice(&large_size.to_le_bytes());

        let result = extract_gzip_isize(&fake_gzip);
        assert!(matches!(result, Err(FastJsonError::Overflow)));
    }

    #[test]
    fn test_decompress_gzip() {
        let data = b"{\"test\": \"data\"}";
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        let compressed = encoder.finish().unwrap();

        let compressed_buf = BytesMut::from(&compressed[..]);
        let decompressed = decompress_gzip(compressed_buf).unwrap();

        assert_eq!(&decompressed[..], data);
    }

    #[test]
    fn test_decompress_gzip_with_json() {
        let json_data = br#"{"id":"123","imp":[{"id":"1","banner":{"w":300,"h":250}}]}"#;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(json_data).unwrap();
        let compressed = encoder.finish().unwrap();

        let compressed_buf = BytesMut::from(&compressed[..]);
        let decompressed = decompress_gzip(compressed_buf).unwrap();

        assert_eq!(&decompressed[..], json_data);
        assert!(serde_json::from_slice::<serde_json::Value>(&decompressed).is_ok());
    }
}
