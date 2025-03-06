use actix_web_lab::sse;
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::common::MAX_TOKENS;

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
#[derive(Deserialize, Serialize, Debug, Default)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}
pub struct StringsStream {
    strings: Vec<String>,
    index: usize,
    max_tokens: usize,
    include_usage: bool,
    state: State,
}

impl StringsStream {
    pub fn new(strings: Vec<String>, max_tokens: Option<usize>, include_usage: bool) -> Self {
        StringsStream {
            strings,
            index: 0,
            max_tokens: max_tokens.unwrap_or(*MAX_TOKENS),
            include_usage,
            state: State::Start,
        }
    }
}

impl Stream for StringsStream {
    type Item = Result<sse::Event, std::convert::Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        match this.state {
            State::Start => {
                if this.index < this.max_tokens {
                    let string_item = this.strings[this.index].clone();
                    this.index += 1;
                    let chunk = StreamingChunkResponse::from_string(string_item);
                    let string_item = serde_json::to_string(&chunk).unwrap();
                    Poll::Ready(Some(Ok(sse::Event::Data(sse::Data::new(string_item)))))
                } else {
                    this.state = State::Done;
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
            State::Done => {
                this.state = State::Usage;
                Poll::Ready(Some(Ok(sse::Event::Data(sse::Data::new(
                    "[DONE]".to_string(),
                )))))
            }
            State::Usage => {
                if this.include_usage {
                    this.state = State::Completed;
                } else {
                    this.state = State::Done;
                }
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
            _ => Poll::Ready(None),
        }
    }
}
