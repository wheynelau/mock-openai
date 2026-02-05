use futures_util::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::{self, Duration, Instant};

use crate::template;
pub struct StringsStream<'a> {
    tokens: &'a [&'a str],
    index: usize,
    max_tokens: usize,
    log_usage: bool,
    interval: Option<time::Interval>,
    usage_sent: bool,
    done_sent: bool,
}

impl<'a> StringsStream<'a> {
    pub fn new(
        tokens: &'a [&'a str],
        max_tokens: Option<usize>,
        log_usage: bool,
        inter_token_latency: u64,
    ) -> Self {
        // reserve one token for finish reason
        let max_tokens = max_tokens.unwrap_or(tokens.len()) - 1;
        // also need to make sure max_tokens is at least 1 to send finish reason
        let max_tokens = std::cmp::max(1, max_tokens);
        let interval = if inter_token_latency > 0 {
            Some(time::interval_at(
                Instant::now() + Duration::from_millis(inter_token_latency),
                Duration::from_millis(inter_token_latency),
            ))
        } else {
            None
        };

        StringsStream {
            tokens,
            index: 0,
            max_tokens,
            log_usage,
            interval,
            usage_sent: false,
            done_sent: false,
        }
    }
    fn get_token(&self) -> Option<&'a str> {
        if self.tokens.is_empty() {
            None
        } else {
            Some(self.tokens[self.index % self.tokens.len()])
        }
    }
}

impl Stream for StringsStream<'_> {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(interval) = &mut self.interval
            && interval.poll_tick(cx).is_pending()
        {
            return Poll::Pending;
        }
        if self.index < self.max_tokens {
            if let Some(token) = self.get_token() {
                self.index += 1;
                return Poll::Ready(Some(template::render_sse_chunk(token)));
            }
            // If get_token returns None (empty array), just fall through to end
            self.index = self.max_tokens;
        }

        // Send the finish reason
        if self.index == self.max_tokens {
            if let Some(token) = self.get_token() {
                self.index += 1;
                return Poll::Ready(Some(template::render_sse_finish(token)));
            }
            return Poll::Ready(Some(template::render_sse_finish("")));
        }

        // 2. Send usage, usually second last message
        if self.log_usage && !self.usage_sent {
            self.usage_sent = true;
            let usage = template::render_sse_usage(self.max_tokens + 1, self.max_tokens + 1);
            return Poll::Ready(Some(usage));
        }

        // 3. Send the done message
        if !self.done_sent {
            self.done_sent = true;
            return Poll::Ready(Some(template::render_sse_done()));
        }

        // Finally, eos
        Poll::Ready(None)
    }
}
