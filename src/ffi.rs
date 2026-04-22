//! C-Compatible FFI Layer for Bamboo Core.
//!
//! This module provides an `extern "C"` API for integrating Bamboo with
//! other languages like C, C++, Python, and IME frameworks (Fcitx5, IBus).

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

use crate::engine::Engine;
use crate::input_method::InputMethod;
use crate::mode::Mode;

static GLOBAL_ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

fn with_engine<F, R>(f: F) -> R
where
    F: FnOnce(&mut Engine) -> R,
    R: Default,
{
    let mut guard = match GLOBAL_ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return R::default(),
    };
    if guard.is_none() {
        *guard = Some(Engine::new(InputMethod::telex()));
    }
    f(guard.as_mut().unwrap())
}

/// Initializes the global engine with the default Telex input method.
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_setup() {
    let mut guard = GLOBAL_ENGINE.lock().unwrap();
    *guard = Some(Engine::new(InputMethod::telex()));
}

/// Resets the global engine state, clearing the current composition.
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_reset() {
    with_engine(|e| e.reset());
}

/// Sets the input method for the global engine.
///
/// # Arguments
///
/// * `method` - An integer representing the input method:
///     * 0: Telex
///     * 1: VNI
///     * 2: VIQR
///     * 3: Microsoft Layout
///     * 4: Telex 2
///     * 5: Telex W
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_set_input_method(method: i32) {
    let im = match method {
        0 => InputMethod::telex(),
        1 => InputMethod::vni(),
        2 => InputMethod::viqr(),
        3 => InputMethod::microsoft_layout(),
        4 => InputMethod::telex_2(),
        5 => InputMethod::telex_w(),
        _ => return,
    };
    let mut guard = GLOBAL_ENGINE.lock().unwrap();
    *guard = Some(Engine::new(im));
}

/// Processes a key and returns the full current word as a C-compatible string.
///
/// # Arguments
///
/// * `key` - The Unicode code point of the key to process.
/// * `is_vietnamese` - Non-zero if the key should be processed as Vietnamese, zero for English mode.
///
/// # Returns
///
/// A pointer to a null-terminated UTF-8 string.
/// **Note:** The caller is responsible for freeing the returned string using [`bamboo_free_string`].
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_process_key(
    key: u32,
    is_vietnamese: i32,
) -> *mut c_char {
    with_engine(|e| {
        let mode =
            if is_vietnamese != 0 { Mode::Vietnamese } else { Mode::English };
        if let Some(c) = std::char::from_u32(key) {
            e.process_key(c, mode);
        }
        let out = e.output();
        CString::new(out).unwrap().into_raw()
    })
}

/// Returns the current word output as a C-compatible string.
///
/// # Returns
///
/// A pointer to a null-terminated UTF-8 string.
/// **Note:** The caller is responsible for freeing the returned string using [`bamboo_free_string`].
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_output() -> *mut c_char {
    with_engine(|e| {
        let out = e.output();
        CString::new(out).unwrap().into_raw()
    })
}

/// Removes the last character from the current composition in the global engine.
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_remove_last_char() {
    with_engine(|e| e.remove_last_char(true));
}

/// Frees a string allocated by the engine and returned via FFI.
///
/// # Safety
///
/// The provided pointer must have been returned by a `bamboo_*` function and not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

// --- Instance-based API for multi-context support ---

/// Opaque handle to a Bamboo Engine instance.
pub type BambooEngine = Engine;

/// Creates a new Bamboo Engine instance.
///
/// # Arguments
///
/// * `method` - An integer representing the input method (1: VNI, 2: VIQR, others: Telex).
///
/// # Returns
///
/// A pointer to the new [`BambooEngine`] instance.
/// **Note:** The caller is responsible for freeing the engine using [`bamboo_engine_free`].
#[unsafe(no_mangle)]
pub extern "C" fn bamboo_engine_new(method: i32) -> *mut BambooEngine {
    let im = match method {
        1 => InputMethod::vni(),
        2 => InputMethod::viqr(),
        _ => InputMethod::telex(),
    };
    Box::into_raw(Box::new(Engine::new(im)))
}

/// Frees a Bamboo Engine instance created with [`bamboo_engine_new`].
///
/// # Safety
///
/// The provided pointer must be a valid pointer to a [`BambooEngine`] instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_free(engine: *mut BambooEngine) {
    if !engine.is_null() {
        unsafe {
            let _ = Box::from_raw(engine);
        }
    }
}

/// Processes a key using a specific engine instance.
///
/// # Arguments
///
/// * `engine` - A pointer to a [`BambooEngine`] instance.
/// * `key` - The Unicode code point of the key to process.
///
/// # Returns
///
/// A pointer to a null-terminated UTF-8 string representing the current word.
/// **Note:** The caller is responsible for freeing the returned string using [`bamboo_free_string`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bamboo_engine_process(
    engine: *mut BambooEngine,
    key: u32,
) -> *mut c_char {
    if engine.is_null() {
        return ptr::null_mut();
    }
    let e = unsafe { &mut *engine };
    if let Some(c) = std::char::from_u32(key) {
        e.process_key(c, Mode::Vietnamese);
    }
    let out = e.output();
    CString::new(out).unwrap().into_raw()
}
