pub mod args;
pub mod common;
pub mod routes;
pub mod stream;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

pub use routes::Request;

pub async fn start_server(address: &str, port: u16) -> std::io::Result<()> {
    // Build our application with routes
    let app = Router::new()
        .route("/hello", get(hello))
        .route("/echo", post(echo))
        .route("/tokens", get(get_max_tokens))
        .nest("/v1", v1_routes())
        .fallback(not_found)
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", address, port))
        .await
        .unwrap();
    axum::serve(listener, app).await
}

async fn hello() -> impl IntoResponse {
    "Hello world!"
}

async fn echo(body: String) -> impl IntoResponse {
    body
}

async fn get_max_tokens() -> impl IntoResponse {
    format!("Max tokens: {}", *common::MAX_TOKENS)
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

fn v1_routes() -> Router {
    Router::new()
        .route("/completions", post(routes::common_completions))
        .route("/chat/completions", post(routes::common_completions))
}
