use futures_util::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::{self, Duration, Instant};

pub struct StringsStream {
    tokens: Vec<String>,
    index: usize,
    log_usage: bool,
    interval: time::Interval,
    usage_sent: bool, // Track if usage message has been sent
}

impl StringsStream {
    pub fn new(tokens: &[String], max_tokens: Option<usize>, log_usage: bool) -> Self {
        let tokens = tokens.to_vec();
        let max_tokens = max_tokens.unwrap_or(tokens.len());
        let tokens = tokens.into_iter().take(max_tokens).collect();

        StringsStream {
            tokens,
            index: 0,
            log_usage,
            interval: time::interval_at(
                Instant::now() + Duration::from_millis(10),
                Duration::from_millis(10),
            ),
            usage_sent: false,
        }
    }
}

impl Stream for StringsStream {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.index >= self.tokens.len() {
            if self.log_usage && !self.usage_sent {
                self.usage_sent = true;
                let usage = format!(
                    r#"{{"usage": {{"prompt_tokens": 0, "completion_tokens": {}, "total_tokens": {}}}}}"#,
                    self.tokens.len(),
                    self.tokens.len()
                );
                return Poll::Ready(Some(usage));
            }
            return Poll::Ready(None);
        }

        // Wait for the next interval tick
        if self.interval.poll_tick(cx).is_pending() {
            return Poll::Pending;
        }

        let token = self.tokens[self.index].clone();
        self.index += 1;
        Poll::Ready(Some(token))
    }
}
