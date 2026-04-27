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

// Pre-compute constant lengths so we don't re-evaluate .len() on every call
const SSE_CHUNK_FIXED_LEN: usize = SSE_CHUNK_PREFIX.len() + SSE_CHUNK_SUFFIX.len();
const SSE_FINISH_FIXED_LEN: usize = SSE_CHUNK_PREFIX.len() + SSE_FINISH_SUFFIX.len();
const SSE_USAGE_FIXED_LEN: usize =
    SSE_USAGE_PREFIX.len() + SSE_USAGE_MID.len() + SSE_USAGE_SUFFIX.len();
const CHAT_FIXED_LEN: usize =
    CHAT_PREFIX.len() + CHAT_MID1.len() + CHAT_MID2.len() + CHAT_SUFFIX.len();

/// Fast integer-to-string formatting via raw pointer write.
/// Writes digits at `ptr.add(pos)` and returns the position after the last digit.
#[inline(always)]
fn write_usize(ptr: *mut u8, n: usize, pos: usize) -> usize {
    if n == 0 {
        unsafe { ptr.add(pos).write(b'0') };
        return pos + 1;
    }
    // Extract digits into a small stack buffer (max 20 digits for usize)
    let mut digits = [0u8; 20];
    let mut idx = 0;
    let mut temp = n;
    while temp > 0 {
        digits[idx] = b'0' + (temp % 10) as u8;
        temp /= 10;
        idx += 1;
    }
    // Write digits in correct order
    let mut out = pos;
    while idx > 0 {
        idx -= 1;
        unsafe { ptr.add(out).write(digits[idx]) };
        out += 1;
    }
    out
}

/// Copy a string slice into a raw buffer at the given offset, advancing the offset.
#[inline(always)]
fn copy_advance(ptr: *mut u8, pos: usize, src: &str) -> usize {
    let src_bytes = src.as_bytes();
    unsafe {
        ptr.add(pos)
            .copy_from_nonoverlapping(src_bytes.as_ptr(), src_bytes.len())
    };
    pos + src_bytes.len()
}

#[inline(always)]
pub fn render_sse_chunk(content: &str) -> String {
    let total = SSE_CHUNK_FIXED_LEN + content.len();
    let mut buf = Vec::<u8>::with_capacity(total);
    let ptr = buf.as_mut_ptr();

    let pos = 0;
    let pos = copy_advance(ptr, pos, SSE_CHUNK_PREFIX);
    let pos = copy_advance(ptr, pos, content);
    let pos = copy_advance(ptr, pos, SSE_CHUNK_SUFFIX);

    unsafe { buf.set_len(pos) };
    // Safety: we only write ASCII bytes and &str content, all valid UTF-8
    unsafe { String::from_utf8_unchecked(buf) }
}

#[inline(always)]
pub fn render_sse_finish(content: &str) -> String {
    let total = SSE_FINISH_FIXED_LEN + content.len();
    let mut buf = Vec::<u8>::with_capacity(total);
    let ptr = buf.as_mut_ptr();

    let pos = 0;
    let pos = copy_advance(ptr, pos, SSE_CHUNK_PREFIX);
    let pos = copy_advance(ptr, pos, content);
    let pos = copy_advance(ptr, pos, SSE_FINISH_SUFFIX);

    unsafe { buf.set_len(pos) };
    // Safety: we only write ASCII bytes and &str content, all valid UTF-8
    unsafe { String::from_utf8_unchecked(buf) }
}

#[inline(always)]
pub fn render_sse_usage(completion_tokens: usize, total_tokens: usize) -> String {
    let ct_digits = if completion_tokens == 0 {
        1
    } else {
        completion_tokens.ilog10() as usize + 1
    };
    let tt_digits = if total_tokens == 0 {
        1
    } else {
        total_tokens.ilog10() as usize + 1
    };

    let mut buf = Vec::<u8>::with_capacity(SSE_USAGE_FIXED_LEN + ct_digits + tt_digits);
    let ptr = buf.as_mut_ptr();

    let mut pos = 0;
    pos = copy_advance(ptr, pos, SSE_USAGE_PREFIX);
    pos = write_usize(ptr, completion_tokens, pos);
    pos = copy_advance(ptr, pos, SSE_USAGE_MID);
    pos = write_usize(ptr, total_tokens, pos);
    pos = copy_advance(ptr, pos, SSE_USAGE_SUFFIX);

    unsafe { buf.set_len(pos) };
    // Safety: we only write ASCII bytes and &str content, all valid UTF-8
    unsafe { String::from_utf8_unchecked(buf) }
}

#[inline(always)]
pub fn render_chat_completion(
    content: &str,
    completion_tokens: usize,
    total_tokens: usize,
) -> String {
    let ct_digits = if completion_tokens == 0 {
        1
    } else {
        completion_tokens.ilog10() as usize + 1
    };
    let tt_digits = if total_tokens == 0 {
        1
    } else {
        total_tokens.ilog10() as usize + 1
    };

    let mut buf = Vec::<u8>::with_capacity(CHAT_FIXED_LEN + content.len() + ct_digits + tt_digits);
    let ptr = buf.as_mut_ptr();

    let mut pos = 0;
    pos = copy_advance(ptr, pos, CHAT_PREFIX);
    pos = copy_advance(ptr, pos, content);
    pos = copy_advance(ptr, pos, CHAT_MID1);
    pos = write_usize(ptr, total_tokens, pos);
    pos = copy_advance(ptr, pos, CHAT_MID2);
    pos = write_usize(ptr, completion_tokens, pos);
    pos = copy_advance(ptr, pos, CHAT_SUFFIX);

    unsafe { buf.set_len(pos) };
    // Safety: we only write ASCII bytes and &str content, all valid UTF-8
    unsafe { String::from_utf8_unchecked(buf) }
}

#[inline]
pub fn render_sse_done() -> String {
    SSE_TEMPLATE_DONE.into()
}
