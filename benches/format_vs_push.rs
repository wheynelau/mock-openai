use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

// just an experiment to see which is faster for generating the JSON strings
// for learning purposes only
const PREFIX: &str = r#"{"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1770187171,"model":"sonnet-mock-model","choices":[{"index":0,"delta":{"content":"#;
const SUFFIX: &str =
    r#","reasoning_content":null},"logprobs":null,"finish_reason":null,"token_ids":null}]}"#;

fn using_push_str(content: &str) -> String {
    let mut result = String::with_capacity(PREFIX.len() + content.len() + SUFFIX.len());
    result.push_str(PREFIX);
    result.push_str(content);
    result.push_str(SUFFIX);
    result
}

fn using_format(content: &str) -> String {
    format!("{}{}{}", PREFIX, content, SUFFIX)
}

fn using_format_with_capacity(content: &str) -> String {
    let mut result = String::with_capacity(PREFIX.len() + content.len() + SUFFIX.len());
    use std::fmt::Write;
    let _ = write!(result, "{}{}{}", PREFIX, content, SUFFIX);
    result
}

fn bench_push_str(c: &mut Criterion) {
    c.bench_function("push_str", |b| {
        b.iter(|| using_push_str(black_box("\"Hello world\"")))
    });
}

fn bench_format(c: &mut Criterion) {
    c.bench_function("format!", |b| {
        b.iter(|| using_format(black_box("\"Hello world\"")))
    });
}

fn bench_format_with_capacity(c: &mut Criterion) {
    c.bench_function("format! with capacity", |b| {
        b.iter(|| using_format_with_capacity(black_box("\"Hello world\"")))
    });
}

criterion_group!(
    benches,
    bench_push_str,
    bench_format,
    bench_format_with_capacity
);
criterion_main!(benches);
