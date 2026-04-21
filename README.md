# Bamboo Core Rust

A Rust port of the Bamboo Vietnamese input method editor library.

## Overview

This is a rewrite of the original [bamboo-core](https://github.com/BambooEngine/bamboo-core) library in Go, ported to Rust for improved performance and memory safety.

## Original Authors

- **Luong Thanh Lam** <ltlam93@gmail.com> - Original author of bamboo-core (Go version)
- **The Little Waltz** <goatastronaut0212@outlook.com> - Previous maintainer of bamboo-core (Go version)

## Rust Port Author

- **nguien** <nguyen10t2lhp@gmail.com>

## License

The MIT License (MIT)

Copyright (C) 2018 Luong Thanh Lam  
Copyright (C) 2024 nguien

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Performance (Benchmark)

Bamboo Core Rust is designed for high-performance and low-latency text processing. Below is a comparison between the original Go implementation and this Rust port.

### Benchmark Results (Telex)

| Implementation | Time/Op (Lower is better) | Speedup | Memory Allocations |
| :--- | :--- | :--- | :--- |
| **Go** | ~5818 µs | 1.00x | ~910 KB/op (24,178 allocs) |
| **Rust** | **~2139 µs** | **2.72x** | **Zero-allocation** |

*Environment: Intel(R) Core(TM) i7-7500U CPU @ 2.70GHz, Linux. Optimized with LTO and target-cpu=native.*

### Why Rust is faster?
- **No Garbage Collection:** Eliminates pauses and overhead associated with Go's GC.
- **Efficient Memory Layout:** Uses stack-allocated structs for transformations instead of heap-allocated pointers.
- **Link-Time Optimization (LTO):** Deep compiler optimizations across crate boundaries.

## Installation

- Original Go implementation: [BambooEngine/bamboo-core](https://github.com/BambooEngine/bamboo-core)
- Credits: Trung Ngo (bogo.js), Tran Ky Nam (GoTiengViet)
