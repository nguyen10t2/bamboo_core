//! Utility functions for Vietnamese character manipulation.
//!
//! This module provides low-level tools for identifying vowels, adding tones,
//! and managing diacritic marks.

use phf::{Map, Set, phf_map, phf_set};

/// A list of all Vietnamese vowels with their various tone marks.
pub const VOWELS: &[char] = &[
    'a', 'à', 'á', 'ả', 'ã', 'ạ', 'ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ', 'â', 'ầ', 'ấ',
    'ẩ', 'ẫ', 'ậ', 'e', 'è', 'é', 'ẻ', 'ẽ', 'ẹ', 'ê', 'ề', 'ế', 'ể', 'ễ', 'ệ',
    'i', 'ì', 'í', 'ỉ', 'ĩ', 'ị', 'o', 'ò', 'ó', 'ỏ', 'õ', 'ọ', 'ô', 'ồ', 'ố',
    'ổ', 'ỗ', 'ộ', 'ơ', 'ờ', 'ớ', 'ở', 'ỡ', 'ợ', 'u', 'ù', 'ú', 'ủ', 'ũ', 'ụ',
    'ư', 'ừ', 'ứ', 'ử', 'ữ', 'ự', 'y', 'ỳ', 'ý', 'ỷ', 'ỹ', 'ỵ',
];

static VOWELS_SET: Set<char> = phf_set! {
    'a', 'à', 'á', 'ả', 'ã', 'ạ', 'ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ', 'â', 'ầ', 'ấ',
    'ẩ', 'ẫ', 'ậ', 'e', 'è', 'é', 'ẻ', 'ẽ', 'ẹ', 'ê', 'ề', 'ế', 'ể', 'ễ', 'ệ',
    'i', 'ì', 'í', 'ỉ', 'ĩ', 'ị', 'o', 'ò', 'ó', 'ỏ', 'õ', 'ọ', 'ô', 'ồ', 'ố',
    'ổ', 'ỗ', 'ộ', 'ơ', 'ờ', 'ớ', 'ở', 'ỡ', 'ợ', 'u', 'ù', 'ú', 'ủ', 'ũ', 'ụ',
    'ư', 'ừ', 'ứ', 'ử', 'ữ', 'ự', 'y', 'ỳ', 'ý', 'ỷ', 'ỹ', 'ỵ',
};

static VOWEL_INDEX: Map<char, usize> = phf_map! {
    'a'=>0,'à'=>1,'á'=>2,'ả'=>3,'ã'=>4,'ạ'=>5,
    'ă'=>6,'ằ'=>7,'ắ'=>8,'ẳ'=>9,'ẵ'=>10,'ặ'=>11,
    'â'=>12,'ầ'=>13,'ấ'=>14,'ẩ'=>15,'ẫ'=>16,'ậ'=>17,
    'e'=>18,'è'=>19,'é'=>20,'ẻ'=>21,'ẽ'=>22,'ẹ'=>23,
    'ê'=>24,'ề'=>25,'ế'=>26,'ể'=>27,'ễ'=>28,'ệ'=>29,
    'i'=>30,'ì'=>31,'í'=>32,'ỉ'=>33,'ĩ'=>34,'ị'=>35,
    'o'=>36,'ò'=>37,'ó'=>38,'ỏ'=>39,'õ'=>40,'ọ'=>41,
    'ô'=>42,'ồ'=>43,'ố'=>44,'ổ'=>45,'ỗ'=>46,'ộ'=>47,
    'ơ'=>48,'ờ'=>49,'ớ'=>50,'ở'=>51,'ỡ'=>52,'ợ'=>53,
    'u'=>54,'ù'=>55,'ú'=>56,'ủ'=>57,'ũ'=>58,'ụ'=>59,
    'ư'=>60,'ừ'=>61,'ứ'=>62,'ử'=>63,'ữ'=>64,'ự'=>65,
    'y'=>66,'ỳ'=>67,'ý'=>68,'ỷ'=>69,'ỹ'=>70,'ỵ'=>71,
};

static PUNCTUATION: Set<char> = phf_set! {
    ',', ';', ':', '.', '"', '\'', '!', '?', ' ',
    '<', '>', '=', '+', '-', '*', '/', '\\',
    '_', '~', '`', '@', '#', '$', '%', '^', '&', '(', ')', '{', '}', '[', ']',
    '|',
};

static MARKS_MAPS: Map<char, [char; 5]> = phf_map! {
    'a' => ['a','â','ă','_','_'],
    'â' => ['a','â','ă','_','_'],
    'ă' => ['a','â','ă','_','_'],

    'e' => ['e','ê','_','_','_'],
    'ê' => ['e','ê','_','_','_'],

    'o' => ['o','ô','_','ơ','_'],
    'ô' => ['o','ô','_','ơ','_'],
    'ơ' => ['o','ô','_','ơ','_'],

    'u' => ['u','_','_','ư','_'],
    'ư' => ['u','_','_','ư','_'],

    'd' => ['d','_','_','_','đ'],
    'đ' => ['d','_','_','_','đ'],
};

/// Returns true if the character is a space.
#[inline]
pub fn is_space(c: char) -> bool {
    c == ' '
}

/// Returns true if the character is a common punctuation mark.
#[inline]
pub fn is_punctuation(c: char) -> bool {
    PUNCTUATION.contains(&c)
}

/// Returns true if the character should trigger a word break.
#[inline]
pub fn is_word_break_symbol(c: char) -> bool {
    is_punctuation(c) || c.is_ascii_digit()
}

/// Returns true if the character is a Vietnamese vowel.
#[inline]
pub fn is_vowel(c: char) -> bool {
    VOWELS_SET.contains(&c)
}

/// Returns true if the character is an ASCII alphabetic character.
#[inline]
pub fn is_alpha(c: char) -> bool {
    c.is_ascii_alphabetic()
}

#[inline]
fn find_vowel_position(c: char) -> Option<usize> {
    VOWEL_INDEX.get(&c).copied()
}

#[inline]
fn find_tone_from_char(c: char) -> u8 {
    find_vowel_position(c).map(|pos| (pos % 6) as u8).unwrap_or(0)
}

/// Adds or changes the tone mark of a Vietnamese vowel.
#[inline]
pub fn add_tone_to_char(c: char, tone: u8) -> char {
    find_vowel_position(c)
        .and_then(|pos| {
            let new_pos = pos as isize + (tone as isize - (pos % 6) as isize);
            VOWELS.get(new_pos as usize).copied()
        })
        .unwrap_or(c)
}

/// Adds a diacritic mark to a toneless character.
#[inline]
pub fn add_mark_to_toneless_char(c: char, mark: u8) -> char {
    MARKS_MAPS
        .get(&c)
        .and_then(|arr| arr.get(mark as usize))
        .copied()
        .filter(|&ch| ch != '_')
        .unwrap_or(c)
}

/// Adds a diacritic mark to a character while preserving its current tone.
#[inline]
pub fn add_mark_to_char(c: char, mark: u8) -> char {
    let tone = find_tone_from_char(c);
    let base = add_tone_to_char(c, 0);
    let marked = add_mark_to_toneless_char(base, mark);
    add_tone_to_char(marked, tone)
}

/// Returns true if the character is a Vietnamese vowel with a tone mark
/// or a diacritic mark.
#[inline]
pub fn is_vietnamese_rune(c: char) -> bool {
    find_tone_from_char(c) != 0 || c != add_mark_to_toneless_char(c, 0)
}

/// Returns true if the word contains at least one Vietnamese-specific character.
#[allow(unused)]
#[inline]
pub fn has_any_vietnamese_rune(word: &str) -> bool {
    word.chars().any(|c| {
        is_vietnamese_rune(if c.is_ascii() {
            c.to_ascii_lowercase()
        } else {
            c.to_lowercase().next().unwrap_or(c)
        })
    })
}

/// Returns true if the word contains at least one Vietnamese vowel.
#[allow(unused)]
#[inline]
pub fn has_any_vietnamese_vowel(word: &str) -> bool {
    word.chars().any(|c| {
        is_vowel(if c.is_ascii() {
            c.to_ascii_lowercase()
        } else {
            c.to_lowercase().next().unwrap_or(c)
        })
    })
}
