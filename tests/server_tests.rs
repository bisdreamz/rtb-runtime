#![cfg(feature = "actix-web")]

use actix_web::web::{PayloadConfig, ServiceConfig};
use actix_web::{HttpResponse, web};
use prost::Message;
use rtb::common::bidresponsestate::BidResponseState;
use rtb::server::protobuf::Protobuf;
use rtb::server::{Server, ServerConfig, TlsConfig};
use rtb::{BidRequest, BidResponse, bid_response};
use std::fs;
use std::time::Duration;

async fn protobuf_handler(req: Protobuf<BidRequest>) -> HttpResponse {
    HttpResponse::Ok().body(format!("id:{}", req.id))
}

fn configure_services(cfg: &mut ServiceConfig) {
    cfg.app_data(PayloadConfig::new(512 * 1024))
        .route("/proto", web::post().to(protobuf_handler));
}

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .http1_only()
        .build()
        .unwrap()
}

fn encode_bid_request() -> Vec<u8> {
    let bid_request: BidRequest =
        serde_json::from_str(r#"{"id":"test-123","imp":[{"id":"imp1"}]}"#).unwrap();
    let mut buf = Vec::new();
    bid_request.encode(&mut buf).unwrap();
    buf
}

#[actix_rt::test]
async fn test_http_server() {
    let cfg = ServerConfig {
        http_port: Some(8081),
        ssl_port: None,
        tls: None,
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: None,
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_services)
        .await
        .expect("Failed to start HTTP server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let response = test_client()
        .post("http://127.0.0.1:8081/proto")
        .header("Content-Type", "application/protobuf")
        .body(encode_bid_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "id:test-123");

    server.stop().await;
}

#[actix_rt::test]
async fn test_https_server() {
    let cfg = ServerConfig {
        http_port: None,
        ssl_port: Some(8443),
        tls: Some(TlsConfig::SelfSigned {
            hosts: vec!["localhost".to_string()],
        }),
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: Some(256),
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_services)
        .await
        .expect("Failed to start HTTPS server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let response = test_client()
        .post("https://127.0.0.1:8443/proto")
        .header("Content-Type", "application/protobuf")
        .body(encode_bid_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "id:test-123");

    server.stop().await;
}

#[actix_rt::test]
async fn test_both_http_and_https() {
    let cfg = ServerConfig {
        http_port: Some(8082),
        ssl_port: Some(8444),
        tls: Some(TlsConfig::SelfSigned {
            hosts: vec!["localhost".to_string()],
        }),
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: Some(256),
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_services)
        .await
        .expect("Failed to start server with both bindings");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let client = test_client();
    let body = encode_bid_request();

    let http_response = client
        .post("http://127.0.0.1:8082/proto")
        .header("Content-Type", "application/protobuf")
        .body(body.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(http_response.status(), 200);
    assert_eq!(http_response.text().await.unwrap(), "id:test-123");

    let https_response = client
        .post("https://127.0.0.1:8444/proto")
        .header("Content-Type", "application/protobuf")
        .body(body)
        .send()
        .await
        .unwrap();

    assert_eq!(https_response.status(), 200);
    assert_eq!(https_response.text().await.unwrap(), "id:test-123");

    server.stop().await;
}

#[actix_rt::test]
async fn test_gzip_compression() {
    let cfg = ServerConfig {
        http_port: Some(8083),
        ssl_port: None,
        tls: None,
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: None,
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_services)
        .await
        .expect("Failed to start server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let protobuf_bytes = encode_bid_request();
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut encoder, &protobuf_bytes).unwrap();
    let compressed = encoder.finish().unwrap();

    let response = test_client()
        .post("http://127.0.0.1:8083/proto")
        .header("Content-Type", "application/protobuf")
        .header("Content-Encoding", "gzip")
        .body(compressed)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "id:test-123");

    server.stop().await;
}

#[actix_rt::test]
async fn test_provided_certs_from_file() {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
    let cert_pem = cert.cert.pem();
    let key_pem = cert.signing_key.serialize_pem();

    let cert_path = std::env::temp_dir().join("test_cert.pem");
    let key_path = std::env::temp_dir().join("test_key.pem");

    fs::write(&cert_path, cert_pem).unwrap();
    fs::write(&key_path, key_pem).unwrap();

    let cfg = ServerConfig {
        http_port: None,
        ssl_port: Some(8445),
        tls: Some(TlsConfig::Provided {
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
        }),
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: Some(256),
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_services)
        .await
        .expect("Failed to start HTTPS server with provided certs");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let response = test_client()
        .post("https://127.0.0.1:8445/proto")
        .header("Content-Type", "application/protobuf")
        .body(encode_bid_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "id:test-123");

    server.stop().await;

    fs::remove_file(cert_path).ok();
    fs::remove_file(key_path).ok();
}

#[actix_rt::test]
async fn test_invalid_protobuf() {
    let cfg = ServerConfig {
        http_port: Some(8084),
        ssl_port: None,
        tls: None,
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: None,
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_services)
        .await
        .expect("Failed to start server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let response = test_client()
        .post("http://127.0.0.1:8084/proto")
        .header("Content-Type", "application/protobuf")
        .body(vec![0xFF, 0xFF, 0xFF, 0xFF])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 400);

    server.stop().await;
}

// New handlers that return protobuf responses
async fn protobuf_responder_handler(req: Protobuf<BidRequest>) -> Protobuf<BidResponse> {
    let response = BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        nbr: 0,
        seatbid: vec![bid_response::SeatBid {
            bid: vec![bid_response::Bid {
                id: "bid-1".to_string(),
                impid: req.imp.first().map(|i| i.id.clone()).unwrap_or_default(),
                price: 1.23,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    Protobuf(response)
}

fn configure_responder_services(cfg: &mut ServiceConfig) {
    cfg.app_data(PayloadConfig::new(512 * 1024)).route(
        "/proto-response",
        web::post().to(protobuf_responder_handler),
    );
}

#[actix_rt::test]
async fn test_protobuf_responder() {
    let cfg = ServerConfig {
        http_port: Some(8085),
        ssl_port: None,
        tls: None,
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: None,
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_responder_services)
        .await
        .expect("Failed to start server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let response = test_client()
        .post("http://127.0.0.1:8085/proto-response")
        .header("Content-Type", "application/protobuf")
        .body(encode_bid_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/x-protobuf"
    );

    let body_bytes = response.bytes().await.unwrap();
    let bid_response = BidResponse::decode(body_bytes.as_ref()).unwrap();

    assert_eq!(bid_response.id, "test-123");
    assert_eq!(bid_response.bidid, "bid-test-123");
    assert_eq!(bid_response.seatbid.len(), 1);
    assert_eq!(bid_response.seatbid[0].bid.len(), 1);
    assert_eq!(bid_response.seatbid[0].bid[0].id, "bid-1");
    assert_eq!(bid_response.seatbid[0].bid[0].price, 1.23);

    server.stop().await;
}

#[actix_rt::test]
async fn test_protobuf_responder_with_gzip() {
    let cfg = ServerConfig {
        http_port: Some(8086),
        ssl_port: None,
        tls: None,
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: None,
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_responder_services)
        .await
        .expect("Failed to start server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let response = test_client()
        .post("http://127.0.0.1:8086/proto-response")
        .header("Content-Type", "application/protobuf")
        .header("Accept-Encoding", "gzip")
        .body(encode_bid_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    // This protobuf response is below the gzip threshold, so it is sent plain;
    // test_gzip_on_the_wire covers compression.
    let body_bytes = response.bytes().await.unwrap();
    let bid_response = BidResponse::decode(body_bytes.as_ref()).unwrap();

    assert_eq!(bid_response.id, "test-123");
    assert_eq!(bid_response.bidid, "bid-test-123");

    server.stop().await;
}

// Handlers that return BidResponseState variants
async fn bid_state_handler(req: Protobuf<BidRequest>) -> Protobuf<BidResponseState> {
    let response = BidResponse {
        id: req.id.clone(),
        bidid: format!("bid-{}", req.id),
        nbr: 0,
        seatbid: vec![bid_response::SeatBid {
            bid: vec![bid_response::Bid {
                id: "bid-1".to_string(),
                impid: req.imp.first().map(|i| i.id.clone()).unwrap_or_default(),
                price: 2.50,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    Protobuf(BidResponseState::Bid(response))
}

async fn nobid_state_handler(_req: Protobuf<BidRequest>) -> Protobuf<BidResponseState> {
    Protobuf(BidResponseState::NoBidReason {
        reqid: "123".to_string(),
        nbr: 1, // Technical error
        desc: Some("Insufficient budget"),
    })
}

fn configure_state_services(cfg: &mut ServiceConfig) {
    cfg.app_data(PayloadConfig::new(512 * 1024))
        .route("/bid-state", web::post().to(bid_state_handler))
        .route("/nobid-state", web::post().to(nobid_state_handler));
}

/// Verify Protobuf<BidResponseState> responder works for Bid variant
#[actix_rt::test]
async fn test_bid_response_state_with_bid() {
    let cfg = ServerConfig {
        http_port: Some(8087),
        ssl_port: None,
        tls: None,
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: None,
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_state_services)
        .await
        .expect("Failed to start server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let response = test_client()
        .post("http://127.0.0.1:8087/bid-state")
        .header("Content-Type", "application/protobuf")
        .body(encode_bid_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/x-protobuf"
    );

    let body_bytes = response.bytes().await.unwrap();
    let bid_response = BidResponse::decode(body_bytes.as_ref()).unwrap();

    // Verify BidResponseState::Bid converted to BidResponse correctly
    assert_eq!(bid_response.id, "test-123");
    assert_eq!(bid_response.bidid, "bid-test-123");
    assert_eq!(bid_response.seatbid.len(), 1);
    assert_eq!(bid_response.seatbid[0].bid.len(), 1);
    assert_eq!(bid_response.seatbid[0].bid[0].price, 2.50);

    server.stop().await;
}

/// Verify Protobuf<BidResponseState> responder works for NoBid variant
#[actix_rt::test]
async fn test_bid_response_state_with_nobid() {
    let cfg = ServerConfig {
        http_port: Some(8088),
        ssl_port: None,
        tls: None,
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: None,
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_state_services)
        .await
        .expect("Failed to start server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    let response = test_client()
        .post("http://127.0.0.1:8088/nobid-state")
        .header("Content-Type", "application/protobuf")
        .body(encode_bid_request())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/x-protobuf"
    );

    let body_bytes = response.bytes().await.unwrap();
    let bid_response = BidResponse::decode(body_bytes.as_ref()).unwrap();

    // Verify BidResponseState::NoBid converted to BidResponse with nbr set
    assert_eq!(bid_response.nbr, 1); // Technical error code
    assert_eq!(bid_response.seatbid.len(), 0); // No bids present

    server.stop().await;
}

const GZIP_BODY_LEN: usize = 4096;

fn large_json_body() -> String {
    format!(r#"{{"p":"{}"}}"#, "a".repeat(GZIP_BODY_LEN))
}

async fn large_body_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(large_json_body())
}

fn configure_large_body_services(cfg: &mut ServiceConfig) {
    cfg.route("/large", web::get().to(large_body_handler))
        .route("/large", web::head().to(large_body_handler));
}

fn no_decompression_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .no_gzip()
        .build()
        .unwrap()
}

fn content_length(headers: &reqwest::header::HeaderMap) -> usize {
    headers
        .get(reqwest::header::CONTENT_LENGTH)
        .expect("buffered gzip responses are framed with content-length")
        .to_str()
        .unwrap()
        .parse()
        .unwrap()
}

/// Gzip as a client sees it over a real socket: no client-side decompression,
/// so the wire headers and bytes are what the server actually framed.
async fn assert_gzipped_on_the_wire(url: &str, expect_h2: bool) {
    use reqwest::header::{ACCEPT_ENCODING, CONTENT_ENCODING, VARY};

    let response = no_decompression_client()
        .get(url)
        .header(ACCEPT_ENCODING, "gzip")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.version() == reqwest::Version::HTTP_2, expect_h2);
    assert_eq!(response.headers()[CONTENT_ENCODING], "gzip");
    assert_eq!(response.headers()[VARY], ACCEPT_ENCODING.as_str());
    let declared_len = content_length(response.headers());
    let wire = response.bytes().await.unwrap();
    assert_eq!(declared_len, wire.len());

    let mut body = String::new();
    std::io::Read::read_to_string(&mut flate2::read::GzDecoder::new(wire.as_ref()), &mut body)
        .unwrap();
    assert_eq!(body, large_json_body());
    assert!(wire.len() < body.len());
}

/// HEAD carries the same representation headers as GET, with no body.
async fn assert_head_matches_get(url: &str) {
    use reqwest::header::{ACCEPT_ENCODING, CONTENT_ENCODING};

    let client = no_decompression_client();
    let get = client
        .get(url)
        .header(ACCEPT_ENCODING, "gzip")
        .send()
        .await
        .unwrap();
    let head = client
        .head(url)
        .header(ACCEPT_ENCODING, "gzip")
        .send()
        .await
        .unwrap();

    assert_eq!(head.headers()[CONTENT_ENCODING], "gzip");
    assert_eq!(
        content_length(head.headers()),
        content_length(get.headers())
    );
    assert!(head.bytes().await.unwrap().is_empty());
}

#[actix_rt::test]
async fn test_gzip_on_the_wire() {
    let cfg = ServerConfig {
        http_port: Some(8089),
        ssl_port: Some(8446),
        tls: Some(TlsConfig::SelfSigned {
            hosts: vec!["localhost".to_string()],
        }),
        tcp_backlog: None,
        max_conns: None,
        threads: Some(2),
        tls_rate_per_worker: None,
        keep_alive_secs: None,
        gzip_min_bytes: None,
    };

    let server = Server::listen(cfg, configure_large_body_services)
        .await
        .expect("Failed to start server");

    actix_rt::time::sleep(Duration::from_millis(100)).await;

    assert_gzipped_on_the_wire("http://127.0.0.1:8089/large", false).await;
    assert_gzipped_on_the_wire("https://127.0.0.1:8446/large", true).await;
    assert_head_matches_get("http://127.0.0.1:8089/large").await;

    server.stop().await;
}
