use axum::{
    extract::Json,
    http::StatusCode,
    response::{sse::Event, sse::Sse, IntoResponse},
};
use futures_util::Stream;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::stream::StringsStream;

use crate::common::{MAX_OUTPUT, MAX_TOKENS, TOKENIZED_OUTPUT};

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

pub async fn common_completions(Json(payload): Json<Request>) -> impl IntoResponse {
    match payload.stream {
        Some(true) => match chat_completions(payload).await {
            Ok(stream) => Sse::new(stream).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Stream error").into_response(),
        },
        _ => match completions(payload).await {
            Ok(response) => (StatusCode::OK, response).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Completion error").into_response(),
        },
    }
}

async fn completions(payload: Request) -> Result<String, ()> {
    let response = if payload.max_tokens.is_some() {
        let max_tokens: usize = payload.max_tokens.unwrap();
        let return_string = if max_tokens >= *MAX_TOKENS {
            MAX_OUTPUT.to_string()
        } else {
            TOKENIZED_OUTPUT[..max_tokens].join("")
        };
        substitute_template(&return_string, max_tokens, None)
    } else {
        substitute_template(&MAX_OUTPUT, *MAX_TOKENS, None)
    };

    Ok(response)
}

async fn chat_completions(
    payload: Request,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, ()> {
    let max_tokens = payload.max_tokens.unwrap_or(*MAX_TOKENS);
    let max_tokens = std::cmp::min(max_tokens, *MAX_TOKENS);
    let log_usage: bool = payload
        .stream_options
        .unwrap_or(StreamOptions {
            include_usage: false,
        })
        .include_usage;

    let stream = StringsStream::new(TOKENIZED_OUTPUT.as_slice(), Some(max_tokens), log_usage)
        .map(|data| Ok(Event::default().data(data)));

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
