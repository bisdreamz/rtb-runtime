use actix_web::web;
use actix_web::web::ServiceConfig;
use serde::de::Unexpected::Option;
use openrtb_rs::server::server::{Binding, Server, ServerConfig, TlsConfig};

#[actix_rt::main]
async fn main() {
    let cfg = ServerConfig {
        tcp_backlog: None,
        max_conns: None,
        threads: None,
        tls_rate_per_worker: Some(512)
    };

    let binding = Binding::Both {
        port: 80,
        tls: TlsConfig::SelfSigned {
            hosts: vec![String::from("localhost")]
        }
    };

    let service = |cfg: &mut ServiceConfig /* Type */| {
        cfg.route("/hello", web::get().to(|| async { "Hello world!" }));
    };

    let server = Server::listen(cfg, binding, service)
        .await.expect("Should listen");

    println!("Listening!");

    actix_rt::time::sleep(std::time::Duration::from_secs(60)).await;

    println!("Shutting down...!");
    server.stop().await;
    println!("Shut done");
}