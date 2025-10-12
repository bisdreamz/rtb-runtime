use actix_web::{FromRequest, HttpRequest, HttpResponse, ResponseError};
use actix_web::dev::Payload;
use actix_web::web::Bytes;
use futures_util::future::LocalBoxFuture;
use prost::Message;
use std::fmt;
use std::ops::Deref;

/// Maximum payload size after decompression (256KB)
///
/// This limit applies to the decoded protobuf payload. Compressed payloads
/// may be smaller on the wire but will be checked against this limit after
/// decompression to prevent zip bomb attacks.
const MAX_SIZE: usize = 262_144;

/// Extractor for protobuf-encoded request bodies.
///
/// This extractor automatically handles:
/// - **Compression**: Transparent decompression of gzip, br, and deflate payloads
/// - **Size limits**: Respects actix-web's `PayloadConfig` for incoming data
/// - **Safety**: Additional post-decompression size check to prevent zip bombs
///
/// # Configuration
///
/// Set payload size limits using `PayloadConfig` in your app data:
///
/// ```ignore
/// use actix_web::{web, App};
/// use actix_web::web::PayloadConfig;
///
/// App::new()
///     .app_data(PayloadConfig::new(512 * 1024)) // 512KB limit
///     .route("/bid", web::post().to(handler))
/// ```
///
/// # Example
///
/// ```ignore
/// use actix_web::{web, HttpResponse};
/// use openrtb_rs::BidRequest;
/// use openrtb_rs::server::extractors::Protobuf;
///
/// async fn bid_handler(req: Protobuf<BidRequest>) -> HttpResponse {
///     // Automatically derefs to &BidRequest
///     println!("Bid ID: {}", req.id);
///
///     // Or unwrap to owned value
///     let bid_request = req.into_inner();
///
///     HttpResponse::Ok().finish()
/// }
/// ```
///
/// # Generic Usage
///
/// Works with any protobuf message type:
///
/// ```ignore
/// async fn response_handler(res: Protobuf<BidResponse>) -> HttpResponse {
///     // Process BidResponse...
///     HttpResponse::Ok().finish()
/// }
/// ```
pub struct Protobuf<T>(T);

impl<T> Protobuf<T> {
    /// Unwrap into the inner protobuf message.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Protobuf<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Errors that can occur during protobuf extraction.
#[derive(Debug)]
pub enum ProtobufError {
    /// Payload exceeds maximum size (pre- or post-decompression).
    Overflow,
    /// Failed to decode protobuf message.
    Decode(prost::DecodeError),
    /// Error reading or processing request body.
    Payload(actix_web::Error),
}

impl fmt::Display for ProtobufError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtobufError::Overflow => write!(f, "Payload too large"),
            ProtobufError::Decode(e) => write!(f, "Protobuf decode error: {}", e),
            ProtobufError::Payload(e) => write!(f, "Payload error: {}", e),
        }
    }
}

impl std::error::Error for ProtobufError {}

impl ResponseError for ProtobufError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ProtobufError::Overflow => HttpResponse::PayloadTooLarge().finish(),
            ProtobufError::Decode(_) => HttpResponse::BadRequest().finish(),
            ProtobufError::Payload(_) => HttpResponse::BadRequest().finish(),
        }
    }
}

impl<T> FromRequest for Protobuf<T>
where
    T: Message + Default + 'static,
{
    type Error = ProtobufError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        // Delegate to Bytes extractor, which handles:
        // - Automatic decompression (gzip, br, deflate)
        // - Size limits from PayloadConfig
        // - Optimized buffering
        let fut = Bytes::from_request(req, payload);

        Box::pin(async move {
            let bytes = fut.await.map_err(ProtobufError::Payload)?;

            // Enforce post-decompression size limit to prevent zip bomb attacks
            // (A small gzipped payload could decompress to gigabytes)
            if bytes.len() > MAX_SIZE {
                return Err(ProtobufError::Overflow);
            }

            // Decode protobuf message from bytes
            let msg = T::decode(bytes.as_ref()).map_err(ProtobufError::Decode)?;

            Ok(Protobuf(msg))
        })
    }
}
