use actix_web_lab::sse;
use futures_util::stream::Stream;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::common::MAX_TOKENS;
use crate::routes::Usage;

enum State {
    Start,
    Done,
    Usage,
    Completed,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct StreamingChunkResponse {
    id: String,
    object: String,
    created: i32,
    model: String,
    system_fingerprint: String,
    choices: Vec<Choice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<Usage>,
}
impl Default for StreamingChunkResponse {
    fn default() -> Self {
        Self {
            id: String::from("chat-completion-000000000"),
            object: String::from("chat.completion.chunk"),
            created: 0,
            model: "gpt-1-turbo".to_string(),
            system_fingerprint: "fp_0000000000".to_string(),
            choices: vec![],
            usage: None,
        }
    }
}
impl StreamingChunkResponse {
    pub fn from_string(response: String) -> Self {
        StreamingChunkResponse {
            choices: vec![Choice {
                delta: Delta {
                    role: Some("assistant".to_string()),
                    content: Some(response),
                },
                ..Choice::default()
            }],
            ..Default::default()
        }
    }
}
#[derive(Deserialize, Serialize, Debug, Default)]
struct Choice {
    index: i32,
    delta: Delta,
    logprobs: Option<String>,
    finish_reason: Option<String>,
}

// optional for role and content because the final chunk does not have anything inside delta: {}
#[derive(Deserialize, Serialize, Debug)]
struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}
impl Default for Delta {
    fn default() -> Self {
        Self {
            role: Some("assistant".to_string()),
            content: Some("".to_string()),
        }
    }
}
pub struct StringsStream<'a> {
    strings: &'a Vec<String>,
    index: usize,
    max_tokens: usize,
    include_usage: bool,
    state: State,
}

impl<'a> StringsStream<'a> {
    pub fn new(strings: &'a Vec<String>, max_tokens: Option<usize>, include_usage: bool) -> Self {
        StringsStream {
            strings,
            index: 0,
            max_tokens: max_tokens.unwrap_or(*MAX_TOKENS),
            include_usage,
            state: State::Start,
        }
    }
}

static TEMPLATE: Lazy<String> = Lazy::new(init_template);

fn init_template() -> String {
    let response = StreamingChunkResponse::from_string("[INPUT]".to_string());
    serde_json::to_string(&response).unwrap()
}

impl Stream for StringsStream<'_> {
    type Item = Result<sse::Event, std::convert::Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        // high level
        // Starts with state::Start
        // Once it reaches the end of the strings, it will switch to state::Usage if log usage is enabled
        // After state::Usage, it will switch to state::Done
        // If log usage is not enabled, it will switch to state::Done
        // Once it reaches state::Done, it will switch to state::Completed

        // init a string for faster access
        // let response = StreamingChunkResponse::from_string("[INPUT]".to_string());
        // let output = serde_json::to_string(&response).unwrap();

        match this.state {
            State::Start => {
                if this.index < this.max_tokens {
                    let string_item = &this.strings[this.index];
                    this.index += 1;
                    // let chunk = StreamingChunkResponse::from_string(string_item);
                    // let string_item = serde_json::to_string(&chunk).unwrap();
                    let string_item = TEMPLATE.replace("[INPUT]", string_item);
                    Poll::Ready(Some(Ok(sse::Event::Data(sse::Data::new(string_item)))))
                } else {
                    if this.include_usage {
                        this.state = State::Usage;
                    } else {
                        this.state = State::Done;
                    }
                    let chunk = StreamingChunkResponse {
                        choices: vec![Choice {
                            finish_reason: Some("stop".to_string()),
                            delta: Delta {
                                role: None,
                                content: None,
                            },
                            ..Choice::default()
                        }],
                        ..Default::default()
                    };
                    let string_item = serde_json::to_string(&chunk).unwrap();
                    Poll::Ready(Some(Ok(sse::Event::Data(sse::Data::new(string_item)))))
                    // TODO: implement the final cost chunk
                }
            }
            State::Usage => {
                this.state = State::Done;
                let chunk = StreamingChunkResponse {
                    choices: vec![], // empty choices
                    usage: Some(Usage {
                        prompt_tokens: 0,
                        completion_tokens: this.max_tokens as i32,
                        total_tokens: this.max_tokens as i32,
                    }),
                    ..Default::default()
                };
                // This is supposed to send usage
                let string_item = serde_json::to_string(&chunk).unwrap();
                Poll::Ready(Some(Ok(sse::Event::Data(sse::Data::new(string_item)))))
            }
            State::Done => {
                this.state = State::Completed;
                Poll::Ready(Some(Ok(sse::Event::Data(sse::Data::new(
                    "[DONE]".to_string(),
                )))))
            }
            State::Completed => Poll::Ready(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_partial_struct() {
        let response = StreamingChunkResponse::from_string("{INPUT}".to_string());
        assert_eq!(response.choices.len(), 1);
        let output = serde_json::to_string(&response).unwrap();
        println!("{}", output);
        // test format
        println!("{}", output.replace("{INPUT}", "Hello World!"));
    }
}
