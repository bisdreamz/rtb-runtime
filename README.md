# rtb-runtime

Rust tooling for the IAB OpenRTB 2.x specification: generated message types, builders, JSON helpers, and a production-ready HTTP server that understands both JSON and protobuf bid requests.

## Highlights

- Ships with the official OpenRTB 2.x protobuf definitions (vendored as a submodule and regenerated during the build)
- JSON round-trips powered by `pbjson`/`serde` so payloads stay spec-compliant without manual glue
- Fully integrated HTTP server (`rtb::server`) with HTTP/1.1, h2c, and HTTP/2 ready to go for JSON and protobuf bid requests
- Builder pattern derived for every OpenRTB message, making handcrafted requests and responses pleasant
- Extension helpers for reading and writing custom `ext` payloads without losing type safety

## Usage

Parse OpenRTB JSON straight into generated types:

```rust
use rtb::BidRequest;

let request: BidRequest = serde_json::from_str(json_body)?;
println!("Request ID: {}", request.id);
```

Construct new messages with the derived builders:

```rust
use rtb::bid_request;
use rtb::BidRequestBuilder;

let request = BidRequestBuilder::default()
    .id("auction-123")
    .imp(vec![
        bid_request::ImpBuilder::default()
            .id("imp-001")
            .secure(true)
            .build()?,
    ])
    .build()?;
```

Extension fields expose both the protobuf-defined structure and a flexible custom map:

```rust
if let Some(imp) = request.imp.first() {
    if let Some(ext) = &imp.ext {
        let skadn = &ext.skadn;                 // Strongly typed protobuf field
        let channel = ext.custom().get_i64("channel");
        let enabled = ext.custom().get_bool_or("enabled", true);
    }
}
```

## HTTP Server

`rtb::server` exposes a high-level server that already wires up Actix Web, payload extractors, TLS, and HTTP/2 options. Provide a `ServerConfig`, register your handlers, and it will listen for both JSON and protobuf bid requests on the endpoints you define:

```rust
use actix_web::{web, web::Json, HttpResponse, web::ServiceConfig};
use rtb::BidRequest;
use rtb::server::extractors::Protobuf;
use rtb::server::{Server, ServerConfig, TlsConfig};

async fn bid_json(Json(request): Json<BidRequest>) -> HttpResponse {
    println!("JSON request {}", request.id);
    HttpResponse::Ok().finish()
}

async fn bid_proto(Protobuf(request): Protobuf<BidRequest>) -> HttpResponse {
    println!("Protobuf request {}", request.id);
    HttpResponse::Ok().finish()
}

fn routes(cfg: &mut ServiceConfig) {
    cfg.route("/bid/json", web::post().to(bid_json))
        .route("/bid/proto", web::post().to(bid_proto));
}

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let server_cfg = ServerConfig {
        http_port: Some(8080),
        ssl_port: None,
        tls: None, // or Some(TlsConfig::SelfSigned { hosts: vec!["localhost".into()] })
        tcp_backlog: None,
        max_conns: None,
        threads: None,
        tls_rate_per_worker: None,
    };

    let server = Server::listen(server_cfg, routes).await?;
    println!("Listening on http://0.0.0.0:8080");

    actix_rt::signal::ctrl_c().await?;
    server.stop().await;
    Ok(())
}
```

TLS (self-signed or provided certificates), h2/h2c support, request limits, and worker tuning are all part of the `ServerConfig`. See `examples/server_usage.rs` for a complete setup.

## Code Generation

A build script keeps the included OpenRTB definition up to date and applies the minimal patches required for Rust codegen. Generated files live in `OUT_DIR` and are rebuilt automatically when the proto sources change.

## Examples & Tooling

Sample payloads live under `examples/` and `test_data/`, and automated tests are in `tests/`.
