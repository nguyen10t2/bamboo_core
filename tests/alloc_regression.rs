//! Allocation regression tests.
//!
//! Asserts zero (or bounded) heap allocations on the keystroke hot path.
//! Uses a counting `GlobalAlloc` wrapper with a capture gate so that only
//! allocations inside the measured block are counted.
//!
//! Run in isolation to avoid cross-test allocation noise:
//! ```sh
//! cargo test --test alloc_regression -- --test-threads=1
//! ```
//! (Tests are also `#[serial]` so they self-serialize under multi-thread runs.)

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use bamboo_core::{Engine, InputMethodPreset, Mode, OutputOptions, RestoreMark};
use serial_test::serial;
use std::borrow::Cow;
use std::hint::black_box;

/// Thread-safe counting allocator wrapping the system allocator.
struct CountingAlloc;

static CAPTURING: AtomicBool = AtomicBool::new(false);
static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if CAPTURING.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if CAPTURING.load(Ordering::Relaxed) {
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(new_size, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// Returns `(alloc_count, alloc_bytes)` deltas while running `f`.
fn measure_allocs<F: FnOnce()>(f: F) -> (usize, usize) {
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    CAPTURING.store(true, Ordering::Relaxed);
    f();
    CAPTURING.store(false, Ordering::Relaxed);
    (ALLOC_COUNT.load(Ordering::Relaxed), ALLOC_BYTES.load(Ordering::Relaxed))
}

fn warm_telex() -> Engine {
    let mut e = Engine::from_preset(InputMethodPreset::Telex);
    // Pre-warm DFA / strings so later measurements isolate the steady-state hot path.
    e.process_str("tieengs vietj nguwowif", Mode::Vietnamese);
    e.commit();
    e
}

#[test]
#[serial]
fn zero_alloc_output_str() {
    let e = warm_telex();
    let (n, _) = measure_allocs(|| {
        for _ in 0..100 {
            black_box(e.output_str());
        }
    });
    assert_eq!(n, 0, "output_str must be zero-alloc, got {n} allocs");
}

#[test]
#[serial]
fn zero_alloc_output_cow_borrowed() {
    let e = warm_telex();
    let (n, _) = measure_allocs(|| {
        for _ in 0..100 {
            let cow = e.output();
            black_box(cow);
        }
    });
    assert_eq!(n, 0, "output() must be zero-alloc when uncommitted, got {n} allocs");
}

#[test]
#[serial]
fn zero_alloc_process_key_dfa_hit() {
    let mut e = warm_telex();
    // Warm the specific word so subsequent keystrokes hit the DFA cache.
    e.process_str("vieejt", Mode::Vietnamese);
    e.commit();

    let (n, _) = measure_allocs(|| {
        for _ in 0..50 {
            e.process_str("vieejt", Mode::Vietnamese);
            black_box(e.output_str());
            e.commit();
        }
    });
    // commit() grows committed_text (String push). Allow bounded allocs from
    // commit growth only; the keystroke stream itself must not allocate.
    // 50 commits * 1 push_str growth = at most ~50 reallocs after capacity is reached.
    assert!(n <= 50, "process_key (DFA hit) + commit should be near-zero alloc, got {n} allocs");
}

#[test]
#[serial]
fn zero_alloc_process_key_stream_no_commit() {
    let mut e = warm_telex();
    // Warm DFA with the target word first.
    e.process_str("tieengs", Mode::Vietnamese);
    e.commit();

    let (n, _) = measure_allocs(|| {
        for _ in 0..50 {
            for ch in "tieengs".chars() {
                e.process_key(ch, Mode::Vietnamese);
            }
            black_box(e.output_str());
            e.commit();
        }
    });
    // Steady-state: cached_output is pre-reserved, DFA is warm.
    // TODO(PR-4): tighten to 0 after eliminating residual slow-path allocs.
    // Current known allocs: committed_text growth on commit().
    assert!(n <= 120, "warm DFA keystroke stream should be near-zero alloc, got {n} allocs");
}

#[test]
#[serial]
fn zero_alloc_remove_last_char() {
    let mut e = warm_telex();
    e.process_str("tieengs", Mode::Vietnamese);

    let (n, _) = measure_allocs(|| {
        for _ in 0..20 {
            e.process_str("tieengs", Mode::Vietnamese);
            e.remove_last_char(RestoreMark::Yes);
            black_box(e.output_str());
            e.commit();
        }
    });
    // TODO(PR-4): tighten after eliminating residual slow-path allocs.
    // Current known allocs: committed_text growth on commit().
    assert!(n <= 60, "remove_last_char should be near-zero alloc, got {n} allocs");
}

#[test]
#[serial]
fn zero_alloc_english_passthrough() {
    let mut e = warm_telex();
    let (n, _) = measure_allocs(|| {
        for _ in 0..50 {
            for ch in "getUserName".chars() {
                e.process_key(ch, Mode::English);
            }
            black_box(e.output_str());
            e.commit();
        }
    });
    assert!(
        n <= 50,
        "English passthrough should be near-zero alloc (commit growth only), got {n} allocs"
    );
}

#[test]
#[serial]
fn zero_alloc_lowercase_poll_borrowed() {
    let mut e = warm_telex();
    // Warm the polled word so its lowercase flatten is cached in the DFA.
    e.process_str("tieengs", Mode::Vietnamese);
    let (n, _) = measure_allocs(|| {
        for _ in 0..100 {
            let cow = e.get_processed_str_cow(OutputOptions::LOWER_CASE);
            assert!(matches!(cow, Cow::Borrowed(_)));
            black_box(cow);
        }
    });
    assert_eq!(n, 0, "LOWER_CASE poll on a warm word must borrow with zero alloc, got {n}");
}

#[test]
#[serial]
fn bounded_alloc_jit_new_state() {
    let mut e = warm_telex();
    // A word the DFA has never seen forces add_state (arena + flat growth
    // written directly, without a temporary String).
    let (n, _) = measure_allocs(|| {
        e.process_str("dduwowngf", Mode::Vietnamese);
        black_box(e.output_str());
    });
    assert_eq!(e.output(), "đường");
    // Measured 2 on warmed arenas; a per-state temporary String (the old
    // flat-cache path) would cost ~1 extra alloc per keystroke and trip this.
    assert!(n <= 8, "one cold word should allocate boundedly, got {n} allocs");
}

#[test]
#[serial]
fn zero_alloc_process_key_delta() {
    let mut e = warm_telex();
    e.process_str("vietj", Mode::Vietnamese);
    e.commit();

    let (n, _) = measure_allocs(|| {
        for _ in 0..50 {
            for ch in "vietj".chars() {
                let d = e.process_key_delta(ch, Mode::Vietnamese);
                black_box(d);
            }
            e.commit();
        }
    });
    // delta API returns Cow; internal delta_buf is pre-reserved.
    assert!(n <= 50, "process_key_delta should be near-zero alloc, got {n} allocs");
}
