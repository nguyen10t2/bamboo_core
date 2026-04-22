//! Provides functions for encoding Vietnamese text into different character sets.

use crate::charset_def::{get_charset_definition, get_charset_definitions};

static UNICODE: &str = "Unicode";

/// Encodes a Vietnamese string into a specific character set (e.g., VNI-Windows, TCVN3).
///
/// If the `charset_name` is "Unicode", it returns the input string unchanged.
pub fn encode(charset_name: &str, input: &str) -> String {
    if charset_name == UNICODE {
        return input.to_string();
    }
    let mut output = String::new();

    match get_charset_definition(charset_name) {
        Some(charset_def) => {
            for char in input.chars() {
                match charset_def.get(&char) {
                    Some(encoded) => output.push_str(encoded),
                    None => output.push(char),
                }
            }
        }
        None => {
            output = input.to_string();
        }
    }
    output
}

/// Returns a list of all supported character set names.
pub fn get_charset_name() -> Vec<String> {
    let mut charset_names = Vec::with_capacity(get_charset_definitions().len() + 1);

    charset_names.push(UNICODE.to_string());
    for (k, _) in get_charset_definitions() {
        charset_names.push(k.to_string());
    }
    charset_names
}

/// Alias for [`get_charset_name`].
pub fn get_charset_names() -> Vec<String> {
    get_charset_name()
}
