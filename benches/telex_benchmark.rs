#![allow(deprecated, unused)]
use bamboo_core::{Engine, InputMethod, Mode};
use std::time::Instant;

const WARMUP_ITERS: usize = 5_000;
const BENCH_ITERS: usize = 200_000;

fn bench_duo<F1, F2>(label: &str, mut bamboo_per_key: F1, mut bamboo_batch: F2)
where
    F1: FnMut(),
    F2: FnMut(),
{
    for _ in 0..WARMUP_ITERS {
        bamboo_per_key();
        bamboo_batch();
    }
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bamboo_per_key();
    }
    let bamboo_pk_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bamboo_batch();
    }
    let bamboo_batch_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;
    println!(
        "{:<40} per-key: {:>8.1} ns | batch: {:>8.1} ns",
        label, bamboo_pk_ns, bamboo_batch_ns
    );
}

fn bench_single<F>(label: &str, mut f: F)
where
    F: FnMut(),
{
    for _ in 0..WARMUP_ITERS {
        f();
    }
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        f();
    }
    let ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;
    println!("{:<40} bamboo: {:>8.1} ns", label, ns);
}

macro_rules! setup_engines {
    () => {{
        let mut bp = Engine::new(InputMethod::telex());
        bp.warm_up();
        let mut bb = Engine::new(InputMethod::telex());
        bb.warm_up();
        (bp, bb)
    }};
}

/// Fast smoke for `cargo test` runs (bench mains get no args there).
/// The full suite only runs under `cargo bench` (which passes `--bench`).
fn smoke() {
    println!("(smoke under `cargo test`; run `cargo bench` for the full suite)");
    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();
    let start = Instant::now();
    for _ in 0..2000 {
        e.reset();
        e.process_str("tieengs", Mode::Vietnamese);
        std::hint::black_box(e.output_str());
    }
    println!("smoke tieengs x2000: {:.1} ns/word", start.elapsed().as_nanos() as f64 / 2000.0);
}

fn main() {
    if !std::env::args().any(|a| a == "--bench") {
        smoke();
        return;
    }
    println!("=== Bamboo-core Telex Benchmark (no skey) ===");

    // Category 1: tones (fresh engine per call = cold/slow path)
    {
        let cases: &[(&str, &str)] =
            &[("vieetj", "việt"), ("tieengs", "tiếng"), ("hoof", "hồ"), ("nguwowif", "người")];
        println!("\n=== Category 1: Tones (cold per-key) ===");
        for &(input, expected) in cases {
            let inp = input.to_string();
            bench_single(&format!("tone {} -> {}", input, expected), || {
                let mut e = Engine::new(InputMethod::telex());
                for c in inp.chars() {
                    e.process_key(c, Mode::Vietnamese);
                }
            });
        }
    }

    // Category 2: common words (warm per-key vs batch)
    {
        let (mut bp, mut bb) = setup_engines!();
        let cases: &[(&str, &str)] = &[
            ("tieengs", "tiếng"),
            ("vietj", "việt"),
            ("huowng", "hương"),
            ("quoocs", "quốc"),
            ("nguwowif", "người"),
            ("nghieengs", "nghiếng"),
            ("khongf", "không"),
            ("duowcj", "được"),
        ];
        println!("\n=== Category 2: Common words (warm) ===");
        for &(input, expected) in cases {
            let inp = input.to_string();
            bench_duo(
                &format!("word {} -> {}", input, expected),
                || {
                    bp.reset();
                    for c in inp.chars() {
                        bp.process_key(c, Mode::Vietnamese);
                    }
                },
                || {
                    bb.reset();
                    bb.process_str(&inp, Mode::Vietnamese);
                },
            );
        }
    }

    // Category 3: sentence
    {
        let (mut bp, mut bb) = setup_engines!();
        let sentences: &[&str] =
            &["hom nay troij depf qua", "toi dang hoc lap trinhr", "tiengf vietj ratj depf"];
        println!("\n=== Category 3: Sentences ===");
        for &input in sentences {
            let inp = input.to_string();
            bench_duo(
                &format!("sentence ({} chars)", input.len()),
                || {
                    bp.reset();
                    for c in inp.chars() {
                        bp.process_key(c, Mode::Vietnamese);
                    }
                },
                || {
                    bb.reset();
                    bb.process_str(&inp, Mode::Vietnamese);
                },
            );
        }
    }

    // Category 4: backspace
    {
        let mut bp = Engine::new(InputMethod::telex());
        bp.warm_up();
        let input = "tieengs";
        println!("\n=== Category 4: Backspace ===");
        bench_single("backspace x1 (tieengs)", || {
            bp.reset();
            bp.process_str(input, Mode::Vietnamese);
            bp.remove_last_char(true);
        });
    }

    // Category 5: delta API
    {
        let mut bp = Engine::new(InputMethod::telex());
        bp.warm_up();
        let keys: Vec<char> = "tieengs".chars().collect();
        println!("\n=== Category 5: Delta API ===");
        bench_single("delta API (tieengs, 7 keys)", || {
            bp.reset();
            for &k in &keys {
                let _ = bp.process_key_delta(k, Mode::Vietnamese);
            }
        });
    }

    // Category 6: english passthrough
    {
        let (mut bp, mut bb) = setup_engines!();
        let inputs = [
            "hello world",
            "fn main() { println!(\"hi\"); }",
            "very_long_variable_name_that_keeps_going",
        ];
        println!("\n=== Category 6: English passthrough ===");
        for &input in &inputs {
            let inp = input.to_string();
            bench_duo(
                &format!("english ({} chars)", input.len()),
                || {
                    bp.reset();
                    for c in inp.chars() {
                        bp.process_key(c, Mode::Vietnamese);
                    }
                },
                || {
                    bb.reset();
                    bb.process_str(&inp, Mode::Vietnamese);
                },
            );
        }
    }

    // Category 7: comprehensive syllables (avg)
    {
        let (mut bp, mut bb) = setup_engines!();
        let syllables: &[&str] = &[
            "a", "af", "as", "ar", "ax", "aj", "e", "ef", "es", "er", "ex", "ej", "aa", "aas",
            "aaf", "aar", "aax", "aaj", "ee", "ees", "eef", "oo", "oos", "aw", "aws", "awf", "ow",
            "ows", "owf", "uw", "uws", "uwf", "dd", "tieengs", "vietj", "huowng", "quoocs",
            "nguwowif", "khongf",
        ];
        println!("\n=== Category 7: Comprehensive syllables (avg) ===");
        let inputs: Vec<String> = syllables.iter().map(|s| s.to_string()).collect();
        for _ in 0..WARMUP_ITERS {
            for inp in &inputs {
                bp.reset();
                for c in inp.chars() {
                    bp.process_key(c, Mode::Vietnamese);
                }
            }
        }
        let start = Instant::now();
        for _ in 0..BENCH_ITERS {
            for inp in &inputs {
                bp.reset();
                for c in inp.chars() {
                    bp.process_key(c, Mode::Vietnamese);
                }
            }
        }
        let pk = start.elapsed().as_nanos() as f64 / (BENCH_ITERS * inputs.len()) as f64;
        for _ in 0..WARMUP_ITERS {
            for inp in &inputs {
                bb.reset();
                bb.process_str(inp, Mode::Vietnamese);
            }
        }
        let start = Instant::now();
        for _ in 0..BENCH_ITERS {
            for inp in &inputs {
                bb.reset();
                bb.process_str(inp, Mode::Vietnamese);
            }
        }
        let batch = start.elapsed().as_nanos() as f64 / (BENCH_ITERS * inputs.len()) as f64;
        println!(
            "{:<40} per-key: {:>8.1} ns | batch: {:>8.1} ns",
            format!("{} syllables avg", inputs.len()),
            pk,
            batch
        );
    }

    println!("\n=== Benchmark Complete ===");
}
