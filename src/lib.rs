//! # Bamboo Core
//!
//! High-performance Vietnamese input method engine (IME) core written in Rust.
//! Ported from [bamboo-core](https://github.com/BambooEngine/bamboo-core) (Go).
//!
//! Supports **Telex**, **VNI**, **VIQR** input methods with custom rule support.
//!
//! ## API Overview
//!
//! | API | Use case | Description |
//! |---|---|---|
//! | [`Engine::process_key`] | **IME integration (recommended)** | Process one keystroke, update internal state |
//! | [`Engine::process_key_delta`] | **Text editor integration** | Like `process_key`, returns `(backspaces, inserted)` diff |
//! | [`Engine::process`] | Convenience | Process a full string, return output |
//! | [`Engine::output`] | Read state | Get current composing word as `String` |
//! | [`Engine::remove_last_char`] | Backspace | Undo last keystroke (O(1) via snapshot stack) |
//! | [`Engine::commit`] | Confirm word | Finalize composing word into committed text |
//! | [`Engine::reset`] | New session | Clear all state |
//!
//! ## Quick Start — IME Integration
//!
//! Feed keystrokes one at a time with [`Engine::process_key`]:
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod};
//!
//! let mut engine = Engine::new(InputMethod::telex());
//!
//! engine.process_key('t', Mode::Vietnamese);
//! engine.process_key('i', Mode::Vietnamese);
//! engine.process_key('e', Mode::Vietnamese);
//! engine.process_key('e', Mode::Vietnamese);
//! engine.process_key('n', Mode::Vietnamese);
//! engine.process_key('g', Mode::Vietnamese);
//! engine.process_key('s', Mode::Vietnamese);
//! assert_eq!(engine.output(), "tiếng");
//! ```
//!
//! For text editors, use [`Engine::process_key_delta`] to get only the diff
//! (number of backspaces + new text to insert):
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod};
//!
//! let mut engine = Engine::new(InputMethod::telex());
//!
//! let (bs, _, ins) = engine.process_key_delta('a', Mode::Vietnamese);
//! assert_eq!(bs, 0);
//! assert_eq!(ins, "a");
//!
//! let (bs, _, ins) = engine.process_key_delta('s', Mode::Vietnamese);
//! assert_eq!(bs, 1); // delete "a"
//! assert_eq!(ins, "á"); // insert "á"
//! ```
//!
//! ## Convenience API
//!
//! [`Engine::process`] processes a full string in one call — useful for testing
//! or batch processing:
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod};
//!
//! let mut engine = Engine::new(InputMethod::telex());
//! assert_eq!(engine.process("tieengs", Mode::Vietnamese), "tiếng");
//!
//! engine.reset();
//! assert_eq!(engine.process("vieetj", Mode::Vietnamese), "việt");
//! ```
//!
//! ## Backspace
//!
//! [`Engine::remove_last_char`] undoes the last keystroke in O(1) time
//! (snapshot stack, zero heap allocation):
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod};
//!
//! let mut engine = Engine::new(InputMethod::telex());
//! engine.process_str("chuyeenr", Mode::Vietnamese);
//! assert_eq!(engine.output(), "chuyển");
//!
//! engine.remove_last_char(true);
//! assert_eq!(engine.output(), "chuyên");
//! ```
//!
//! ## Output Customization
//!
//! Use [`OutputOptions`] to transform the output:
//!
//! ```rust
//! use bamboo_core::{Engine, Mode, InputMethod, OutputOptions};
//!
//! let mut engine = Engine::new(InputMethod::telex());
//! engine.process_str("Trangws", Mode::Vietnamese);
//!
//! assert_eq!(engine.get_processed_str(OutputOptions::TONE_LESS), "Trăng");
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
