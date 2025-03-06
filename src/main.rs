pub mod common;
pub mod routes;
pub mod stream;
use std::env;

use actix_web::{HttpResponse, Responder, middleware::Logger};
use env_logger::Env;
use tokio::time::Duration;

use actix_web::{App, HttpServer, web};

fn configure(cfg: &mut web::ServiceConfig) {
    let scope = web::scope("/v1")
        .route("/completions", web::post().to(routes::completions))
        .route(
            "/streaming",
            web::post().to(routes::chat_completions),
        );

    cfg.service(scope);
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("Not Found")
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    // init the global variable
    
    env_logger::init_from_env(Env::new().default_filter_or("warning"));
    let _ = common::TOKENIZED_OUTPUT.clone();

    let workers: usize = env::var("ACTIX_WORKERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        });

    let max_connection_rate = env::var("ACTIX_MAX_CONN_RATE")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(256);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .configure(configure)
            .default_service(web::route().to(not_found))
    })
    .client_request_timeout(Duration::from_secs(3600))
    .workers(workers)
    .max_connection_rate(max_connection_rate)
    .bind(("127.0.0.1", 8079))?
    .run()
    .await
}
