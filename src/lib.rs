//! Bamboo Core Rust - Vietnamese Input Method Engine
//!
//! This crate provides a high-level Vietnamese input method engine supporting
//! multiple input methods (Telex, VNI, VIQR, Microsoft layout, etc.).
//!
//! # Quick Start
//!
//! ```
//! use bamboo_core::{Engine, Mode, InputMethod};
//!
//! // Create an engine with Telex input method
//! let mut engine = Engine::new(InputMethod::telex());
//!
//! // Process some keys
//! let result = engine.process_str("trangws", Mode::Vietnamese).output();
//!
//! assert_eq!(result, "trắng");
//! ```
//!
//! # Features
//!
//! - Multiple input methods: Telex, VNI, VIQR, Microsoft layout, and more
//! - Full Unicode support
//! - Tone mark positioning rules
//! - Mark transformation support
//! - Input validation and auto-correction

mod bamboo_util;
mod charset_def;
mod config;
mod encoder;
mod engine;
mod flattener;
mod input_method;
mod input_method_def;
mod mode;
mod spelling;
mod utils;

pub use config::Config;
pub use engine::Engine;
pub use input_method::InputMethod;
pub use mode::{Mode, OutputOptions};

/// Advanced types for low-level interaction with the engine.
pub mod advanced {
    pub use crate::engine::Transformation;
    pub use crate::input_method::{EffectType, Mark, Rule, Tone};
    pub use crate::mode::OutputOptions;

    pub use crate::charset_def::{
        get_charset_definition, get_charset_definitions,
    };
    pub use crate::encoder::{encode, get_charset_names};
    pub use crate::input_method_def::{
        get_input_method, get_input_method_definitions,
    };
}
