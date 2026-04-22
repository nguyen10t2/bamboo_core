# Bamboo Core Rust

[![Crates.io](https://img.shields.io/crates/v/bamboo-core.svg)](https://crates.io/crates/bamboo-core)
[![Documentation](https://docs.rs/bamboo-core/badge.svg)](https://docs.rs/bamboo-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, $O(1)$ scaling Vietnamese input method engine (IME) core, ported from the original [bamboo-core](https://github.com/BambooEngine/bamboo-core) in Go.

## Overview

Bamboo Core Rust is a complete rewrite of the original library, optimized for maximum performance, memory safety, and cross-platform scalability. It is designed to be the foundational engine for modern Vietnamese IMEs on Linux, Windows, macOS, and the Web.

## Key Features (v0.2.0)

- **Syllable-Based Processing:** Performance remains constant ($O(1)$) regardless of document length.
- **Zero Heap Allocations:** Core typing logic uses a stack-based active buffer, eliminating garbage collection pauses and allocator overhead.
- **Rule-Based Flexibility:** Supports all major input methods (Telex, VNI, VIQR) via a highly optimized rule engine.
- **Multi-Platform:**
  - **Rust API:** For native Rust applications.
  - **C-FFI:** Stable C-compatible interface for integration with C++, Python, and native IME frameworks (Fcitx5, IBus).
  - **WebAssembly (WASM):** Built-in support for browser-based editors and extensions.

## Performance (Benchmark)

Comparison against the original Go implementation using complex Vietnamese sentences.

| Implementation | Time/Op (Lower is better) | Speedup | Memory Allocations | Complexity |
| :--- | :--- | :--- | :--- | :--- |
| **Go (Original)** | ~6196 µs | 1.00x | ~24,000+ allocs | $O(N^2)$ |
| **Rust (v0.2.0)** | **~1385 µs** | **~4.50x** | **Zero (during typing)** | **$O(1)$** |

*Environment: Intel(R) Core(TM) i7-7500U CPU @ 2.70GHz, Linux. Optimized with LTO and native CPU flags.*

## Quick Start

### Rust
Add this to your `Cargo.toml`:
```toml
[dependencies]
bamboo-core = "0.2.0"
```

```rust
use bamboo_core::{Engine, Mode, InputMethod};

let mut engine = Engine::new(InputMethod::telex());
engine.process_str("tieengs vieetj", Mode::Vietnamese);

assert_eq!(engine.output(), "việt"); // engine.output() returns the active word
```

### WebAssembly
Enable the `wasm` feature:
```bash
wasm-pack build --scope myorg -- --features wasm
```

## Credits

- **Luong Thanh Lam** <ltlam93@gmail.com> - Original author of bamboo-core (Go version)
- **nguien** - Rust port and optimization author

## License

The MIT License (MIT)

Copyright (C) 2018 Luong Thanh Lam  
Copyright (C) 2024 nguien

See [LICENSE](LICENSE) for full details.
