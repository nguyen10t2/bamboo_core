#![allow(deprecated)]
use bamboo_core::{Engine, InputMethod, Mode};
use std::time::Instant;
use uvie::UltraFastViEngine;

const WARMUP_ITERS: usize = 10_000;
const BENCH_ITERS: usize = 500_000;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn bench_pair<F1, F2>(label: &str, mut bamboo_fn: F1, mut uvie_fn: F2)
where
    F1: FnMut(),
    F2: FnMut(),
{
    // Warmup
    for _ in 0..WARMUP_ITERS {
        bamboo_fn();
        uvie_fn();
    }

    // Bamboo
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bamboo_fn();
    }
    let bamboo_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    // Uvie
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        uvie_fn();
    }
    let uvie_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    let ratio = if uvie_ns > 0.0 { bamboo_ns / uvie_ns } else { 0.0 };
    println!(
        "{:<30} bamboo: {:>8.1} ns | uvie: {:>8.1} ns | ratio: {:.2}x",
        label, bamboo_ns, uvie_ns, ratio
    );
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_feed_single_word() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    let keys: Vec<char> = "tieengs".chars().collect();

    bench_pair(
        "feed single word (tieengs)",
        || {
            bamboo.reset();
            for &k in &keys {
                bamboo.process_key(k, Mode::Vietnamese);
            }
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
        },
    );
}

fn bench_feed_no_warmup() {
    let keys: Vec<char> = "tieengs".chars().collect();

    bench_pair(
        "feed no warmup (DFA miss)",
        || {
            let mut e = Engine::new(InputMethod::telex());
            for &k in &keys {
                e.process_key(k, Mode::Vietnamese);
            }
        },
        || {
            let mut e = UltraFastViEngine::new();
            for &k in &keys {
                e.feed(k);
            }
        },
    );
}

fn bench_output() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();

    let mut uvie = UltraFastViEngine::new();

    let keys: Vec<char> = "tieengs".chars().collect();

    bench_pair(
        "feed + delta (per key)",
        || {
            bamboo.reset();
            for &k in &keys {
                let _ = bamboo.process_key_delta(k, Mode::Vietnamese);
            }
        },
        || {
            uvie.clear();
            for &k in &keys {
                let _ = uvie.feed(k);
            }
        },
    );
}

fn bench_backspace() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    let keys: Vec<char> = "tieengs".chars().collect();

    bench_pair(
        "backspace (tieengs)",
        || {
            bamboo.process_str("tieengs", Mode::Vietnamese);
            bamboo.remove_last_char(true);
            bamboo.reset();
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
            uvie.backspace();
        },
    );
}

fn bench_backspace_3x() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    let keys: Vec<char> = "tieengs".chars().collect();

    bench_pair(
        "backspace x3 (tieengs)",
        || {
            bamboo.process_str("tieengs", Mode::Vietnamese);
            bamboo.remove_last_char(true);
            bamboo.remove_last_char(true);
            bamboo.remove_last_char(true);
            bamboo.reset();
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
            uvie.backspace();
            uvie.backspace();
            uvie.backspace();
        },
    );
}

fn bench_mixed_typing() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    let keys: Vec<char> = "hoc rust tieengs vietj tieengs".chars().collect();

    bench_pair(
        "mixed typing (29 chars)",
        || {
            bamboo.reset();
            for &k in &keys {
                bamboo.process_key(k, Mode::Vietnamese);
            }
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
        },
    );
}

fn bench_many_words() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    let words = [
        "tieengs",
        "vietj",
        "huowng",
        "quoocs",
        "nguwowif",
        "viẹtj",
        "namf",
        "chuyeenn",
        "thuyeet",
        "truwowjt",
        "nghieengs",
        "hoas",
        "khuỵu",
    ];

    bench_pair(
        "many words (13 words + spaces)",
        || {
            bamboo.reset();
            for w in &words {
                bamboo.process_str(w, Mode::Vietnamese);
                bamboo.process_key(' ', Mode::Vietnamese);
            }
        },
        || {
            uvie.clear();
            for w in &words {
                for c in w.chars() {
                    uvie.feed(c);
                }
                uvie.feed(' ');
            }
        },
    );
}

// ---------------------------------------------------------------------------
// New benchmarks: real-world scenarios
// ---------------------------------------------------------------------------

fn bench_english_passthrough() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    // Simulates typing code: "let mut count = 0;"
    let keys: Vec<char> = "let mut count = 0;".chars().collect();

    bench_pair(
        "english passthrough (code)",
        || {
            bamboo.reset();
            for &k in &keys {
                bamboo.process_key(k, Mode::English);
            }
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
        },
    );
}

fn bench_long_identifier() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    // Typical Rust long identifier
    let keys: Vec<char> = "very_long_variable_name_that_keeps_going".chars().collect();

    bench_pair(
        "long identifier (39 chars)",
        || {
            bamboo.reset();
            for &k in &keys {
                bamboo.process_key(k, Mode::English);
            }
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
        },
    );
}

fn bench_backspace_spam() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    let keys: Vec<char> = "tieengs".chars().collect();

    // Type 7 chars, then backspace all 7 (simulates holding Backspace)
    bench_pair(
        "backspace spam (7x on tieengs)",
        || {
            bamboo.process_str("tieengs", Mode::Vietnamese);
            for _ in 0..7 {
                bamboo.remove_last_char(true);
            }
            bamboo.reset();
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
            for _ in 0..7 {
                uvie.backspace();
            }
        },
    );
}

fn bench_commit_latency() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    // Type "tieng" then space (commit)
    let keys: Vec<char> = "tieng".chars().collect();

    bench_pair(
        "commit via space (tieng + ' ')",
        || {
            bamboo.reset();
            for &k in &keys {
                bamboo.process_key(k, Mode::Vietnamese);
            }
            bamboo.process_key(' ', Mode::Vietnamese);
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
            uvie.feed(' ');
        },
    );
}

fn bench_worst_case_syllable() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    // "nghieengs" = nghiếng (9 chars, complex transformations)
    let keys: Vec<char> = "nghieengs".chars().collect();

    bench_pair(
        "worst-case syllable (nghieengs)",
        || {
            bamboo.reset();
            for &k in &keys {
                bamboo.process_key(k, Mode::Vietnamese);
            }
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
        },
    );
}

fn bench_random_typing() {
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let mut uvie = UltraFastViEngine::new();

    // Realistic paragraph: Vietnamese mixed with English, spaces, punctuation
    let text = "hom nay toi hoc rust, viet ham process_data() rat hay! \
                chuuyen nayf khoownf ddeeer nhees.";
    let keys: Vec<char> = text.chars().collect();

    bench_pair(
        "random typing (real workload)",
        || {
            bamboo.reset();
            for &k in &keys {
                bamboo.process_key(k, Mode::Vietnamese);
            }
        },
        || {
            uvie.clear();
            for &k in &keys {
                uvie.feed(k);
            }
        },
    );
}

fn main() {
    println!("=== Bamboo Core vs Uvie Benchmark ===");
    println!("Iterations: {}\n", BENCH_ITERS);
    println!("{:<35} {:>16} {:>16} {:>10}", "Benchmark", "Bamboo", "Uvie", "Ratio");
    println!("{}", "-".repeat(80));

    bench_feed_single_word();
    bench_feed_no_warmup();
    bench_output();
    bench_backspace();
    bench_backspace_3x();
    bench_mixed_typing();
    bench_many_words();

    println!("\n--- Real-world scenarios ---\n");

    bench_english_passthrough();
    bench_long_identifier();
    bench_backspace_spam();
    bench_commit_latency();
    bench_worst_case_syllable();
    bench_random_typing();
}
