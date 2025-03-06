use actix_web::{HttpRequest, HttpResponse, error::HttpError, web};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use actix_web_lab::sse::{self, Sse};
use futures_util::Stream;
use std::convert::Infallible;

use crate::stream::StringsStream;

use crate::common::{MAX_TOKENS, TOKENIZED_OUTPUT};
#[derive(Deserialize, Serialize, Debug)]
pub struct Request {
    max_tokens: Option<usize>,
    // extras
    #[serde(flatten)]
    stream_options: Option<StreamOptions>,
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
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Deserialize, Serialize, Debug)]
struct Choice {
    text: String,
    index: u64,
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

pub async fn completions(
    _req: HttpRequest,
    payload: web::Json<Request>,
) -> Result<HttpResponse, HttpError> {
    // This returns the string
    // check if there is max_tokens inside the payload
    let payload = payload.into_inner();
    let response: Response = if payload.max_tokens.is_some() {
        // Only slice if max_tokens is explicitly provided
        let max_tokens = payload.max_tokens.unwrap();
        let return_string = TOKENIZED_OUTPUT[..max_tokens].join("");
        Response::from_response_string(return_string, max_tokens)
    } else {
        // Use the full output when max_tokens is not specified
        let return_string = TOKENIZED_OUTPUT.join("");
        Response::from_response_string(return_string, *MAX_TOKENS)
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn chat_completions(
    _req: HttpRequest,
    payload: web::Json<Request>,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, HttpError> {
    let payload = payload.into_inner();
    let max_tokens = payload.max_tokens.unwrap_or(*MAX_TOKENS);
    let log_usage: bool = payload
        .stream_options
        .unwrap_or(StreamOptions {
            include_usage: false,
        })
        .include_usage;
    let stream = StringsStream::new(TOKENIZED_OUTPUT.clone(), Some(max_tokens), log_usage);
    Ok(Sse::from_stream(stream))
}
