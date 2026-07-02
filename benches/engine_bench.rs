#![allow(deprecated)]
use bamboo_core::{Engine, InputMethod, Mode};
use std::time::Instant;

const WARMUP_ITERS: usize = 10_000;
const BENCH_ITERS: usize = 500_000;

fn bench_dfa_fast_path() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.warm_up();

    let keys: Vec<char> = "tieengs".chars().collect();

    // Warmup
    for _ in 0..WARMUP_ITERS {
        engine.reset();
        for &k in &keys {
            engine.process_key(k, Mode::Vietnamese);
        }
    }

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        engine.reset();
        for &k in &keys {
            engine.process_key(k, Mode::Vietnamese);
        }
    }
    let elapsed = start.elapsed();
    let per_iter_ns = elapsed.as_nanos() as f64 / BENCH_ITERS as f64;
    let per_key_ns = per_iter_ns / keys.len() as f64;
    println!("DFA fast path (tieengs): {:.1} ns/iter ({:.1} ns/key)", per_iter_ns, per_key_ns);
}

fn bench_dfa_miss_path() {
    let mut engine = Engine::new(InputMethod::telex());
    // No warmup - forces DFA misses

    let keys: Vec<char> = "tieengs".chars().collect();

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        engine.reset();
        for &k in &keys {
            engine.process_key(k, Mode::Vietnamese);
        }
    }
    let elapsed = start.elapsed();
    let per_iter_ns = elapsed.as_nanos() as f64 / BENCH_ITERS as f64;
    let per_key_ns = per_iter_ns / keys.len() as f64;
    println!("DFA miss path (tieengs): {:.1} ns/iter ({:.1} ns/key)", per_iter_ns, per_key_ns);
}

fn bench_output() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.warm_up();
    engine.process_str("tieengs", Mode::Vietnamese);

    // Warmup
    for _ in 0..WARMUP_ITERS {
        let _ = engine.output();
    }

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        let _ = engine.output();
    }
    let elapsed = start.elapsed();
    let per_call_ns = elapsed.as_nanos() as f64 / BENCH_ITERS as f64;
    println!("output(): {:.1} ns/call", per_call_ns);
}

fn bench_process_key_delta() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.warm_up();

    let keys: Vec<char> = "tieengs".chars().collect();

    // Warmup
    for _ in 0..WARMUP_ITERS {
        engine.reset();
        for &k in &keys {
            let _ = engine.process_key_delta(k, Mode::Vietnamese);
        }
    }

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        engine.reset();
        for &k in &keys {
            let _ = engine.process_key_delta(k, Mode::Vietnamese);
        }
    }
    let elapsed = start.elapsed();
    let per_iter_ns = elapsed.as_nanos() as f64 / BENCH_ITERS as f64;
    let per_key_ns = per_iter_ns / keys.len() as f64;
    println!("process_key_delta (tieengs): {:.1} ns/iter ({:.1} ns/key)", per_iter_ns, per_key_ns);
}

fn bench_mixed_typing() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.warm_up();

    let words = ["hoc", " ", "ru", "s", "t", " ", "tieengs", " ", "vietj", " ", "tieengs"];
    let keys: Vec<(char, Mode)> = words
        .iter()
        .flat_map(|w| w.chars())
        .map(|c| (c, Mode::Vietnamese))
        .collect();

    // Warmup
    for _ in 0..WARMUP_ITERS {
        engine.reset();
        for &(k, m) in &keys {
            engine.process_key(k, m);
        }
    }

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        engine.reset();
        for &(k, m) in &keys {
            engine.process_key(k, m);
        }
    }
    let elapsed = start.elapsed();
    let per_iter_ns = elapsed.as_nanos() as f64 / BENCH_ITERS as f64;
    let per_key_ns = per_iter_ns / keys.len() as f64;
    println!("mixed_typing: {:.1} ns/iter ({:.1} ns/key)", per_iter_ns, per_key_ns);
}

fn bench_backspace() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.warm_up();

    // Warmup
    for _ in 0..WARMUP_ITERS {
        engine.process_str("tieengs", Mode::Vietnamese);
        engine.remove_last_char(true);
        engine.reset();
    }

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        engine.process_str("tieengs", Mode::Vietnamese);
        engine.remove_last_char(true);
        engine.reset();
    }
    let elapsed = start.elapsed();
    let per_iter_ns = elapsed.as_nanos() as f64 / BENCH_ITERS as f64;
    println!("backspace (tieengs): {:.1} ns/iter", per_iter_ns);
}

fn bench_many_words() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.warm_up();

    let words = [
        "tieengs", "vietj", "huowng", "quoocs", "nguwowif", "viẹtj", "namf",
        "chuyeenn", "thuyeet", "truwowjt", "nghieengs", "hoas", "khuỵu",
    ];

    // Warmup
    for _ in 0..WARMUP_ITERS {
        engine.reset();
        for w in &words {
            engine.process_str(w, Mode::Vietnamese);
            engine.process_key(' ', Mode::Vietnamese);
        }
    }

    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        engine.reset();
        for w in &words {
            engine.process_str(w, Mode::Vietnamese);
            engine.process_key(' ', Mode::Vietnamese);
        }
    }
    let elapsed = start.elapsed();
    let per_iter_ns = elapsed.as_nanos() as f64 / BENCH_ITERS as f64;
    println!("many_words (13 words): {:.1} ns/iter", per_iter_ns);
}

fn main() {
    println!("=== Bamboo Core Benchmark ===");
    println!("Iterations: {}", BENCH_ITERS);
    println!();
    bench_dfa_fast_path();
    bench_dfa_miss_path();
    bench_output();
    bench_process_key_delta();
    bench_mixed_typing();
    bench_backspace();
    bench_many_words();
}
