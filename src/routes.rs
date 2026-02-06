use axum::{
    extract::{Json, State},
    http::{StatusCode, header},
    response::{IntoResponse, sse::Event, sse::Sse},
};
use axum_extra::headers::authorization::{Authorization, Bearer};
use axum_extra::typed_header::TypedHeader;
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::generated;
use crate::stream::StringsStream;
use crate::template::render_chat_completion;

// Application state for holding the optional token
#[derive(Clone)]
pub struct AppState {
    pub token: Option<String>,
    pub inter_token_latency: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Request {
    max_tokens: Option<usize>,
    stream: Option<bool>,
    stream_options: Option<StreamOptions>,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

#[derive(Deserialize, Serialize, Debug)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

// Validate Bearer token against the configured token
fn validate_bearer_token(provided_token: &str, expected_token: &Option<String>) -> bool {
    match expected_token {
        Some(expected) => {
            // For a simple toy project, we'll use direct string comparison
            // In production, you might want to use constant-time comparison
            provided_token == expected
        }
        None => {
            // No token configured, so no authentication required
            true
        }
    }
}

pub async fn common_completions(
    State(state): State<AppState>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    Json(payload): Json<Request>,
) -> impl IntoResponse {
    // Check authentication if token is configured
    if let Some(expected_token) = &state.token {
        match auth_header {
            Some(TypedHeader(auth)) => {
                if !validate_bearer_token(auth.token(), &Some(expected_token.clone())) {
                    log::warn!("Authentication failed: invalid token");
                    return (
                        StatusCode::UNAUTHORIZED,
                        [(header::CONTENT_TYPE, "application/json")],
                        crate::template::ERROR_INVALID_API_KEY,
                    )
                        .into_response();
                }
            }
            None => {
                log::warn!("Authentication failed: missing Authorization header");
                return (
                    StatusCode::UNAUTHORIZED,
                    [(header::CONTENT_TYPE, "application/json")],
                    crate::template::ERROR_MISSING_API_KEY,
                )
                    .into_response();
            }
        }
    }

    log::info!(
        "Received completion request: stream={:?}, stream_options={:?}, max_tokens={:?}",
        payload.stream,
        payload.stream_options,
        payload.max_tokens
    );

    match payload.stream {
        Some(true) => {
            log::debug!("Processing streaming completion request");
            match streaming_completions(State(state.clone()), payload).await {
                Ok(stream) => {
                    log::debug!("Successfully created streaming completion");
                    Sse::new(stream).into_response()
                }
                Err(_) => {
                    log::error!("Failed to create streaming completion");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Stream error").into_response()
                }
            }
        }
        _ => {
            log::debug!("Processing non-streaming completion request");
            match normal_completions(payload).await {
                Ok(response) => {
                    log::debug!("Successfully created non-streaming completion");
                    (StatusCode::OK, response).into_response()
                }
                Err(_) => {
                    log::error!("Failed to create non-streaming completion");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Completion error").into_response()
                }
            }
        }
    }
}

async fn normal_completions(payload: Request) -> Result<String, ()> {
    let response = if let Some(max_tokens) = payload.max_tokens {
        log::debug!("Generating completion with {} max tokens", max_tokens);

        let return_string = if max_tokens >= generated::MAX_TOKENS {
            log::debug!("Requested tokens exceed available, using full output");
            generated::MAX_OUTPUT.to_string()
        } else {
            log::debug!("Using partial output with {} tokens", max_tokens);
            generated::TOKENIZED_OUTPUT[..max_tokens].join("")
        };
        render_chat_completion(&return_string, max_tokens, max_tokens)
    } else {
        log::debug!("No max_tokens specified, using full output");
        render_chat_completion(
            generated::MAX_OUTPUT,
            generated::MAX_TOKENS,
            generated::MAX_TOKENS,
        )
    };

    log::debug!("Generated response of {} characters", response.len());
    Ok(response)
}

async fn streaming_completions(
    State(state): State<AppState>,
    payload: Request,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, ()> {
    let requested_max_tokens = payload.max_tokens.unwrap_or(generated::MAX_TOKENS);
    let max_tokens = std::cmp::min(requested_max_tokens, generated::MAX_TOKENS);
    log::debug!(
        "Streaming completion: requested={}, actual={}, latency={}ms",
        requested_max_tokens,
        max_tokens,
        state.inter_token_latency
    );

    let log_usage: bool = payload
        .stream_options
        .unwrap_or(StreamOptions {
            include_usage: false,
        })
        .include_usage;
    log::debug!("Stream usage logging: {}", log_usage);

    let stream = StringsStream::new(
        generated::TOKENIZED_OUTPUT,
        Some(max_tokens),
        log_usage,
        state.inter_token_latency,
    )
    .map(|data| Ok(Event::default().data(data)));

    log::debug!("Created streaming completion with {} tokens", max_tokens);
    Ok(stream)
}
