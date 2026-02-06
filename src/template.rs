/// Absolutely pointless optimisation
///
/// Rather that using format or replace, we know how the template looks like
///
/// So we can just split it into parts and concatenate them
///
/// Start chunk + content + end chunk
/// Rather than format!("{}{}{}", start, content, end) or SSE_TEMPLATE.replace("{content}", content)

const SSE_CHUNK_PREFIX: &str = r#"{"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1770187171,"model":"sonnet-mock-model","choices":[{"index":0,"delta":{"content":"#;
const SSE_CHUNK_SUFFIX: &str =
    r#","reasoning_content":null},"logprobs":null,"finish_reason":null,"token_ids":null}]}"#;

// start chunk is the same as above
const SSE_FINISH_SUFFIX: &str =
    r#","reasoning_content":null},"logprobs":null,"finish_reason":"length","token_ids":null}]}"#;

// can be faster if we don't need the numbers
const SSE_USAGE_PREFIX: &str = r#"{"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1770187171,"model":"sonnet-mock-model","choices":[],"usage":{"prompt_tokens":0,"completion_tokens":"#;
const SSE_USAGE_MID: &str = r#","total_tokens":"#;
const SSE_USAGE_SUFFIX: &str = r#"}}"#;

// does this even need a template?
const SSE_TEMPLATE_DONE: &str = "[DONE]";

// error messages, honestly they don't trigger much but its fine
pub const ERROR_INVALID_API_KEY: &str = r#"{"error":{"message":"Invalid API key","type":"invalid_request_error","code":"invalid_api_key"}}"#;
pub const ERROR_MISSING_API_KEY: &str = r#"{"error":{"message":"Missing Authorization header","type":"invalid_request_error","code":"missing_api_key"}}"#;

/// Pre-split chat completion template
const CHAT_PREFIX: &str = r#"{"id":"chatcmpl-xxx","object":"chat.completion","created":1770188771,"model":"sonnet-mock-model","choices":[{"index":0,"message":{"role":"assistant","content":"#;
const CHAT_MID1: &str = r#","refusal":null,"annotations":null,"audio":null,"function_call":null,"tool_calls":[],"reasoning":null,"reasoning_content":null},"logprobs":null,"finish_reason":"length","stop_reason":null,"token_ids":null}],"usage":{"prompt_tokens":0,"total_tokens":"#;
const CHAT_MID2: &str = r#","completion_tokens":"#;
const CHAT_SUFFIX: &str = r#","prompt_tokens_details":null}}"#;

#[inline]
pub fn render_sse_chunk(content: &str) -> String {
    let mut result =
        String::with_capacity(SSE_CHUNK_PREFIX.len() + content.len() + SSE_CHUNK_SUFFIX.len());
    result.push_str(SSE_CHUNK_PREFIX);
    result.push_str(content);
    result.push_str(SSE_CHUNK_SUFFIX);
    result
}

#[inline]
pub fn render_sse_finish(content: &str) -> String {
    let mut result =
        String::with_capacity(SSE_CHUNK_PREFIX.len() + content.len() + SSE_FINISH_SUFFIX.len());
    result.push_str(SSE_CHUNK_PREFIX);
    result.push_str(content);
    result.push_str(SSE_FINISH_SUFFIX);
    result
}

#[inline]
pub fn render_sse_usage(completion_tokens: usize, total_tokens: usize) -> String {
    // Format numbers first to get exact length
    let ct_str = completion_tokens.to_string();
    let tt_str = total_tokens.to_string();

    let mut result = String::with_capacity(
        SSE_USAGE_PREFIX.len()
            + ct_str.len()
            + SSE_USAGE_MID.len()
            + tt_str.len()
            + SSE_USAGE_SUFFIX.len(),
    );
    result.push_str(SSE_USAGE_PREFIX);
    result.push_str(&ct_str);
    result.push_str(SSE_USAGE_MID);
    result.push_str(&tt_str);
    result.push_str(SSE_USAGE_SUFFIX);
    result
}

#[inline]
pub fn render_chat_completion(
    content: &str,
    completion_tokens: usize,
    total_tokens: usize,
) -> String {
    let tt_str = total_tokens.to_string();
    let ct_str = completion_tokens.to_string();

    let mut result = String::with_capacity(
        CHAT_PREFIX.len()
            + content.len()
            + CHAT_MID1.len()
            + tt_str.len()
            + CHAT_MID2.len()
            + ct_str.len()
            + CHAT_SUFFIX.len(),
    );
    result.push_str(CHAT_PREFIX);
    result.push_str(content);
    result.push_str(CHAT_MID1);
    result.push_str(&tt_str);
    result.push_str(CHAT_MID2);
    result.push_str(&ct_str);
    result.push_str(CHAT_SUFFIX);
    result
}

#[inline]
pub fn render_sse_done() -> String {
    SSE_TEMPLATE_DONE.to_string()
}
