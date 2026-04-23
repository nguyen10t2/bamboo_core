//! # Bamboo Core Rust
//!
//! A high-performance Vietnamese input method engine (IME) core, ported from the original
//! [bamboo-core](https://github.com/BambooEngine/bamboo-core) in Go.
//!
//! This crate provides the foundational logic for processing Vietnamese text, supporting
//! various input methods like Telex, VNI, and VIQR. It is designed to be extremely fast,
//! memory-efficient (zero-allocation in core processing), and easy to integrate into
//! UI applications or other text processing tools.
//!
//! ## Core Features
//!
//! - **Hybrid Engine**: Combines rule-based flexibility with DFA-based speed.
//! - **Zero-Allocation**: Core processing path avoids heap allocations for maximum performance.
//! - **Lazy JIT DFA**: Learns and caches syllable states on-the-fly.
//! - **WASM Ready**: Optimized for web and constrained environments.
//!
//! ## Core Concepts
//!
//! - **[`Engine`]**: The main stateful processor. You feed it characters/strings, and it
//!   maintains the internal composition state to produce the correctly marked Vietnamese text.
//! - **[`InputMethod`]**: Defines the rules for transformations (e.g., how "as" becomes "á").
//!   Built-in methods include [`InputMethod::telex()`], [`InputMethod::vni()`], etc.
//! - **[`Mode`]**: Determines if the engine should process characters as Vietnamese or
//!   treat them as plain English.
//! - **[`OutputOptions`]**: A bitmask to customize the flattened string output (e.g., lowercase,
//!   toneless, etc.).
//!
//! ## Quick Start
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod, OutputOptions};
//!
//! // Create an engine with the standard Telex input method
//! let mut engine = Engine::new(InputMethod::telex());
//!
//! // Optional: pre-populate DFA for peak performance
//! engine.warm_up();
//!
//! // Process a string and get the output immediately
//! let word = engine.process("tieengs", Mode::Vietnamese);
//! assert_eq!(word, "tiếng");
//!
//! // Reset for a new word
//! engine.reset();
//! let word2 = engine.process("vieetj", Mode::Vietnamese);
//! assert_eq!(word2, "việt");
//! ```
//!
//! ## Advanced Usage
//!
//! ### Customizing Output
//!
//! You can use [`OutputOptions`] to transform the result on the fly:
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod, OutputOptions};
//!
//! let mut engine = Engine::new(InputMethod::telex());
//! engine.process_str("Trangws", Mode::Vietnamese);
//!
//! // Get toneless version
//! let options = OutputOptions::TONE_LESS;
//! assert_eq!(engine.get_processed_str(options), "Trăng");
//! ```
//!
//! ### Handling Backspaces
//!
//! The engine supports removing the last transformation while maintaining DFA consistency:
//!
//! ```rust
//! # use bamboo_core::{Engine, Mode, InputMethod};
//! # let mut engine = Engine::new(InputMethod::telex());
//! engine.process_str("chuyeenr", Mode::Vietnamese);
//! assert_eq!(engine.output(), "chuyển");
//!
//! // remove_last_char removes the last physical keystroke and its effects
//! engine.remove_last_char(true);
//! assert_eq!(engine.output(), "chuyên");
//! ```

mod bamboo_util;
mod charset_def;
mod config;
mod dfa;
mod encoder;
mod engine;
mod flattener;
mod input_method;
mod input_method_def;
mod mode;
mod spelling;
mod utils;

pub mod ffi;
pub mod wasm;

pub use config::Config;
pub use engine::{Engine, Transformation, TransformationStack};
pub use input_method::InputMethod;
pub use mode::{Mode, OutputOptions};

/// Advanced types for low-level interaction with the engine.
///
/// This module exposes internal structures and raw definitions
/// for users who need to build custom input methods or analyze the composition state.
pub mod advanced {
    pub use crate::engine::{MAX_ACTIVE_TRANS, Transformation, TransformationStack};
    pub use crate::input_method::{EffectType, Mark, Rule, Tone};
    pub use crate::mode::OutputOptions;

    pub use crate::charset_def::{get_charset_definition, get_charset_definitions};
    pub use crate::dfa::{Dfa, State};
    pub use crate::encoder::{encode, get_charset_name, get_charset_names};
    pub use crate::input_method_def::{get_input_method, get_input_method_definitions};
}
