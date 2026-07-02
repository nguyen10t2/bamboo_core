# Changelog

All notable changes to this project will be documented in this file.

## [0.3.6] - 2026-07-03

### Performance
- **DFA Fast Path 2× Faster (~100ns → ~45ns):** Reorganized `process_key` into three distinct paths (English → DFA fast → slow). The DFA fast path now skips `can_process_key_raw` validation entirely since cached transitions guarantee key validity.
- **Compact Transformation Struct (48 → 28 bytes):** Changed `target` field from `Option<usize>` (16 bytes) to `Option<u8>` (2 bytes). Since `MAX_ACTIVE_TRANS = 16`, `u8` is sufficient. This reduces `[Transformation; 16]` from 768 to 448 bytes (42% smaller), improving CPU cache utilization.
- **Pre-allocated `committed_text`:** Now pre-allocates 256 bytes to avoid reallocation during typical Vietnamese text input.

### Changed
- `Transformation.target` type: `Option<usize>` → `Option<u8>`
- `find_root_target()` signature: `usize` → `u8` for target parameter and return
- `find_tone_target()`, `find_mark_target()`, `find_target()`: return types use `u8` instead of `usize`

### Internal
- Added `benches/engine_bench.rs` with benchmarks for DFA fast path, DFA miss path, output, process_key_delta, mixed typing, backspace, and many words scenarios.

## [0.3.5] - 2026-07-03

### Performance
- Zero-allocation hot path using stack-allocated `TransformationStack`
- Lazy JIT DFA engine with dynamic state caching
- Arena-based DFA state storage

### Fixed
- Robust state recovery in `remove_last_char` and backspace logic
- DFA state and transformation target synchronization

## [0.3.3] - 2026-06-01

### Added
- `Engine::warm_up()` for DFA pre-compilation
- High-performance single-pass O(N) spelling validation

### Performance
- 20× performance improvement through DFA caching (~490µs → ~23µs per cycle)

## [0.3.0] - 2026-05-01

### Added
- `process_key_delta()` for efficient IME text updates
- `OutputOptions` bitflags for output customization
- `Config` struct for engine configuration
- FFI layer for C/C++ integration
- WASM bindings via `wasm-bindgen`

### Changed
- Major refactor of engine architecture
- New transformation-based input processing model

## [0.2.1] - 2026-04-01

### Fixed
- Various input method parsing issues
- Documentation improvements

## [0.1.2] - 2026-03-01

### Added
- FFI layer for external integration
- Initial documentation

## [0.1.1] - 2026-02-01

### Added
- Initial release
- Telex, VNI, VIQR input methods
- Unicode support
- Basic engine functionality
