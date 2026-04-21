//! Bamboo Core Rust - Vietnamese Input Method Engine
//!
//! This crate provides a high-level Vietnamese input method engine supporting
//! multiple input methods (Telex, VNI, VIQR, Microsoft layout, etc.).
//!
//! # Quick Start
//!
//! ```
//! use bamboo_core::{Engine, Mode, BambooEngine, ESTD_FLAGS, parse_input_method};
//!
//! // Create an engine with Telex input method
//! let im = parse_input_method("Telex");
//! let mut engine = BambooEngine::new(im, ESTD_FLAGS);
//!
//! // Process some keys
//! engine.process_key('t', Mode::VIETNAMESE);
//! engine.process_key('r', Mode::VIETNAMESE);
//! engine.process_key('a', Mode::VIETNAMESE);
//! engine.process_key('n', Mode::VIETNAMESE);
//!
//! // Get the processed string (English mode to see the raw input)
//! let result = engine.get_processed_str(Mode::ENGLISH);
//! assert_eq!(result, "tran");
//! ```
//!
//! # Features
//!
//! - Multiple input methods: Telex, VNI, VIQR, Microsoft layout, and more
//! - Full Unicode support
//! - Tone mark positioning rules
//! - Mark transformation support
//! - Input validation and auto-correction

mod bamboo;
mod bamboo_util;
mod charset_def;
mod encoder;
mod flattener;
mod input_method_def;
mod rules_parser;
mod spelling;
mod utils;

// Re-export public types at the crate root
pub use bamboo::{BambooEngine, ESTD_FLAGS, Engine, Mode};
pub use bamboo::{
    ENGLISH_MODE, FULL_TEXT, IN_REVERSE_ORDER, LOWER_CASE, MARK_LESS,
    PUNCTUATION_MODE, TONE_LESS, VIETNAMESE_MODE,
};
pub use charset_def::{get_charset_definition, get_charset_definitions};
pub use encoder::{encode, get_charset_names};
pub use input_method_def::{get_input_method, get_input_method_definitions};
pub use rules_parser::{
    EffectType, InputMethod, Mark, Rule, Tone, parse_input_method, parse_rules,
    parse_toneless_rules,
};
