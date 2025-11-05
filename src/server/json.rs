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
use futures_util::future::LocalBoxFuture;
#[cfg(feature = "simd-json")]
use futures_util::StreamExt;
use std::fmt;
use std::ops::Deref;

#[cfg(feature = "simd-json")]
const MAX_SIZE: usize = 262_144;

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
}

impl fmt::Display for FastJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FastJsonError::Overflow => write!(f, "Payload too large"),
            #[cfg(feature = "simd-json")]
            FastJsonError::Parse(e) => write!(f, "JSON parse error: {}", e),
            FastJsonError::Payload(e) => write!(f, "Payload error: {}", e),
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

    fn from_request(_req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let mut payload = payload.take();

        Box::pin(async move {
            let mut body = BytesMut::new();

            while let Some(chunk) = payload.next().await {
                let chunk = chunk.map_err(FastJsonError::Payload)?;

                if (body.len() + chunk.len()) > MAX_SIZE {
                    return Err(FastJsonError::Overflow);
                }

                body.extend_from_slice(&chunk);
            }

            let value = simd_json::from_slice(body.as_mut()).map_err(FastJsonError::Parse)?;
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
