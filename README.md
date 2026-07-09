# Bamboo Core (Rust)

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Vietnamese input method engine (IME) core written in Rust, inherited and optimized from the original [bamboo-core](https://github.com/BambooEngine/bamboo-core) in Go.

## 🚀 Version 0.3.6: Pipeline & Cache Optimization

This release focuses on hot-path performance, achieving a **2× speedup** on the DFA fast path through pipeline reorganization and memory layout optimization.

### Key Enhancements:
- **DFA Fast Path 2× Faster:** Reorganized `process_key` into three distinct paths (English → DFA fast → slow), allowing the DFA fast path to skip redundant `can_process_key_raw` validation.
- **Compact Transformation Struct:** Reduced `Transformation` from 48 bytes to 28 bytes (42% smaller) by using `Option<u8>` for target indices. This improves CPU cache utilization for all buffer operations.
- **Pre-allocated Buffers:** `committed_text` now pre-allocates 256 bytes to avoid reallocation during typical usage.

### Performance (vs 0.3.5 baseline):
| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| DFA fast path | ~100 ns | **~45 ns** | 2.2× |
| DFA miss path | ~90 ns | **~43 ns** | 2.1× |
| `Transformation` size | 48 bytes | **28 bytes** | 42% smaller |
| `[Transformation; 16]` | 768 bytes | **448 bytes** | 42% smaller |

## 💡 Origin & Philosophy

This project is a high-performance Rust port of the **Bamboo** Vietnamese engine, originally developed by **Luong Thanh Lam**.

Bamboo provides a flexible Vietnamese typing solution based on **rule-based transformations**. This approach allows the engine to easily adapt to various typing styles (Telex, VNI, VIQR) and support modern features like free-style typing and intelligent spell checking.

The core philosophy is inspired by:
- **[bogo.js](https://github.com/lewtds/bogo.js)**: Pioneering the transformation model.
- **[bamboo-core](https://github.com/BambooEngine/bamboo-core)**: The gold standard for accuracy and user experience.
- **[NexusKey](https://github.com/phatMT97/NexusKey)**: Modern state machine and array optimization techniques.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bamboo-core = "0.3.6"
```

## 🛠️ Quick Start

```rust
use bamboo_core::{Engine, Mode, InputMethod};

fn main() {
    let mut engine = Engine::new(InputMethod::telex());
    
    // Optional: Boost initial performance
    engine.warm_up();
    
    let word = engine.process("tieengs", Mode::Vietnamese);
    println!("Output: {}", word); // Result: "tiếng"
}
```

## 👥 Credits

- **Rust Port & Optimization:** Dao Trong Nguyen ([@nguyen10t2](https://github.com/nguyen10t2))
- **Original Author (Go):** Lam ([@lamtq](https://github.com/t1ld3x))
- **Technical Consultant:** Mai Tan Phat ([@phatMT97](https://github.com/phatMT97)) - Author of **VKey**.

## 📜 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
