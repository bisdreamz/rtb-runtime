//! Response gzip with a size threshold.
//!
//! Replaces actix's `Compress` middleware, which builds a fresh zlib encoder
//! (allocating and zeroing its full deflate state) for every response it
//! touches. RTB traffic is dominated by tiny bodies (no-bid `{"id","nbr"}`
//! responses) where that setup is most of the response cost and the gzipped
//! output is larger than the input.
//!
//! This middleware gzips only complete, in-memory bodies of at least
//! `min_bytes`, using one reused libdeflate compressor per worker thread.
//! Streamed bodies, bodies below the threshold, and responses that already
//! carry a `Content-Encoding` pass through untouched. The one exception is a
//! client that refuses the identity encoding: it gets gzip at any size.
//!
//! Compression runs inline on the worker thread. At level 1 libdeflate
//! compresses a few hundred KB in well under a millisecond, which is cheaper
//! than handing the body to the blocking pool the way actix did for every
//! chunk of 1 KiB or more.
//!
//! Only gzip is offered: no deflate, brotli or zstd negotiation, and no 406
//! when a client accepts nothing we produce (it gets the identity body). As
//! with actix, raster images and video are never compressed.

use actix_web::body::{BodySize, BoxBody, EitherBody, MessageBody};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::StatusCode;
use actix_web::http::header::{self, HeaderMap, HeaderValue};
use actix_web::mime::{self, Mime};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use libdeflater::{CompressionLvl, Compressor};
use std::cell::RefCell;

/// Default for [`crate::server::ServerConfig::gzip_min_bytes`].
pub const DEFAULT_GZIP_MIN_BYTES: usize = 256;

/// libdeflate level 1 (`deflate_compress_fastest`), the fastest level that
/// actually compresses. Level 0 emits stored, uncompressed blocks, and
/// libdeflater's `CompressionLvl::fastest()` maps to it, so never use that.
const GZIP_LEVEL: CompressionLvl = match CompressionLvl::new(1) {
    Ok(level) => level,
    Err(_) => panic!("libdeflate level 1 must be valid"),
};

const GZIP: &str = "gzip";
/// Legacy alias for gzip (RFC 9110 §8.4.1.3).
const X_GZIP: &str = "x-gzip";
const IDENTITY: &str = "identity";
const WILDCARD: &str = "*";
const QUALITY: &str = "q";

thread_local! {
    static COMPRESSOR: RefCell<Compressor> = RefCell::new(Compressor::new(GZIP_LEVEL));
}

/// Middleware factory. See the module docs.
#[derive(Debug, Clone, Copy)]
pub struct Gzip {
    min_bytes: usize,
}

impl Gzip {
    pub fn new(min_bytes: usize) -> Self {
        Self { min_bytes }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Gzip
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Transform = GzipMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(GzipMiddleware {
            service,
            min_bytes: self.min_bytes,
        }))
    }
}

pub struct GzipMiddleware<S> {
    service: S,
    min_bytes: usize,
}

impl<S, B> Service<ServiceRequest> for GzipMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let min_bytes = self.min_bytes;
        let fut = self.service.call(req);

        Box::pin(async move { Ok(encode(fut.await?, min_bytes)) })
    }
}

fn encode<B: MessageBody>(
    mut res: ServiceResponse<B>,
    min_bytes: usize,
) -> ServiceResponse<EitherBody<B>> {
    let skip_status = matches!(
        res.status(),
        StatusCode::NO_CONTENT | StatusCode::PARTIAL_CONTENT | StatusCode::SWITCHING_PROTOCOLS
    );
    let BodySize::Sized(len) = res.response().body().size() else {
        return res.map_into_left_body();
    };
    if skip_status
        || len == 0
        || res.headers().contains_key(header::CONTENT_ENCODING)
        || !compressible(res.headers())
    {
        return res.map_into_left_body();
    }

    // Parsed only for responses that could be compressed.
    let accepted = Accepted::from_headers(res.request().headers());

    // Vary follows the threshold, not this client, so every client sees the
    // same header for the same response. Plain sub-threshold responses omit it
    // on purpose: they are mostly uncacheable POST no-bids, and only a client
    // refusing identity would get them gzipped (and then they do carry Vary).
    let varies = len >= min_bytes as u64;
    if varies {
        add_vary(res.headers_mut());
    }
    let must_gzip = accepted.gzip && !accepted.identity;
    if !accepted.gzip || !(varies || must_gzip) {
        return res.map_into_left_body();
    }

    let (req, response) = res.into_parts();
    let (mut head, body) = response.into_parts();

    // Only a single in-memory chunk is compressed; anything else streams as-is.
    let bytes = match body.try_into_bytes() {
        Ok(bytes) => bytes,
        Err(body) => return ServiceResponse::new(req, head.set_body(EitherBody::left(body))),
    };

    let body = match gzip(&bytes, must_gzip) {
        Some(compressed) => {
            let headers = head.headers_mut();
            headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static(GZIP));
            if !varies {
                add_vary(headers);
            }
            headers.remove(header::CONTENT_LENGTH);
            BoxBody::new(compressed)
        }
        None => BoxBody::new(bytes),
    };

    ServiceResponse::new(req, head.set_body(EitherBody::right(body)))
}

/// Actix `Compress`'s predicate: raster images and video are already
/// compressed, so a deflate pass is wasted. SVG is text and still compresses.
fn compressible(headers: &HeaderMap) -> bool {
    let Some(mime) = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<Mime>().ok())
    else {
        return true;
    };
    match mime.type_() {
        mime::IMAGE => mime.subtype() == mime::SVG,
        mime::VIDEO => false,
        _ => true,
    }
}

/// Adds `Vary: accept-encoding` unless an existing `Vary` already covers it.
fn add_vary(headers: &mut HeaderMap) {
    let covered = headers
        .get_all(header::VARY)
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .any(|name| {
            name == WILDCARD || name.eq_ignore_ascii_case(header::ACCEPT_ENCODING.as_str())
        });
    if !covered {
        headers.append(header::VARY, HeaderValue::from(header::ACCEPT_ENCODING));
    }
}

/// Gzips `data` with the thread's reused compressor. `None` if compression
/// fails, or if it does not make the body smaller and `force` is false, in
/// which case the original is sent.
fn gzip(data: &[u8], force: bool) -> Option<Vec<u8>> {
    COMPRESSOR.with(|compressor| {
        let mut compressor = compressor.borrow_mut();
        let mut out = vec![0u8; compressor.gzip_compress_bound(data.len())];
        let len = compressor.gzip_compress(data, &mut out).ok()?;
        (force || len < data.len()).then(|| {
            out.truncate(len);
            out
        })
    })
}

/// Response codings the client accepts (RFC 9110 §12.5.3). An explicit entry
/// decides; otherwise a `*` entry does. `q=0` refuses. Identity is acceptable
/// unless refused by `identity;q=0`, or by `*;q=0` with no explicit identity.
/// A malformed or out-of-range q-value is ignored, so the coding counts as
/// accepted (RFC 9110 §12.4.2 allows only 0 to 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Accepted {
    gzip: bool,
    identity: bool,
}

impl Accepted {
    fn from_headers(headers: &HeaderMap) -> Self {
        let (mut gzip, mut identity, mut wildcard) = (None, None, None);
        for value in headers.get_all(header::ACCEPT_ENCODING) {
            let Ok(value) = value.to_str() else { continue };
            for entry in value.split(',') {
                let mut parts = entry.split(';');
                let coding = parts.next().unwrap_or_default().trim();
                if coding.is_empty() {
                    continue;
                }
                let accepted = !parts
                    .filter_map(|param| {
                        let (key, q) = param.trim().split_once('=')?;
                        key.trim()
                            .eq_ignore_ascii_case(QUALITY)
                            .then(|| q.trim().parse::<f32>().ok())?
                    })
                    .next()
                    .is_some_and(|q| q == 0.0);
                if coding.eq_ignore_ascii_case(GZIP) || coding.eq_ignore_ascii_case(X_GZIP) {
                    gzip = Some(accepted);
                } else if coding.eq_ignore_ascii_case(IDENTITY) {
                    identity = Some(accepted);
                } else if coding == WILDCARD {
                    wildcard = Some(accepted);
                }
            }
        }
        Self {
            gzip: gzip.or(wildcard).unwrap_or(false),
            identity: identity.or(wildcard).unwrap_or(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test as actix_test;
    use actix_web::{App, HttpResponse, web};
    use std::io::Read;

    const MIN: usize = DEFAULT_GZIP_MIN_BYTES;

    fn accepted(accept_encoding: &str) -> Accepted {
        let mut map = HeaderMap::new();
        map.insert(
            header::ACCEPT_ENCODING,
            HeaderValue::from_str(accept_encoding).unwrap(),
        );
        Accepted::from_headers(&map)
    }

    fn acc(gzip: bool, identity: bool) -> Accepted {
        Accepted { gzip, identity }
    }

    #[test]
    fn accept_encoding_negotiation() {
        let cases = [
            ("gzip", acc(true, true)),
            ("deflate, GZIP;q=0.5, br", acc(true, true)),
            ("x-gzip", acc(true, true)),
            ("*", acc(true, true)),
            ("gzip;q=0", acc(false, true)),
            ("gzip; q=0.0, *", acc(false, true)),
            ("*;q=0, gzip", acc(true, false)),
            ("gzip, identity;q=0", acc(true, false)),
            ("*;q=0, identity", acc(false, true)),
            ("br, zstd", acc(false, true)),
            ("identity", acc(false, true)),
            ("gzip;q=abc", acc(true, true)),
            ("gzip;q=NaN", acc(true, true)),
            ("gzip;q=-1", acc(true, true)),
            ("gzip;q=2", acc(true, true)),
            ("gzip, identity;q=-1", acc(true, true)),
            ("gzip;q=0.000", acc(false, true)),
            (", ,gzip", acc(true, true)),
        ];
        for (header, expected) in cases {
            assert_eq!(accepted(header), expected, "{header}");
        }
        assert_eq!(Accepted::from_headers(&HeaderMap::new()), acc(false, true));
    }

    /// gzip member header length when no optional fields are set (RFC 1952).
    const GZIP_HEADER_LEN: usize = 10;
    /// DEFLATE block type for a stored (uncompressed) block (RFC 1951 §3.2.3).
    const BTYPE_STORED: u8 = 0b00;

    #[test]
    fn gzip_level_emits_compressed_blocks_not_stored() {
        let bids: Vec<String> = (0..8)
            .map(|i| {
                format!(
                    r#"{{"id":"bid-{i}","impid":"imp-{i}","price":{}.{i}5,"adomain":["brand{i}.com"],"crid":"cr-{i}","w":300,"h":250,"adm":"<div class=\"ad\"><a href=\"https://click.example/{i}\">ad {i}</a></div>"}}"#,
                    i + 1
                )
            })
            .collect();
        let body = format!(
            r#"{{"id":"req-1","cur":"USD","seatbid":[{{"seat":"dsp","bid":[{}]}}]}}"#,
            bids.join(",")
        );

        let wire = gzip(body.as_bytes(), true).expect("gzip");

        assert_eq!(wire[3], 0, "no optional gzip header fields");
        let btype = (wire[GZIP_HEADER_LEN] >> 1) & 0b11;
        assert_ne!(
            btype, BTYPE_STORED,
            "level is a passthrough, not compression"
        );
        assert!(
            wire.len() * 2 < body.len(),
            "{} -> {}",
            body.len(),
            wire.len()
        );
        assert_eq!(gunzip(&wire), body.as_bytes());
    }

    fn json_of_len(len: usize) -> String {
        let pad = len - r#"{"p":""}"#.len();
        format!(r#"{{"p":"{}"}}"#, "a".repeat(pad))
    }

    async fn call_with(
        min_bytes: usize,
        body: impl Into<Vec<u8>>,
        status: StatusCode,
        accept: Option<&str>,
    ) -> (HeaderMap, Vec<u8>) {
        let body: Vec<u8> = body.into();
        let app = actix_test::init_service(App::new().wrap(Gzip::new(min_bytes)).route(
            "/",
            web::get().to(move || {
                let body = body.clone();
                async move {
                    HttpResponse::build(status)
                        .content_type("application/json")
                        .body(body)
                }
            }),
        ))
        .await;
        let mut req = actix_test::TestRequest::get().uri("/");
        if let Some(accept) = accept {
            req = req.insert_header((header::ACCEPT_ENCODING, accept));
        }
        let res = actix_test::call_service(&app, req.to_request()).await;
        let headers = res.headers().clone();
        (headers, actix_test::read_body(res).await.to_vec())
    }

    async fn call_typed(
        content_type: &'static str,
        body: impl Into<Vec<u8>>,
    ) -> (HeaderMap, Vec<u8>) {
        let body: Vec<u8> = body.into();
        let app = actix_test::init_service(App::new().wrap(Gzip::new(MIN)).route(
            "/",
            web::get().to(move || {
                let body = body.clone();
                async move { HttpResponse::Ok().content_type(content_type).body(body) }
            }),
        ))
        .await;
        let req = actix_test::TestRequest::get()
            .uri("/")
            .insert_header((header::ACCEPT_ENCODING, GZIP))
            .to_request();
        let res = actix_test::call_service(&app, req).await;
        let headers = res.headers().clone();
        (headers, actix_test::read_body(res).await.to_vec())
    }

    async fn call(
        body: impl Into<Vec<u8>>,
        status: StatusCode,
        accept: Option<&str>,
    ) -> (HeaderMap, Vec<u8>) {
        call_with(MIN, body, status, accept).await
    }

    fn gunzip(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        flate2::read::GzDecoder::new(data)
            .read_to_end(&mut out)
            .unwrap();
        out
    }

    fn is_gzip(headers: &HeaderMap) -> bool {
        headers.get(header::CONTENT_ENCODING).map(|v| v.as_bytes()) == Some(GZIP.as_bytes())
    }

    fn varies(headers: &HeaderMap) -> bool {
        headers
            .get_all(header::VARY)
            .any(|v| v.as_bytes() == header::ACCEPT_ENCODING.as_str().as_bytes())
    }

    #[actix_rt::test]
    async fn body_at_threshold_is_gzipped_and_round_trips() {
        let body = json_of_len(MIN);
        let (headers, wire) = call(body.clone(), StatusCode::OK, Some("gzip")).await;
        assert!(is_gzip(&headers));
        assert!(varies(&headers));
        assert!(wire.len() < body.len());
        assert_eq!(gunzip(&wire), body.as_bytes());
    }

    #[actix_rt::test]
    async fn body_below_threshold_is_sent_plain_without_vary() {
        let body = json_of_len(MIN - 1);
        let (headers, wire) = call(body.clone(), StatusCode::OK, Some("gzip")).await;
        assert!(!is_gzip(&headers));
        assert!(!varies(&headers), "plain sub-threshold responses omit Vary");
        assert_eq!(wire, body.as_bytes());
    }

    #[actix_rt::test]
    async fn configured_threshold_is_honoured() {
        let body = json_of_len(200);
        let (headers, _) = call_with(100, body.clone(), StatusCode::OK, Some("gzip")).await;
        assert!(is_gzip(&headers));
        let (headers, _) = call_with(300, body, StatusCode::OK, Some("gzip")).await;
        assert!(!is_gzip(&headers));
    }

    #[actix_rt::test]
    async fn client_without_gzip_gets_plain_body_with_vary() {
        let body = json_of_len(MIN * 4);
        for accept in [None, Some("br"), Some("gzip;q=0")] {
            let (headers, wire) = call(body.clone(), StatusCode::OK, accept).await;
            assert!(!is_gzip(&headers), "{accept:?}");
            assert!(
                varies(&headers),
                "{accept:?}: a cache must not serve this to gzip clients blindly"
            );
            assert_eq!(wire, body.as_bytes(), "{accept:?}");
        }
    }

    #[actix_rt::test]
    async fn identity_refusal_forces_gzip_below_threshold() {
        let body = json_of_len(64);
        for accept in ["gzip, identity;q=0", "*;q=0, gzip"] {
            let (headers, wire) = call(body.clone(), StatusCode::OK, Some(accept)).await;
            assert!(is_gzip(&headers), "{accept}");
            assert!(varies(&headers), "{accept}");
            assert_eq!(gunzip(&wire), body.as_bytes(), "{accept}");
        }
    }

    fn incompressible(len: usize) -> Vec<u8> {
        let mut state = 0x2545_f491_4f6c_dd1du64;
        (0..len)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                (state >> 32) as u8
            })
            .collect()
    }

    #[actix_rt::test]
    async fn incompressible_body_is_sent_plain_unless_identity_refused() {
        let body = incompressible(MIN * 2);
        let (headers, wire) = call(body.clone(), StatusCode::OK, Some("gzip")).await;
        assert!(!is_gzip(&headers));
        assert!(varies(&headers));
        assert_eq!(wire, body);

        let (headers, wire) = call(body.clone(), StatusCode::OK, Some("gzip, identity;q=0")).await;
        assert!(
            is_gzip(&headers),
            "identity refused: gzip even when it grows"
        );
        assert_eq!(gunzip(&wire), body);
    }

    #[actix_rt::test]
    async fn skipped_statuses_are_untouched() {
        let body = json_of_len(MIN * 4);
        let (headers, wire) = call(body.clone(), StatusCode::PARTIAL_CONTENT, Some("gzip")).await;
        assert!(!is_gzip(&headers));
        assert!(!varies(&headers));
        assert_eq!(wire, body.as_bytes());

        let (headers, wire) = call(Vec::new(), StatusCode::NO_CONTENT, Some("gzip")).await;
        assert!(!is_gzip(&headers));
        assert!(wire.is_empty());
    }

    #[actix_rt::test]
    async fn raster_images_and_video_are_not_compressed() {
        let body = json_of_len(MIN * 4);
        for content_type in ["image/png", "video/mp4"] {
            let (headers, wire) = call_typed(content_type, body.clone()).await;
            assert!(!is_gzip(&headers), "{content_type}");
            assert!(!varies(&headers), "{content_type}");
            assert_eq!(wire, body.as_bytes(), "{content_type}");
        }
        let (headers, wire) = call_typed("image/svg+xml", body.clone()).await;
        assert!(is_gzip(&headers), "SVG is text and compresses");
        assert_eq!(gunzip(&wire), body.as_bytes());
    }

    async fn vary_values_with_existing(existing: &'static str) -> Vec<String> {
        let body = json_of_len(MIN * 4);
        let app = actix_test::init_service(App::new().wrap(Gzip::new(MIN)).route(
            "/",
            web::get().to(move || {
                let body = body.clone();
                async move {
                    HttpResponse::Ok()
                        .insert_header((header::VARY, existing))
                        .body(body)
                }
            }),
        ))
        .await;
        let req = actix_test::TestRequest::get()
            .uri("/")
            .insert_header((header::ACCEPT_ENCODING, GZIP))
            .to_request();
        let res = actix_test::call_service(&app, req).await;
        assert!(is_gzip(res.headers()));
        res.headers()
            .get_all(header::VARY)
            .map(|v| v.to_str().unwrap().to_string())
            .collect()
    }

    #[actix_rt::test]
    async fn existing_vary_is_preserved_and_never_duplicated() {
        assert_eq!(
            vary_values_with_existing("origin").await,
            ["origin", header::ACCEPT_ENCODING.as_str()]
        );
        assert_eq!(
            vary_values_with_existing("Origin, Accept-Encoding").await,
            ["Origin, Accept-Encoding"]
        );
        assert_eq!(vary_values_with_existing("*").await, ["*"]);
    }

    #[actix_rt::test]
    async fn already_encoded_response_is_untouched() {
        let body = json_of_len(MIN * 4);
        let app = actix_test::init_service(App::new().wrap(Gzip::new(MIN)).route(
            "/",
            web::get().to(move || {
                let body = body.clone();
                async move {
                    HttpResponse::Ok()
                        .insert_header((header::CONTENT_ENCODING, "br"))
                        .body(body)
                }
            }),
        ))
        .await;
        let req = actix_test::TestRequest::get()
            .uri("/")
            .insert_header((header::ACCEPT_ENCODING, GZIP))
            .to_request();
        let res = actix_test::call_service(&app, req).await;
        assert_eq!(res.headers().get(header::CONTENT_ENCODING).unwrap(), "br");
        assert!(!varies(res.headers()));
    }

    #[actix_rt::test]
    async fn streamed_body_passes_through() {
        let app = actix_test::init_service(App::new().wrap(Gzip::new(MIN)).route(
            "/",
            web::get().to(|| async {
                let chunk = web::Bytes::from(json_of_len(MIN * 4));
                HttpResponse::Ok().streaming(futures_util::stream::once(async move {
                    Ok::<_, actix_web::Error>(chunk)
                }))
            }),
        ))
        .await;
        let req = actix_test::TestRequest::get()
            .uri("/")
            .insert_header((header::ACCEPT_ENCODING, GZIP))
            .to_request();
        let res = actix_test::call_service(&app, req).await;
        assert!(!is_gzip(res.headers()));
        assert_eq!(actix_test::read_body(res).await.len(), MIN * 4);
    }
}
