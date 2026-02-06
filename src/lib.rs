pub mod args;
pub mod routes;
pub mod stream;
pub mod template;

pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use tower_http::trace::TraceLayer;

pub use routes::{AppState, Request};

pub async fn start_server(
    address: &str,
    port: u16,
    token: Option<String>,
    inter_token_latency: u64,
) -> std::io::Result<()> {
    log::info!("Configuring application routes");

    // Create application state
    let app_state = AppState {
        token,
        inter_token_latency,
    };

    // Log authentication configuration
    if app_state.token.is_some() {
        log::info!("Authentication: Bearer token authentication enabled");
    } else {
        log::info!("Authentication: No token configured - accepting all requests");
    }

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        .route("/tokens", get(get_max_tokens))
        .nest("/v1", v1_routes())
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    log::info!("Binding server to {}:{}", address, port);
    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", address, port))
        .await
        .map_err(|e| {
            log::error!("Failed to bind to {}:{}: {}", address, port, e);
            e
        })?;
    log::info!("Server successfully bound to {}:{}", address, port);
    log::info!("Starting to accept incoming connections");
    axum::serve(listener, app).await
}

async fn health() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_max_tokens() -> impl IntoResponse {
    format!("Max tokens: {}", generated::MAX_TOKENS)
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

fn v1_routes() -> Router<AppState> {
    Router::new()
        .route("/completions", post(routes::common_completions))
        .route("/chat/completions", post(routes::common_completions))
}
