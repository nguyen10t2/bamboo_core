# Bamboo Core (Rust)

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Vietnamese input method engine (IME) core written in Rust, inherited and optimized from the original [bamboo-core](https://github.com/BambooEngine/bamboo-core) in Go.

## 🚀 Version 0.3.2: The DFA Revolution

This version introduces a **major architectural overhaul** inspired by Deterministic Finite Automata (DFA) and high-performance engines like `uvie-rs`.

- **Lazy JIT DFA Engine:** 20x performance improvement (~490µs -> ~25µs per processing cycle).
- **Zero-Allocation Core:** Optimized transformation generation using stack buffers.
- **Fast O(N) Validation:** Improved spelling and syllable validation algorithm.
- **Hybrid Architecture:** Combines the flexibility of a rule engine with the speed of static lookups.

## 💡 Origin & Philosophy

This project is a high-performance Rust port of the **Bamboo** Vietnamese engine, originally developed by **Luong Thanh Lam**.

Bamboo aims to provide a flexible Vietnamese typing solution based on **rule-based transformations** rather than hardcoded logic. This approach allows the engine to easily adapt to various typing styles and support modern features like free-style typing and intelligent spell checking.

The core philosophy is inspired by:
- **[bogo.js](https://github.com/lewtds/bogo.js)**: A pioneering project by **Trung Ngo**, introducing the transformation model.
- **GoTiengViet**: A classic engine by **Tran Ky Nam**, the gold standard for accuracy and user experience.
- **[NexusKey](https://github.com/phatMT97/NexusKey)**: Optimization techniques from **Mai Thanh Phát**.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bamboo-core = "0.3.2"
```

## 🛠️ Quick Start

```rust
use bamboo_core::{Engine, Mode, InputMethod};

fn main() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.process_str("tieengs", Mode::Vietnamese);
    println!("Output: {}", engine.output()); // Result: "tiếng"
}
```

## 🧩 Delta API for IMEs

Efficiently update your IME buffer using the Delta API:

```rust
let (backspaces, _, inserted) = engine.process_key_delta('s', Mode::Vietnamese);
// Result: backspaces = 1 (delete 'a'), inserted = "á"
```

## 👥 Credits

- **Rust Port & Optimization:** Dao Trong Nguyen ([@nguyen10t2](https://github.com/nguyen10t2))
- **Original Author (Go):** Luong Thanh Lam ([@lamtq](https://github.com/lamtq))
- **Technical Consultant:** Mai Thanh Phát ([@phatMT97](https://github.com/phatMT97)) - Author of **NexusKey**.

## 📜 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
