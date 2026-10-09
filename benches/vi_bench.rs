#![allow(deprecated)]
use bamboo_core::{Engine, InputMethod, Mode};
use skey_engine::{Method, SkeyEngine};
use std::time::Instant;
use uvie::UltraFastViEngine;

const WARMUP_ITERS: usize = 5_000;
const BENCH_ITERS: usize = 100_000;

fn bench_all_4<F1, F2, F3, F4>(
    label: &str,
    mut bamboo_fn: F1,
    mut skey_fn: F2,
    mut uvie_fn: F3,
    mut vi_fn: F4,
) where
    F1: FnMut(),
    F2: FnMut(),
    F3: FnMut(),
    F4: FnMut(),
{
    // Warmup
    for _ in 0..WARMUP_ITERS {
        bamboo_fn();
        skey_fn();
        uvie_fn();
        vi_fn();
    }

    // Bamboo
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        bamboo_fn();
    }
    let bamboo_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    // Skey
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        skey_fn();
    }
    let skey_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    // Uvie
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        uvie_fn();
    }
    let uvie_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    // Vi
    let start = Instant::now();
    for _ in 0..BENCH_ITERS {
        vi_fn();
    }
    let vi_ns = start.elapsed().as_nanos() as f64 / BENCH_ITERS as f64;

    let vs_skey = if skey_ns > 0.0 { bamboo_ns / skey_ns } else { 0.0 };
    let vs_uvie = if uvie_ns > 0.0 { bamboo_ns / uvie_ns } else { 0.0 };
    let vs_vi = if vi_ns > 0.0 { bamboo_ns / vi_ns } else { 0.0 };

    println!(
        "{:<42} | Bamboo: {:>6.1} ns | Skey: {:>6.1} ns | Uvie: {:>7.1} ns | Vi: {:>7.1} ns | vs Skey: {:>4.2}x | vs Uvie: {:>4.2}x | vs Vi: {:>4.2}x",
        label, bamboo_ns, skey_ns, uvie_ns, vi_ns, vs_skey, vs_uvie, vs_vi
    );
}

/// Fast smoke for `cargo test` runs (bench mains get no args there).
/// The full suite only runs under `cargo bench` (which passes `--bench`).
fn smoke() {
    println!("(smoke under `cargo test`; run `cargo bench` for the full suite)");
    let mut bamboo = Engine::new(InputMethod::telex());
    bamboo.warm_up();
    let keys: Vec<char> = "tieengs".chars().collect();
    let start = Instant::now();
    for _ in 0..2000 {
        bamboo.reset();
        for &k in &keys {
            bamboo.process_key(k, Mode::Vietnamese);
        }
        std::hint::black_box(bamboo.output_str());
    }
    println!(
        "smoke feed tieengs x2000: {:.1} ns/key",
        start.elapsed().as_nanos() as f64 / 2000.0 / keys.len() as f64
    );
}

fn main() {
    if !std::env::args().any(|a| a == "--bench") {
        smoke();
        return;
    }
    println!(
        "========================================================================================================================"
    );
    println!("              FULL 4-WAY REAL-TIME KEYSTROKE & IME BENCHMARK SUITE (2026)");
    println!(
        "              Bamboo Core (v0.3.26) | Skey-Engine (v0.1.23) | Uvie (v2.7.0) | Vi (v0.8.0)"
    );
    println!(
        "========================================================================================================================"
    );
    println!("Iterations: {}\n", BENCH_ITERS);

    let skey = SkeyEngine::new(Method::Telex);

    // =========================================================================
    // SECTION 1: SINGLE WORD STREAM (Per-keystroke interactive typing)
    // =========================================================================
    println!("--- [1. SINGLE WORD KEYSTROKE STREAMS] ---");

    // 1. tieengs -> tiếng
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "tieengs";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Word 'tieengs' -> 'tiếng'",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 2. vietj -> việt
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "vietj";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Word 'vietj' -> 'việt'",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 3. nguwowif -> người
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "nguwowif";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Word 'nguwowif' -> 'người'",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 4. dduwowngf -> đường
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "dduwowngf";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Word 'dduwowngf' -> 'đường'",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 5. khuyeens -> khuyến
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "khuyeens";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Word 'khuyeens' -> 'khuyến'",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    println!();

    // =========================================================================
    // SECTION 2: BACKSPACE, UNDO & RETYPE (Keystroke recovery)
    // =========================================================================
    println!("--- [2. INTERACTIVE BACKSPACE & RECOVERY] ---");

    // 1. Single word BS x2
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "tieengs";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Type 'tieengs' + Backspace x2",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
                bamboo.remove_last_char(true);
                bamboo.remove_last_char(true);
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
                let _ = skey.transform(&raw[..5]);
                let _ = skey.transform(&raw[..4]);
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
                uvie.backspace();
                uvie.backspace();
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys[..5].iter().copied(), &mut vi_buf);
            },
        );
    }

    // 2. Heavy word BS x5: 'thuyeens' + 5x BS
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "thuyeens";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Type 'thuyeens' + Backspace x5",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
                for _ in 0..5 {
                    bamboo.remove_last_char(true);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
                for i in (3..=7).rev() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
                for _ in 0..5 {
                    uvie.backspace();
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys[..3].iter().copied(), &mut vi_buf);
            },
        );
    }

    // 3. Sentence + Backspace 1 word: "Tooi laf nguwoif Vieetj Nam" + 4 BS (" Nam")
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(64);
        let raw = "Tooi laf nguwoif Vieetj Nam";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Sentence (27ch) + Backspace 4ch (' Nam')",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
                for _ in 0..4 {
                    bamboo.remove_last_char(true);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
                for i in (23..=26).rev() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
                for _ in 0..4 {
                    uvie.backspace();
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys[..23].iter().copied(), &mut vi_buf);
            },
        );
    }

    // 4. Sentence + Backspace halfway: "Coong hoaf xax hooj" + 9 BS (" xax hooj")
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(64);
        let raw = "Coong hoaf xax hooj";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Sentence (19ch) + Backspace 9ch halfway",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
                for _ in 0..9 {
                    bamboo.remove_last_char(true);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
                for i in (10..=18).rev() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
                for _ in 0..9 {
                    uvie.backspace();
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys[..10].iter().copied(), &mut vi_buf);
            },
        );
    }

    // 5. Delete entire word & retype next word
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let k1: Vec<char> = "tieengs".chars().collect();
        let k2: Vec<char> = "vieetj".chars().collect();

        bench_all_4(
            "Full delete: 'tieengs' + Del*7 + 'vieetj'",
            || {
                bamboo.reset();
                for &k in &k1 {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
                for _ in 0..7 {
                    bamboo.remove_last_char(true);
                }
                for &k in &k2 {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=7 {
                    let _ = skey.transform(&"tieengs"[..i]);
                }
                for i in (0..=6).rev() {
                    let _ = skey.transform(&"tieengs"[..i]);
                }
                for i in 1..=6 {
                    let _ = skey.transform(&"vieetj"[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &k1 {
                    uvie.feed(k);
                }
                for _ in 0..7 {
                    uvie.backspace();
                }
                for &k in &k2 {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, k2.iter().copied(), &mut vi_buf);
            },
        );
    }

    println!();

    // =========================================================================
    // SECTION 3: SENTENCES & LONG PARAGRAPHS (Per-keystroke stream)
    // =========================================================================
    println!("--- [3. SENTENCES & PARAGRAPH KEYSTROKE STREAMS] ---");

    // 1. Short sentence stream (23 chars)
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(64);
        let raw = "Tooi laf nguwoif Vieetj";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Sentence stream (23 chars)",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 2. Full sentence stream (27 chars)
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(64);
        let raw = "Tooi laf nguwoif Vieetj Nam";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Full sentence stream (27 chars)",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 3. Long text stream (13 words, 62 chars)
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(128);
        let raw = "Coong hoaf xax hooj chuwr nghiax Vieetj Nam docj laapj tuwj do";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Long text stream (13 words, 62 chars)",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    println!();

    // =========================================================================
    // SECTION 4: UPPERCASE, CAMELCASE & CODE PASSTHROUGH
    // =========================================================================
    println!("--- [4. UPPERCASE, CAMELCASE & ENGLISH PASSTHROUGH] ---");

    // 1. Uppercase: TIEENGS -> TIẾNG
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "TIEENGS";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Uppercase stream 'TIEENGS' -> 'TIẾNG'",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 2. CamelCase: VieetjNam -> ViệtNam
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(32);
        let raw = "VieetjNam";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "CamelCase stream 'VieetjNam' -> 'ViệtNam'",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 3. English Identifier Passthrough (31 chars)
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(64);
        let raw = "bamboo_engine_process_key_delta";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "English Identifier (31 chars passthrough)",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    // 4. Mixed Code Statement (30 chars)
    {
        let mut bamboo = Engine::new(InputMethod::telex());
        let mut uvie = UltraFastViEngine::new();
        let mut vi_buf = String::with_capacity(64);
        let raw = "let mut buf = String::new();";
        let keys: Vec<char> = raw.chars().collect();

        bench_all_4(
            "Mixed code line: 'let mut buf = ...'",
            || {
                bamboo.reset();
                for &k in &keys {
                    bamboo.process_key(k, Mode::Vietnamese);
                }
            },
            || {
                for i in 1..=raw.len() {
                    let _ = skey.transform(&raw[..i]);
                }
            },
            || {
                uvie.clear();
                for &k in &keys {
                    uvie.feed(k);
                }
            },
            || {
                vi_buf.clear();
                vi::transform_buffer(&vi::TELEX, keys.iter().copied(), &mut vi_buf);
            },
        );
    }

    println!("\n=== Benchmark Suite Finished Successfully ===");
}
