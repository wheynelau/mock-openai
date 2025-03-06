pub mod args;
pub mod common;
pub mod routes;
pub mod stream;

use actix_web::{HttpResponse, Responder, middleware::Logger};
use env_logger::Env;

use actix_web::{App, HttpServer, web};

fn configure(cfg: &mut web::ServiceConfig) {
    let scope = web::scope("/v1")
        .route("/completions", web::post().to(routes::common_completions))
        .route(
            "/chat/completions",
            web::post().to(routes::common_completions),
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

    let args = args::get_args();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .configure(configure)
            .default_service(web::route().to(not_found))
    })
    .client_request_timeout(args.client_request_timeout)
    .workers(args.workers)
    .max_connection_rate(args.max_connection_rate)
    .bind((args.address, args.port))?
    .run()
    .await
}
