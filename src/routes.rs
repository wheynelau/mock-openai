use actix_web::Either;
use actix_web::{error::HttpError, web, HttpRequest, HttpResponse};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use actix_web_lab::sse::{self, Sse};
use futures_util::Stream;
use std::convert::Infallible;

use crate::stream::StringsStream;

use crate::common::{MAX_OUTPUT, MAX_TOKENS, TOKENIZED_OUTPUT};
#[derive(Deserialize, Serialize, Debug)]
pub struct Request {
    max_tokens: Option<usize>,
    // extras
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

fn init_template() -> String {
    let max_tokens = i32::MAX;
    let response = Response::from_response_string(String::from("[INPUT]"), max_tokens as usize);
    let out = serde_json::to_string(&response).unwrap();
    // replace maxtokens with a placeholder
    out.replace(&max_tokens.to_string(), "[MAX_TOKENS]")
}

fn substitute_template(input: &str, max_tokens: usize, template: Option<&String>) -> String {
    let template = template.unwrap_or_else(|| &TEMPLATE);
    let response = template.replace("[INPUT]", input);
    response.replace("[MAX_TOKENS]", &max_tokens.to_string())
}
// Use the same endpoint to allow the streaming
pub async fn common_completions(
    _req: HttpRequest,
    payload: web::Json<Request>,
) -> Result<Either<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, HttpResponse>, HttpError>
{
    // Return Either Left -> streaming, Right-> normal
    let payload = payload.into_inner();
    // route to streaming
    match payload.stream {
        Some(true) => Ok(Either::Left(chat_completions(_req, payload).await?)),
        _ => Ok(Either::Right(completions(_req, payload).await?)),
    }
}

async fn completions(_req: HttpRequest, payload: Request) -> Result<HttpResponse, HttpError> {
    // This returns the string
    // check if there is max_tokens inside the payload
    let response: String = if payload.max_tokens.is_some() {
        // Only slice if max_tokens is explicitly provided
        let max_tokens: usize = payload.max_tokens.unwrap();
        // TODO Allow users to request for max_tokens> MAX_TOKENS
        // This would require a wrapper or custom Impl for indexing
        // For now restrict as a safety
        let return_string = if max_tokens >= *MAX_TOKENS {
            MAX_OUTPUT.clone()
        } else {
            TOKENIZED_OUTPUT[..max_tokens].concat()
        };
        substitute_template(&return_string, max_tokens, None)
    } else {
        // Use the full output when max_tokens is not specified
        let return_string = &MAX_OUTPUT;
        substitute_template(return_string, *MAX_TOKENS, None)
    };

    Ok(HttpResponse::Ok().body(response))
}

async fn chat_completions(
    _req: HttpRequest,
    payload: Request,
) -> Result<Sse<impl Stream<Item = Result<sse::Event, Infallible>>>, HttpError> {
    // Same as above, restrict
    // TODO: Implement max_tokens > MAX_TOKENS
    let max_tokens = payload.max_tokens.unwrap_or(*MAX_TOKENS);
    let max_tokens = std::cmp::min(max_tokens, *MAX_TOKENS);
    let log_usage: bool = payload
        .stream_options
        .unwrap_or(StreamOptions {
            include_usage: false,
        })
        .include_usage;
    let stream = StringsStream::new(&TOKENIZED_OUTPUT, Some(max_tokens), log_usage);
    Ok(Sse::from_stream(stream))
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
        // serialize the baseline
        let baseline = serde_json::to_string(&baseline).unwrap();
        assert_eq!(response, baseline);
    }
    // TODO: Implement tests for the response
}
