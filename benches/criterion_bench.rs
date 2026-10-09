//! Criterion micro/macro benchmarks for bamboo-core.
//!
//! Run with `cargo bench --bench criterion_bench`.
//! Reports are written to `target/criterion/`.

use bamboo_core::{Engine, InputMethodPreset, Mode, RestoreMark};
use criterion::{BenchmarkId, Criterion, criterion_group};
use std::hint::black_box;

fn fresh_engine() -> Engine {
    Engine::from_preset(InputMethodPreset::Telex)
}

fn warm_engine(keys: &str) -> Engine {
    let mut e = fresh_engine();
    e.process_str(keys, Mode::Vietnamese);
    e
}

fn bench_dfa_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("dfa_hit");
    for word in ["tieengs", "vietj", "nguwowif", "dduwowngf", "khuyeens"] {
        group.bench_with_input(BenchmarkId::new("keystroke_stream", word), word, |b, w| {
            let mut e = warm_engine("");
            b.iter(|| {
                for ch in w.chars() {
                    e.process_key(black_box(ch), Mode::Vietnamese);
                }
                black_box(e.output_str());
            });
        });
    }
    group.finish();
}

fn bench_dfa_miss_slow(c: &mut Criterion) {
    let mut group = c.benchmark_group("dfa_miss_slow");
    // Fresh engine each iteration = DFA miss / rule slow path.
    group.bench_function("cold_tone_word", |b| {
        b.iter_custom(|iters| {
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let mut e = fresh_engine();
                for ch in "nghieengs".chars() {
                    e.process_key(black_box(ch), Mode::Vietnamese);
                }
                black_box(e.output_str());
            }
            start.elapsed()
        });
    });
    group.finish();
}

fn bench_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("output");
    let e = warm_engine("tieengs");
    group.bench_function("output_str", |b| {
        b.iter(|| black_box(e.output_str()));
    });
    group.bench_function("output_cow", |b| {
        b.iter(|| black_box(e.output()));
    });
    group.finish();
}

fn bench_delta(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta");
    group.bench_function("process_key_delta_per_key", |b| {
        let mut e = fresh_engine();
        b.iter(|| {
            for ch in "vietj".chars() {
                let d = e.process_key_delta(black_box(ch), Mode::Vietnamese);
                black_box(d);
            }
        });
    });
    group.finish();
}

fn bench_backspace(c: &mut Criterion) {
    let mut group = c.benchmark_group("backspace");
    group.bench_function("remove_last_char_x1", |b| {
        let mut e = warm_engine("tieengs");
        b.iter(|| {
            e.process_str("tieengs", Mode::Vietnamese);
            e.remove_last_char(RestoreMark::Yes);
            black_box(e.output_str());
        });
    });
    group.bench_function("remove_last_char_x3", |b| {
        let mut e = warm_engine("tieengs");
        b.iter(|| {
            e.process_str("tieengs", Mode::Vietnamese);
            for _ in 0..3 {
                e.remove_last_char(RestoreMark::Yes);
            }
            black_box(e.output_str());
        });
    });
    group.finish();
}

fn bench_english(c: &mut Criterion) {
    let mut group = c.benchmark_group("english");
    group.bench_function("passthrough", |b| {
        let mut e = fresh_engine();
        b.iter(|| {
            for ch in "getUserNameByIdentifier".chars() {
                e.process_key(black_box(ch), Mode::English);
            }
            black_box(e.output_str());
        });
    });
    group.finish();
}

fn bench_sentence(c: &mut Criterion) {
    let mut group = c.benchmark_group("sentence");
    group.bench_function("27_chars", |b| {
        let mut e = fresh_engine();
        b.iter(|| {
            for ch in "tooi ddax tieengs Vieejt Nam".chars() {
                e.process_key(black_box(ch), Mode::Vietnamese);
            }
            black_box(e.output_str());
        });
    });
    group.bench_function("62_chars", |b| {
        let mut e = fresh_engine();
        b.iter(|| {
            for ch in "Chufng ta ddaang soongs treen quoocs gia Vieejt Nam xinh depf vaaf tuwj doj"
                .chars()
            {
                e.process_key(black_box(ch), Mode::Vietnamese);
            }
            black_box(e.output_str());
        });
    });
    group.finish();
}

fn bench_commit(c: &mut Criterion) {
    let mut group = c.benchmark_group("commit");
    group.bench_function("commit_word", |b| {
        let mut e = fresh_engine();
        b.iter(|| {
            e.process_str("tieengs", Mode::Vietnamese);
            e.commit();
            black_box(e.output_str());
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_dfa_hit,
    bench_dfa_miss_slow,
    bench_output,
    bench_delta,
    bench_backspace,
    bench_english,
    bench_sentence,
    bench_commit
);

fn main() {
    // `cargo test --all-targets` executes bench mains with no args: run a
    // fast smoke instead of the full criterion suite (`cargo bench` only,
    // which passes `--bench`).
    if !std::env::args().any(|a| a == "--bench") {
        println!("(smoke under `cargo test`; run `cargo bench` for the full criterion suite)");
        return;
    }
    benches();
}
