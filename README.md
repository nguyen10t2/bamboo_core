# Bamboo Core (Rust)

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Vietnamese input method engine (IME) core written in Rust, ported from [bamboo-core](https://github.com/BambooEngine/bamboo-core) (Go).

## Features

- **Telex, VNI, VIQR** input methods with custom input method support
- **Hybrid engine**: Rule-based transformations + Lazy JIT DFA caching
- **Zero heap allocation** in core processing path (stack-allocated buffers)
- **O(1) backspace** via snapshot stack
- **O(N) single-pass** spelling validation
- **FFI** for C/C++ integration and **WASM** bindings

## Installation

```toml
[dependencies]
bamboo-core = "0.3.11"
```

## Quick Start

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

let word = engine.process("tieengs", Mode::Vietnamese);
assert_eq!(word, "tiếng");

engine.reset();
let word2 = engine.process("vieetj", Mode::Vietnamese);
assert_eq!(word2, "việt");
```

## Incremental Processing

Process keystrokes one at a time for IME integration:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

engine.process_key('t', Mode::Vietnamese);
engine.process_key('i', Mode::Vietnamese);
engine.process_key('e', Mode::Vietnamese);
engine.process_key('e', Mode::Vietnamese);
engine.process_key('n', Mode::Vietnamese);
engine.process_key('g', Mode::Vietnamese);
engine.process_key('s', Mode::Vietnamese);
assert_eq!(engine.output(), "tiếng");
```

### Delta Updates

For efficient text editor integration, use `process_key_delta` to get a **3-way diff**:

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());

let (bs, _, ins) = engine.process_key_delta('a', Mode::Vietnamese);
assert_eq!(bs, 0);
assert_eq!(ins, "a");

// previous = "a", new = "á"
let (bs, _, ins) = engine.process_key_delta('s', Mode::Vietnamese);
assert_eq!(bs, 1);     // delete 1 char ("a")
assert_eq!(ins, "á");  // insert "á"
// result: "" + "á" = "á"
```

Contract:
```text
previous = [common_prefix] + [backspace_count chars to delete]
new      = [common_prefix] + [inserted_suffix]
```
Frontend does not need to compute LCP — the engine does it.

## Backspace

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("chuyeenr", Mode::Vietnamese);
assert_eq!(engine.output(), "chuyển");

engine.remove_last_char(true);
assert_eq!(engine.output(), "chuyên");
```

## Output Customization

```rust
use bamboo_core::{Engine, Mode, InputMethod, OutputOptions};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("Trangws", Mode::Vietnamese);

// Toneless
assert_eq!(engine.get_processed_str(OutputOptions::TONE_LESS), "Trăng");

// Full text (committed + active)
assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "Trăng");
```

## Benchmarks

Benchmarked against [uvie](https://github.com/thuupx/uvie-rs) v2.1.1 on the same machine. Lower is better.

### Core Operations

| Benchmark | Bamboo | Uvie | Speedup |
|---|---:|---:|---:|
| feed single word (tieengs) | 76 ns | 874 ns | **11.5x** |
| feed + delta (per key) | 718 ns | 861 ns | **1.2x** |
| mixed typing (29 chars) | 904 ns | 3,658 ns | **4.0x** |
| many words (13 words) | 6,745 ns | 11,878 ns | **1.8x** |

### Backspace

| Benchmark | Bamboo | Uvie | Speedup |
|---|---:|---:|---:|
| backspace (1x) | 143 ns | 1,066 ns | **7.5x** |
| backspace x3 | 262 ns | 1,372 ns | **5.2x** |
| backspace spam (7x) | 405 ns | 1,704 ns | **4.2x** |

### Real-world Scenarios

| Benchmark | Bamboo | Uvie | Speedup |
|---|---:|---:|---:|
| english passthrough (code) | 348 ns | 1,506 ns | **4.3x** |
| long identifier (39 chars) | 566 ns | 10,467 ns | **18.5x** |
| commit via space | 167 ns | 868 ns | **5.2x** |
| worst-case syllable (nghieengs) | 98 ns | 990 ns | **10.1x** |
| random typing (real workload) | 3,892 ns | 8,488 ns | **2.2x** |

> **Note**: Bamboo's cold start (first keystroke) is slower (~19 µs vs ~1.5 µs) due to Engine + DFA allocation. After warmup, the DFA fast path dominates.

Run benchmarks: `cargo bench`

## Credits

- **Rust Port & Optimization:** Dao Trong Nguyen ([@nguyen10t2](https://github.com/nguyen10t2))
- **Original Author (Go):** Lam ([@lamtq](https://github.com/t1ld3x))
- **Technical Consultant:** Mai Tan Phat ([@phatMT97](https://github.com/phatMT97)) - Author of **VKey**

## License

MIT. See [LICENSE](LICENSE).
