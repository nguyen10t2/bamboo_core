//! RAM & memory profiling benchmark for bamboo-core.
//!
//! Measures:
//! - Struct size (stack layout)
//! - Heap growth during long typing sessions
//! - DFA memory growth
//! - committed_text growth
//! - Allocations per reset cycle

#![allow(deprecated, unused)]
use bamboo_core::{Engine, InputMethod, Mode, OutputOptions};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// ---------------------------------------------------------------------------
// Custom allocator to track total allocated bytes
// ---------------------------------------------------------------------------

struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn reset_counters() {
    ALLOCATED.store(0, Ordering::Relaxed);
    DEALLOCATED.store(0, Ordering::Relaxed);
    ALLOC_COUNT.store(0, Ordering::Relaxed);
}

fn snapshot() -> (usize, usize, isize) {
    let a = ALLOCATED.load(Ordering::Relaxed);
    let d = DEALLOCATED.load(Ordering::Relaxed);
    let count = ALLOC_COUNT.load(Ordering::Relaxed);
    (a, d, count as isize)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_bytes(b: usize) -> String {
    if b < 1024 {
        format!("{} B", b)
    } else if b < 1024 * 1024 {
        format!("{:.1} KB", b as f64 / 1024.0)
    } else {
        format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
    }
}

// ===========================================================================
// Test 1: Struct sizes (stack layout)
// ===========================================================================

fn test_struct_sizes() {
    println!("=== 1. Struct sizes (stack layout) ===");
    println!(
        "  Engine size:           {:>8} bytes ({:.1} KB)",
        std::mem::size_of::<Engine>(),
        std::mem::size_of::<Engine>() as f64 / 1024.0
    );
    println!(
        "  Transformation size:   {:>8} bytes",
        std::mem::size_of::<bamboo_core::Transformation>()
    );
    println!(
        "  TransformationStack:   {:>8} bytes ({:.1} KB)",
        std::mem::size_of::<bamboo_core::TransformationStack>(),
        std::mem::size_of::<bamboo_core::TransformationStack>() as f64 / 1024.0
    );
    println!();
}

// ===========================================================================
// Test 2: Engine::new() allocation cost
// ===========================================================================

fn test_engine_new_cost() {
    println!("=== 2. Engine::new() allocation cost ===");
    reset_counters();
    let before = snapshot();
    let e = Engine::new(InputMethod::telex());
    let after = snapshot();
    let net_alloc = after.0 as isize - before.0 as isize;
    let net_count = after.2 - before.2;
    println!(
        "  Engine::new():         {:>8} bytes allocated, {} allocations",
        format_bytes(net_alloc as usize),
        net_count
    );
    drop(e);
    println!();
}

// ===========================================================================
// Test 3: warm_up() cost
// ===========================================================================

fn test_warm_up_cost() {
    println!("=== 3. warm_up() cost ===");
    reset_counters();
    let before = snapshot();
    let mut e = Engine::new(InputMethod::telex());
    let after_new = snapshot();
    e.warm_up();
    let after_warmup = snapshot();

    let new_bytes = after_new.0 as isize - before.0 as isize;
    let warmup_bytes = after_warmup.0 as isize - after_new.0 as isize;
    let warmup_count = after_warmup.2 - after_new.2;

    println!(
        "  Engine::new():         {:>8} bytes, {} allocs",
        format_bytes(new_bytes as usize),
        after_new.2 - before.2
    );
    println!(
        "  warm_up():             {:>8} bytes, {} allocs",
        format_bytes(warmup_bytes as usize),
        warmup_count
    );
    println!(
        "  Total after warm_up:   {:>8} bytes",
        format_bytes((new_bytes + warmup_bytes) as usize)
    );

    let dfa_states = e.dfa_state_count();
    let dfa_arena = e.dfa_arena_len();
    println!("  DFA states after warm_up: {}", dfa_states);
    println!("  DFA arena (Transformations): {}", dfa_arena);
    println!(
        "  DFA arena memory:      {:>8}",
        format_bytes(dfa_arena * std::mem::size_of::<bamboo_core::Transformation>())
    );
    drop(e);
    println!();
}

// ===========================================================================
// Test 4: RAM growth during long typing sessions
// ===========================================================================

fn test_ram_growth() {
    println!("=== 4. RAM growth during long typing sessions ===");
    println!(
        "{:<20} {:>12} {:>12} {:>12} {:>12}",
        "Scenario", "Alloc'd", "Net RAM", "Allocs", "DFA states"
    );
    println!("{}", "-".repeat(72));

    let words = [
        "tieengs",
        "vietj",
        "huowng",
        "quoocs",
        "nguwowif",
        "namf",
        "chuyeenn",
        "thuyeet",
        "truwowjt",
        "nghieengs",
        "hoas",
        "khongf",
        "duowcj",
        "nhuwngf",
        "moiw",
        "laaj",
        "tuoif",
        "troiwf",
        "doocs",
        "muaws",
    ];

    for &word_count in &[10usize, 100, 1000, 5000, 10000] {
        reset_counters();
        let mut e = Engine::new(InputMethod::telex());
        e.warm_up();
        let after_warmup = snapshot();

        for i in 0..word_count {
            let w = words[i % words.len()];
            e.process_str(w, Mode::Vietnamese);
            e.process_key(' ', Mode::Vietnamese);
            if i % 5 == 0 {
                e.commit();
            }
        }
        e.commit();

        let after_typing = snapshot();
        let typing_bytes = after_typing.0 as isize - after_warmup.0 as isize;
        let typing_count = after_typing.2 - after_warmup.2;
        let dfa_states = e.dfa_state_count();

        let text = e.get_processed_str(OutputOptions::FULL_TEXT);

        println!(
            "{:<20} {:>12} {:>12} {:>12} {:>12}",
            format!("{} words", word_count),
            format_bytes(typing_bytes as usize),
            format_bytes(text.len()),
            typing_count,
            dfa_states
        );
    }
    println!();
}

// ===========================================================================
// Test 5: DFA memory growth tracking
// ===========================================================================

fn test_dfa_growth() {
    println!("=== 5. DFA memory growth during typing ===");
    println!(
        "{:<15} {:>12} {:>15} {:>15}",
        "After N keys", "DFA states", "DFA arena (B)", "composition_map"
    );
    println!("{}", "-".repeat(60));

    let mut e = Engine::new(InputMethod::telex());

    let input = "tieengs vietj huowng quoocs nguwowif namf chuyeenn thuyeet truwowjt nghieengs hoas khongf duowcj nhuwngf moiw laaj tuoif troiwf doocs muaws";
    let keys: Vec<char> = input.chars().collect();

    for (i, &k) in keys.iter().enumerate() {
        e.process_key(k, Mode::Vietnamese);

        if [0, 6, 13, 21, 29, 38, 46, 55, 65, 76, 84, 92, 100, 110, 118, 124, 131, 139, 147, 155]
            .contains(&i)
        {
            let states = e.dfa_state_count();
            let arena = e.dfa_arena_len();
            let arena_bytes = arena * std::mem::size_of::<bamboo_core::Transformation>();
            let map_entries = e.dfa_composition_count();

            println!(
                "{:<15} {:>12} {:>15} {:>15}",
                i + 1,
                states,
                format_bytes(arena_bytes),
                map_entries
            );
        }
    }
    println!();
}

// ===========================================================================
// Test 6: Reset cycle — does RAM leak?
// ===========================================================================

fn test_reset_leak() {
    println!("=== 6. Reset cycle — memory leak check ===");

    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();

    reset_counters();
    let before = snapshot();

    for _ in 0..1000 {
        e.process_str("tieengs vietj huowng quoocs nguwowif", Mode::Vietnamese);
        e.commit();
        e.reset();
    }

    let after = snapshot();
    let leaked = after.0 as isize - before.0 as isize;
    let alloc_count = after.2 - before.2;

    println!(
        "  1000 reset cycles:     {:>8} net allocated, {} allocs",
        format_bytes(leaked.max(0) as usize),
        alloc_count
    );
    println!("  DFA states after:      {}", e.dfa_state_count());
    println!("  committed_text cap:    {} bytes", e.committed_text_capacity());
    println!();
}

// ===========================================================================
// Test 7: Uppercase DFA bypass analysis
// ===========================================================================

fn test_uppercase_dfa_bypass() {
    println!("=== 7. Uppercase DFA bypass analysis ===");
    println!("  DFA fast path condition: is_ascii && !is_upper_case");
    println!("  → ALL uppercase keys hit the slow path (rule engine)");
    println!();

    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();

    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        e.reset();
        for c in "tieengs".chars() {
            e.process_key(c, Mode::Vietnamese);
        }
    }
    let lower_ns = start.elapsed().as_nanos() as f64 / 100_000.0;

    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        e.reset();
        for c in "TIEENGS".chars() {
            e.process_key(c, Mode::Vietnamese);
        }
    }
    let upper_ns = start.elapsed().as_nanos() as f64 / 100_000.0;

    println!("  lowercase 'tieengs':   {:>8.1} ns (DFA hit)", lower_ns);
    println!("  uppercase 'TIEENGS':   {:>8.1} ns (DFA miss, slow path)", upper_ns);
    println!("  ratio:                 {:>8.1}x slower", upper_ns / lower_ns);
    println!();

    e.reset();
    e.process_key('a', Mode::Vietnamese);
    e.reset();

    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        e.reset();
        e.process_key('a', Mode::Vietnamese);
    }
    let a_ns = start.elapsed().as_nanos() as f64 / 100_000.0;

    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        e.reset();
        e.process_key('A', Mode::Vietnamese);
    }
    let a_up_ns = start.elapsed().as_nanos() as f64 / 100_000.0;

    println!("  single 'a':            {:>8.1} ns (DFA hit)", a_ns);
    println!("  single 'A':            {:>8.1} ns (DFA miss)", a_up_ns);
    println!("  ratio:                 {:>8.1}x slower", a_up_ns / a_ns);
    println!();
}

// ===========================================================================
// Test 8: Realistic long session
// ===========================================================================

fn test_realistic_session() {
    println!("=== 8. Realistic long session (100 sentences) ===");

    let sentences = [
        "hom nay troij depf qua, toi di choiwj voiwf banj bef.",
        "tiengf vietj ratj depf va phuwf tapj, canf phaij hocj nhieeuf.",
        "chao mowf banf ddenf vieetj namf, quoocs depf lamf.",
        "tuoif treo hom nayf hocj ratj chams va nghifeng tueej.",
        "duownjg nhuw laajf nguwowif nha dangf nauw anwj.",
        "buwows saungf hom nayf troijf ratj depf, khongf cooj muaaj.",
        "anhf ayf laf mojt nguwowif totj buojngf, luoonf giuwp doj moiwf nguwowif.",
        "truwowjt khi di nguwr, toi thuwowngf doocj sajspj motf cuoons sajspj.",
        "nguwowif vietj namf tuowngf tuwf ratj caof, khongf soj gianj nanf.",
        "moij ngayf toi ddayf tuowfj lucj 6 gioof sangf deej tapj theer ducj.",
    ];

    reset_counters();
    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();
    let after_warmup = snapshot();

    for i in 0..100 {
        let s = sentences[i % sentences.len()];
        e.process_str(s, Mode::Vietnamese);
        e.process_key('\n', Mode::Vietnamese);
        e.commit();
    }

    let after = snapshot();
    let typing_bytes = after.0 as isize - after_warmup.0 as isize;
    let typing_count = after.2 - after_warmup.2;
    let dfa_states = e.dfa_state_count();

    let text = e.get_processed_str(OutputOptions::FULL_TEXT);

    println!(
        "  100 sentences typed:   {:>8} allocated, {} allocs",
        format_bytes(typing_bytes.max(0) as usize),
        typing_count
    );
    println!("  DFA states:            {}", dfa_states);
    println!("  Output text length:    {} bytes", text.len());
    println!();
}

// ===========================================================================
// Test 9: EXTREME — 10,000 Vietnamese sentences
// ===========================================================================

fn test_extreme_vietnamese() {
    println!("=== 9. EXTREME Vietnamese — 10,000 sentences ===");

    let sentences = [
        "hom nay troij depf qua, toi di choiwj voiwf banj bef.",
        "tiengf vietj ratj depf va phuwf tapj, canf phaij hocj nhieeuf.",
        "chao mowf banf ddenf vieetj namf, quoocs depf lamf.",
        "tuoif treo hom nayf hocj ratj chams va nghifeng tueej.",
        "duownjg nhuw laajf nguwowif nha dangf nauw anwj.",
        "buwows saungf hom nayf troijf ratj depf, khongf cooj muaaj.",
        "anhf ayf laf mojt nguwowif totj buojngf, luoonf giuwp doj moiwf nguwowif.",
        "truwowjt khi di nguwr, toi thuwowngf doocj sajspj motf cuoons sajspj.",
        "nguwowif vietj namf tuowngf tuwf ratj caof, khongf soj gianj nanf.",
        "moij ngayf toi ddayf tuowfj lucj 6 gioof sangf deej tapj theer ducj.",
    ];

    reset_counters();
    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();
    let after_warmup = snapshot();

    let checkpoints = [100, 500, 1000, 2000, 5000, 10000];
    println!(
        "{:<12} {:>12} {:>12} {:>12} {:>12} {:>15}",
        "Sentences", "Heap alloc", "Net RAM", "DFA states", "DFA arena", "committed_text"
    );
    println!("{}", "-".repeat(80));

    for i in 0..10000 {
        let s = sentences[i % sentences.len()];
        e.process_str(s, Mode::Vietnamese);
        e.process_key(' ', Mode::Vietnamese);
        e.commit();

        if checkpoints.contains(&(i + 1)) {
            let now = snapshot();
            let heap = now.0 as isize - after_warmup.0 as isize;
            let text = e.get_processed_str(OutputOptions::FULL_TEXT);
            println!(
                "{:<12} {:>12} {:>12} {:>12} {:>12} {:>15}",
                i + 1,
                format_bytes(heap.max(0) as usize),
                format_bytes(text.len()),
                e.dfa_state_count(),
                e.dfa_arena_len(),
                format_bytes(e.committed_text_capacity())
            );
        }
    }

    let after = snapshot();
    let total_heap = after.0 as isize - after_warmup.0 as isize;
    let total_allocs = after.2 - after_warmup.2;
    let text = e.get_processed_str(OutputOptions::FULL_TEXT);

    println!("{}", "-".repeat(80));
    println!(
        "  FINAL:     heap={:>10}, allocs={}, dfa_states={}, text={}",
        format_bytes(total_heap.max(0) as usize),
        total_allocs,
        e.dfa_state_count(),
        format_bytes(text.len())
    );
    println!();
}

// ===========================================================================
// Test 10: EXTREME — 10,000 English sentences
// ===========================================================================

fn test_extreme_english() {
    println!("=== 10. EXTREME English — 10,000 sentences ===");

    let sentences = [
        "The quick brown fox jumps over the lazy dog.",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        "fn main() { println!(\"Hello, world!\"); }",
        "let mut count = 0; for i in 0..100 { count += i; }",
        "const MAX_BUFFER_SIZE: usize = 4096;",
        "use std::collections::HashMap;",
        "impl Display for MyStruct { fn fmt(&self, f: &mut Formatter) -> Result { ... } }",
        "async fn fetch_data(url: &str) -> Result<String, Error> { ... }",
        "struct Config { host: String, port: u16, debug: bool }",
        "match self.state { State::Running => continue, State::Stopped => break }",
    ];

    reset_counters();
    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();
    let after_warmup = snapshot();

    let checkpoints = [100, 500, 1000, 2000, 5000, 10000];
    println!(
        "{:<12} {:>12} {:>12} {:>12} {:>15}",
        "Sentences", "Heap alloc", "DFA states", "DFA arena", "committed_text"
    );
    println!("{}", "-".repeat(70));

    for i in 0..10000 {
        let s = sentences[i % sentences.len()];
        e.process_str(s, Mode::English);
        e.process_key('\n', Mode::English);
        e.commit();

        if checkpoints.contains(&(i + 1)) {
            let now = snapshot();
            let heap = now.0 as isize - after_warmup.0 as isize;
            println!(
                "{:<12} {:>12} {:>12} {:>12} {:>15}",
                i + 1,
                format_bytes(heap.max(0) as usize),
                e.dfa_state_count(),
                e.dfa_arena_len(),
                format_bytes(e.committed_text_capacity())
            );
        }
    }

    let after = snapshot();
    let total_heap = after.0 as isize - after_warmup.0 as isize;
    let total_allocs = after.2 - after_warmup.2;
    let text = e.get_processed_str(OutputOptions::FULL_TEXT);

    println!("{}", "-".repeat(70));
    println!(
        "  FINAL:     heap={:>10}, allocs={}, dfa_states={}, text={}",
        format_bytes(total_heap.max(0) as usize),
        total_allocs,
        e.dfa_state_count(),
        format_bytes(text.len())
    );
    println!();
}

// ===========================================================================
// Test 11: DFA saturation — does DFA stop growing?
// ===========================================================================

fn test_dfa_saturation() {
    println!("=== 11. DFA saturation — 50,000 unique Vietnamese words ===");

    let mut words = Vec::new();
    let prefixes = [
        "", "b", "c", "ch", "d", "g", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh", "p", "ph",
        "q", "r", "s", "t", "th", "tr", "v", "x",
    ];
    let vowels = [
        "a", "e", "i", "o", "u", "aa", "ee", "oo", "aw", "ow", "uw", "ai", "ao", "au", "ay", "ie",
        "oa", "oe", "oi", "ua", "ue", "ui", "uo", "uy",
    ];
    let tones = ["", "s", "f", "r", "x", "j"];

    for p in &prefixes {
        for v in &vowels {
            for t in tones {
                words.push(format!("{}{}{}", p, v, t));
            }
        }
    }

    println!("  Generated {} unique syllable combinations", words.len());

    reset_counters();
    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();
    let after_warmup = snapshot();

    let checkpoints = [1000, 5000, 10000, 20000, 30000, 40000, 50000];
    println!(
        "{:<12} {:>12} {:>12} {:>15} {:>15}",
        "Words", "Heap alloc", "DFA states", "DFA arena (B)", "DFA comp_map"
    );
    println!("{}", "-".repeat(70));

    for (i, w) in words.iter().enumerate() {
        e.process_str(w, Mode::Vietnamese);
        e.process_key(' ', Mode::Vietnamese);
        e.commit();

        if checkpoints.contains(&(i + 1)) {
            let now = snapshot();
            let heap = now.0 as isize - after_warmup.0 as isize;
            let arena_bytes =
                e.dfa_arena_len() * std::mem::size_of::<bamboo_core::Transformation>();
            println!(
                "{:<12} {:>12} {:>12} {:>15} {:>15}",
                i + 1,
                format_bytes(heap.max(0) as usize),
                e.dfa_state_count(),
                format_bytes(arena_bytes),
                e.dfa_composition_count()
            );
        }
    }

    let after = snapshot();
    let total_heap = after.0 as isize - after_warmup.0 as isize;
    let arena_bytes = e.dfa_arena_len() * std::mem::size_of::<bamboo_core::Transformation>();

    println!("{}", "-".repeat(70));
    println!(
        "  FINAL: heap={:>10}, dfa_states={}, dfa_arena={}, comp_map={}",
        format_bytes(total_heap.max(0) as usize),
        e.dfa_state_count(),
        format_bytes(arena_bytes),
        e.dfa_composition_count()
    );
    println!();
}

// ===========================================================================
// Test 12: Sustained typing — no commit
// ===========================================================================

fn test_sustained_no_commit() {
    println!("=== 12. Sustained typing — no commit, single long word ===");

    let long_input = "tieengsvietjhuowngquoocsnguwowifnamfchuyeennthuyeetruw\
                      owjtnghieengshoaskhongfduowcjnhuwngfmoiwlaajtuoiftroiwf\
                      doocsmuawsphujquyfnguxhoaxbuwowsngoiflamfanhfbuocjduownjg";

    reset_counters();
    let mut e = Engine::new(InputMethod::telex());
    e.warm_up();
    let after_warmup = snapshot();

    let chars: Vec<char> = long_input.chars().collect();
    let checkpoints = [10, 50, 100, 200, 300, 500, chars.len()];

    println!(
        "{:<12} {:>12} {:>12} {:>12} {:>12}",
        "Chars typed", "Heap alloc", "DFA states", "Active len", "Snapshot len"
    );
    println!("{}", "-".repeat(65));

    for (i, &c) in chars.iter().enumerate() {
        e.process_key(c, Mode::Vietnamese);

        if checkpoints.contains(&(i + 1)) || i + 1 == chars.len() {
            let now = snapshot();
            let heap = now.0 as isize - after_warmup.0 as isize;
            println!(
                "{:<12} {:>12} {:>12} {:>12} {:>12}",
                i + 1,
                format_bytes(heap.max(0) as usize),
                e.dfa_state_count(),
                e.active_len(),
                e.snapshot_len()
            );
        }
    }
    println!();
}

// ===========================================================================
// Main
// ===========================================================================

fn main() {
    // `cargo test --all-targets` executes bench mains with no args: run a
    // fast smoke instead of the EXTREME suite (`cargo bench` only, which
    // passes `--bench`).
    if !std::env::args().any(|a| a == "--bench") {
        println!("(smoke under `cargo test`; run `cargo bench` for the full profiling suite)");
        test_struct_sizes();
        return;
    }
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║            Bamboo-core — RAM & Memory Profiling (EXTREME)            ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();

    test_struct_sizes();
    test_engine_new_cost();
    test_warm_up_cost();
    test_ram_growth();
    test_dfa_growth();
    test_reset_leak();
    test_uppercase_dfa_bypass();
    test_realistic_session();
    test_extreme_vietnamese();
    test_extreme_english();
    test_dfa_saturation();
    test_sustained_no_commit();

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                          Profiling Complete                        ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
}
