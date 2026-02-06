use criterion::{Criterion, criterion_group, criterion_main};
use mock_openai::template;
use std::hint::black_box;

fn bench_render_sse_chunk(c: &mut Criterion) {
    c.bench_function("render_sse_chunk", |b| {
        b.iter(|| template::render_sse_chunk(black_box("Hello world")))
    });
}

fn bench_render_sse_finish(c: &mut Criterion) {
    c.bench_function("render_sse_finish", |b| {
        b.iter(|| template::render_sse_finish(black_box("")))
    });
}

fn bench_render_sse_usage(c: &mut Criterion) {
    c.bench_function("render_sse_usage", |b| {
        b.iter(|| template::render_sse_usage(black_box(100), black_box(100)))
    });
}

fn bench_render_chat_completion(c: &mut Criterion) {
    let content = "FROM fairest creatures we desire increase,";
    c.bench_function("render_chat_completion", |b| {
        b.iter(|| {
            template::render_chat_completion(black_box(content), black_box(100), black_box(100))
        })
    });
}

fn bench_render_chat_completion_large(c: &mut Criterion) {
    let content = "FROM fairest creatures we desire increase,".repeat(100);
    c.bench_function("render_chat_completion_large", |b| {
        b.iter(|| {
            template::render_chat_completion(black_box(&content), black_box(2048), black_box(2048))
        })
    });
}

criterion_group!(
    benches,
    bench_render_sse_chunk,
    bench_render_sse_finish,
    bench_render_sse_usage,
    bench_render_chat_completion,
    bench_render_chat_completion_large
);
criterion_main!(benches);
