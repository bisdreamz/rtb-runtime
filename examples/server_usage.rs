use actix_web::web::{Json, PayloadConfig, ServiceConfig};
use actix_web::{HttpResponse, web};
use rtb::BidRequest;
use rtb::common::bidresponsestate::BidResponseState;
use rtb::server::json::JsonBidResponseState;
use rtb::server::protobuf::Protobuf;
use rtb::server::server::{Server, ServerConfig, TlsConfig};

fn log_br(req: BidRequest) {
    println!("{}", serde_json::to_string(&req).unwrap());
}

async fn proto_bid_handler(req: Protobuf<BidRequest>) -> Protobuf<BidResponseState> {
    // Automatically derefs to &BidRequest
    println!("Protobuf request");
    log_br(req.into_inner());

    Protobuf(BidResponseState::NoBidReason {
        nbr: 1,
        desc: Some("Sample Nbr Message".into())
    })
}

async fn json_bid_handler(req: Json<BidRequest>) -> JsonBidResponseState {
    println!("Json request");

    log_br(req.into_inner());

    JsonBidResponseState(
        BidResponseState::NoBidReason {
            nbr: 1,
            desc: Some("Sample Nbr Message".into())
        }
    )
}

#[actix_rt::main]
async fn main() {
    let cfg = ServerConfig {
        http_port: Some(80),
        ssl_port: Some(443),
        tls: Some(TlsConfig::SelfSigned {
            hosts: vec![String::from("localhost")],
        }),
        tcp_backlog: None,
        max_conns: None,
        threads: None,
        tls_rate_per_worker: Some(512),
    };

    let service = |cfg: &mut ServiceConfig| {
        cfg
            // Configure payload limits (512KB max)
            .app_data(PayloadConfig::new(512 * 1024))
            // Hello world endpoint
            .route("/hello", web::get().to(|| async { "Hello world!" }))
            .service(
                web::scope("/br")
                    .route("/proto", web::post().to(proto_bid_handler))
                    .route("/json", web::post().to(json_bid_handler)),
            );
    };

    let server = Server::listen(cfg, service).await.expect("Should listen");

    println!("Server listening on port 80 (HTTP) and 443 (HTTPS)");
    println!("  - GET  /hello    - Hello world");
    println!("  - POST /bid      - Protobuf bid request handler");

    actix_rt::time::sleep(std::time::Duration::from_secs(60)).await;

    println!("Shutting down...");
    server.stop().await;
    println!("Shutdown complete");
}
