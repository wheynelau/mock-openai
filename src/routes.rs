use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{sse::Event, sse::Sse, IntoResponse},
};
use axum_extra::headers::authorization::{Authorization, Bearer};
use axum_extra::typed_header::TypedHeader;
use futures_util::Stream;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::stream::StringsStream;

use crate::common::{MAX_OUTPUT, MAX_TOKENS, TOKENIZED_OUTPUT};

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

#[derive(Deserialize, Serialize, Debug)]
struct Response {
    id: String,
    object: String,
    created: i32,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

impl Response {
    fn from_response_string(response: String, max_tokens: usize) -> Self {
        Response {
            choices: vec![Choice {
                text: response,
                ..Choice::default()
            }],
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: max_tokens as i32,
                total_tokens: max_tokens as i32,
            },
            ..Default::default()
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self {
            id: String::from("chat-completion-000000000"),
            object: String::from("chat.completion"),
            created: 0,
            model: "gpt-1-turbo".to_string(),
            choices: vec![],
            usage: Usage::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Deserialize, Serialize, Debug)]
struct Choice {
    index: i32,
    text: String,
    logprobs: Option<f32>,
    finish_reason: String,
}

impl Default for Choice {
    fn default() -> Self {
        Self {
            text: String::default(),
            index: 0,
            logprobs: None,
            finish_reason: String::from("length"),
        }
    }
}

static TEMPLATE: Lazy<String> = Lazy::new(init_template);
static RE: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"\[INPUT\]|\[MAX_TOKENS\]").unwrap());

fn init_template() -> String {
    let max_tokens = i32::MAX;
    let response = Response::from_response_string(String::from("[INPUT]"), max_tokens as usize);
    let out = serde_json::to_string(&response).unwrap();
    out.replace(&max_tokens.to_string(), "[MAX_TOKENS]")
}

fn substitute_template(input: &str, max_tokens: usize, template: Option<&String>) -> String {
    let template = template.unwrap_or(&TEMPLATE);
    RE.replace_all(template, |caps: &regex::Captures| match &caps[0] {
        "[INPUT]" => input.to_string(),
        "[MAX_TOKENS]" => max_tokens.to_string(),
        _ => unreachable!(),
    })
    .to_string()
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
                        Json(serde_json::json!({
                            "error": {
                                "message": "Invalid API key",
                                "type": "invalid_request_error",
                                "code": "invalid_api_key"
                            }
                        })),
                    )
                        .into_response();
                }
            }
            None => {
                log::warn!("Authentication failed: missing Authorization header");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": {
                            "message": "Missing Authorization header",
                            "type": "invalid_request_error",
                            "code": "missing_api_key"
                        }
                    })),
                )
                    .into_response();
            }
        }
    }

    log::info!(
        "Received completion request: stream={:?}, max_tokens={:?}",
        payload.stream,
        payload.max_tokens
    );

    match payload.stream {
        Some(true) => {
            log::debug!("Processing streaming completion request");
            match chat_completions(State(state.clone()), payload).await {
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
            match completions(payload).await {
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

async fn completions(payload: Request) -> Result<String, ()> {
    let response = if payload.max_tokens.is_some() {
        let max_tokens: usize = payload.max_tokens.unwrap();
        log::debug!("Generating completion with {} max tokens", max_tokens);

        let return_string = if max_tokens >= *MAX_TOKENS {
            log::debug!("Requested tokens exceed available, using full output");
            MAX_OUTPUT.to_string()
        } else {
            log::debug!("Using partial output with {} tokens", max_tokens);
            TOKENIZED_OUTPUT[..max_tokens].join("")
        };
        substitute_template(&return_string, max_tokens, None)
    } else {
        log::debug!("No max_tokens specified, using full output");
        substitute_template(&MAX_OUTPUT, *MAX_TOKENS, None)
    };

    log::debug!("Generated response of {} characters", response.len());
    Ok(response)
}

async fn chat_completions(
    State(state): State<AppState>,
    payload: Request,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, ()> {
    let requested_max_tokens = payload.max_tokens.unwrap_or(*MAX_TOKENS);
    let max_tokens = std::cmp::min(requested_max_tokens, *MAX_TOKENS);
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
        TOKENIZED_OUTPUT.as_slice(),
        Some(max_tokens),
        log_usage,
        state.inter_token_latency,
    )
    .map(|data| Ok(Event::default().data(data)));

    log::debug!("Created streaming completion with {} tokens", max_tokens);
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_partial_struct() {
        let template = init_template();
        let input = "\tHello World!";
        let input = serde_json::to_string(&input)
            .unwrap()
            .trim_matches('"')
            .to_string();
        let max_tokens = 10;
        let response = substitute_template(&input, max_tokens, Some(&template));

        let baseline = Response::from_response_string(String::from("\tHello World!"), 10);
        let baseline = serde_json::to_string(&baseline).unwrap();
        assert_eq!(response, baseline);
    }
}
