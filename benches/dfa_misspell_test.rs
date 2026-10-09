#![allow(deprecated, unused)]
use bamboo_core::{Engine, InputMethod, Mode};

/// Fast smoke for `cargo test` runs (bench mains get no args there).
/// The full suite only runs under `cargo bench` (which passes `--bench`).
fn smoke() {
    println!("(smoke under `cargo test`; run `cargo bench` for the full suite)");
    let mut e = Engine::new(InputMethod::telex());
    for _ in 0..200 {
        e.process_str("xyzabc", Mode::Vietnamese);
        e.process_key(' ', Mode::Vietnamese);
        e.commit();
    }
    println!("smoke misspell x200: dfa_states={}", e.dfa_state_count());
}

fn main() {
    if !std::env::args().any(|a| a == "--bench") {
        smoke();
        return;
    }
    println!("=== DFA growth with MISSPELLED input ===\n");

    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();
    println!("After warm_up: dfa_states={}, arena={}", e.dfa_state_count(), e.dfa_arena_len());

    // Test 1: Repeat same misspelled word 10000 times
    println!("\n--- Test 1: Same misspelled word x10000 ---");
    for i in 0..10000 {
        e.process_str("xyzabc", Mode::Vietnamese);
        e.process_key(' ', Mode::Vietnamese);
        e.commit();
    }
    println!(
        "After 10K same misspell: dfa_states={}, arena={}",
        e.dfa_state_count(),
        e.dfa_arena_len()
    );
    e.reset();

    // Test 2: 10000 UNIQUE misspelled words (worst case)
    println!("\n--- Test 2: 10000 UNIQUE misspelled words ---");
    e.warm_up();
    for i in 0..10000 {
        // Generate unique nonsense: "qwert", "qwery", "qweru", ...
        let w = format!("qwer{}", i);
        e.process_str(&w, Mode::Vietnamese);
        e.process_key(' ', Mode::Vietnamese);
        e.commit();
    }
    println!(
        "After 10K unique misspell: dfa_states={}, arena={}",
        e.dfa_state_count(),
        e.dfa_arena_len()
    );
    e.reset();

    // Test 3: Mixed valid + invalid (realistic misspelling)
    println!("\n--- Test 3: Mixed valid + invalid x10000 ---");
    e.warm_up();
    let valid = ["tieengs", "vietj", "huowng", "khongf", "duowcj"];
    let invalid = ["tieengx", "viezx", "huowq", "khonq", "duowz"];
    for i in 0..10000 {
        if i % 2 == 0 {
            e.process_str(valid[i % valid.len()], Mode::Vietnamese);
        } else {
            e.process_str(invalid[i % invalid.len()], Mode::Vietnamese);
        }
        e.process_key(' ', Mode::Vietnamese);
        e.commit();
    }
    println!("After 10K mixed: dfa_states={}, arena={}", e.dfa_state_count(), e.dfa_arena_len());
    e.reset();

    // Test 4: English misspelling (code-like nonsense)
    println!("\n--- Test 4: English nonsense x10000 ---");
    e.warm_up();
    for i in 0..10000 {
        let w = format!("asdf{}", i);
        e.process_str(&w, Mode::English);
        e.process_key(' ', Mode::English);
        e.commit();
    }
    println!(
        "After 10K english nonsense: dfa_states={}, arena={}",
        e.dfa_state_count(),
        e.dfa_arena_len()
    );

    // Test 5: Vietnamese with random suffix (simulates typo corrections)
    println!("\n--- Test 5: Vietnamese + random suffix x5000 ---");
    e.reset();
    e.warm_up();
    for i in 0..5000 {
        // Type valid word then backspace and retype
        e.process_str("tieengs", Mode::Vietnamese);
        e.remove_last_char(true);
        e.remove_last_char(true);
        e.remove_last_char(true);
        e.process_str("xyz", Mode::Vietnamese);
        e.commit();
    }
    println!(
        "After 5K typo+correct: dfa_states={}, arena={}",
        e.dfa_state_count(),
        e.dfa_arena_len()
    );
}
