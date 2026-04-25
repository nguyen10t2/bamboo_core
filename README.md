# Bamboo Core (Rust)

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Vietnamese input method engine (IME) core written in Rust, inherited and optimized from the original [bamboo-core](https://github.com/BambooEngine/bamboo-core) in Go.

## 🚀 Version 0.3.5: High-Performance Snapshot

This version marks a significant milestone in efficiency and stability, achieving a **Zero-Allocation Hot Path** while maintaining absolute linguistic correctness.

### Key Enhancements:
- **Zero-Allocation Processing:** Replaced `Vec` with stack-allocated `TransformationStack` for all intermediate processing, making the engine ideal for WASM and embedded systems.
- **Lazy JIT DFA Engine:** 20x performance improvement (~490µs -> ~23µs per processing cycle) through dynamic state caching.
- **Robust State Recovery:** Completely refactored `remove_last_char` and backspace logic to ensure the internal DFA state and transformation targets are always perfectly synchronized.
- **Pre-compilation Support:** New `Engine::warm_up()` method to pre-populate the DFA with common Vietnamese syllables.
- **Optimized Validation:** High-performance single-pass O(N) spelling and syllable validation.

## 💡 Origin & Philosophy

This project is a high-performance Rust port of the **Bamboo** Vietnamese engine, originally developed by **Luong Thanh Lam**.

Bamboo provides a flexible Vietnamese typing solution based on **rule-based transformations**. This approach allows the engine to easily adapt to various typing styles (Telex, VNI, VIQR) and support modern features like free-style typing and intelligent spell checking.

The core philosophy is inspired by:
- **[bogo.js](https://github.com/lewtds/bogo.js)**: Pioneering the transformation model.
- **GoTiengViet**: The gold standard for accuracy and user experience.
- **[NexusKey](https://github.com/phatMT97/NexusKey)**: Modern state machine and array optimization techniques.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bamboo-core = "0.3.3"
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
- **Original Author (Go):** Luong Thanh Lam ([@lamtq](https://github.com/lamtq))
- **Technical Consultant:** Mai Thanh Phát ([@phatMT97](https://github.com/phatMT97)) - Author of **NexusKey**.

## 📜 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
