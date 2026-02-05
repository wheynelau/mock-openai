/// This is the standard SSE template with content
const SSE_TEMPLATE_CHUNK: &str = r#"{"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1770187171,"model":"sonnet-mock-model","choices":[{"index":0,"delta":{"content":{{CONTENT}},"reasoning_content":null},"logprobs":null,"finish_reason":null,"token_ids":null}]}"#;

// this chunk is for the finish_reason with content
const SSE_TEMPLATE_FINISH: &str = r#"{"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1770187171,"model":"sonnet-mock-model","choices":[{"index":0,"delta":{"content":{{CONTENT}},"reasoning_content":null},"logprobs":null,"finish_reason":"length","token_ids":null}]}"#;

const SSE_TEMPLATE_USAGE: &str = r#"{"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1770187171,"model":"sonnet-mock-model","choices":[],"usage":{"prompt_tokens":0,"completion_tokens":{{COMPLETION_TOKENS}},"total_tokens":{{TOTAL_TOKENS}}}}"#;
const SSE_TEMPLATE_DONE: &str = "[DONE]";

const CHAT_COMPLETION_TEMPLATE: &str = r#"{"id":"chatcmpl-xxx","object":"chat.completion","created":1770188771,"model":"sonnet-mock-model","choices":[{"index":0,"message":{"role":"assistant","content":{{CONTENT}},"refusal":null,"annotations":null,"audio":null,"function_call":null,"tool_calls":[],"reasoning":null,"reasoning_content":null},"logprobs":null,"finish_reason":"length","stop_reason":null,"token_ids":null}],"usage":{"prompt_tokens":0,"total_tokens":{{TOTAL_TOKENS}},"completion_tokens":{{COMPLETION_TOKENS}},"prompt_tokens_details":null}}"#;

pub fn render_sse_chunk(content: &str) -> String {
    let escaped = serde_json::to_string(content).expect("Failed to escape content");
    SSE_TEMPLATE_CHUNK.replace("{{CONTENT}}", &escaped)
}

pub fn render_sse_finish(content: &str) -> String {
    let escaped = serde_json::to_string(content).expect("Failed to escape content");
    SSE_TEMPLATE_FINISH.replace("{{CONTENT}}", &escaped)
}

pub fn render_sse_usage(completion_tokens: usize, total_tokens: usize) -> String {
    SSE_TEMPLATE_USAGE
        .replace("{{COMPLETION_TOKENS}}", &completion_tokens.to_string())
        .replace("{{TOTAL_TOKENS}}", &total_tokens.to_string())
}

pub fn render_chat_completion(
    content: &str,
    completion_tokens: usize,
    total_tokens: usize,
) -> String {
    let escaped = serde_json::to_string(content).expect("Failed to escape content");
    CHAT_COMPLETION_TEMPLATE
        .replace("{{CONTENT}}", &escaped)
        .replace("{{COMPLETION_TOKENS}}", &completion_tokens.to_string())
        .replace("{{TOTAL_TOKENS}}", &total_tokens.to_string())
}

pub fn render_sse_done() -> String {
    SSE_TEMPLATE_DONE.to_string()
}
