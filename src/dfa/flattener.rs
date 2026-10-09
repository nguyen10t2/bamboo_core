//! Functions for converting a sequence of transformations into a final string.

use crate::engine::{MAX_ACTIVE_TRANS, Transformation};
use crate::input_method::{EffectType, Mark};
use crate::mode::OutputOptions;
use crate::phonetics::{add_mark_to_char, add_tone_to_char, lower, upper};

/// Converts a slice of transformations into a string based on the provided options.
pub(crate) fn flatten_slice(composition: &[Transformation], options: OutputOptions) -> String {
    let mut out = String::with_capacity(estimate_cap_bytes_slice(composition, options));
    write_canvas_slice(composition, options, &mut out);
    out
}

/// Appends the flattened composition to an existing string buffer without clearing it.
/// Used by `commit()` to avoid allocating a temporary String.
pub(crate) fn append_flatten_slice(
    composition: &[Transformation],
    options: OutputOptions,
    out: &mut String,
) {
    out.reserve(estimate_cap_bytes_slice(composition, options));
    write_canvas_slice(composition, options, out);
}

/// Destination for flattened characters. The generic canvas below
/// monomorphizes per sink, so the hot [`String`] path codegens exactly like
/// the direct `push` it replaces.
pub(crate) trait CanvasSink {
    fn push_char(&mut self, c: char);
}

impl CanvasSink for String {
    #[inline(always)]
    fn push_char(&mut self, c: char) {
        self.push(c);
    }
}

impl CanvasSink for Vec<u8> {
    #[inline(always)]
    fn push_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        self.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    }
}

/// Appends the flattened composition as UTF-8 bytes without clearing `out`.
/// Used by [`Dfa::add_state`](crate::dfa::Dfa::add_state) to fill the flat
/// arena directly instead of round-tripping through a temporary `String`
/// (one allocation plus a copy per new JIT state).
pub(crate) fn append_flatten_bytes(
    composition: &[Transformation],
    options: OutputOptions,
    out: &mut Vec<u8>,
) {
    out.reserve(estimate_cap_bytes_slice(composition, options));
    write_canvas_slice(composition, options, out);
}

/// Same output as flattening with [`OutputOptions::RAW`], without the canvas
/// setup, since `commit()` runs it on every word.
pub(crate) fn append_raw_keys(composition: &[Transformation], out: &mut String) {
    for t in composition.iter().filter(|t| t.key != '\0') {
        out.push(if t.is_upper_case { upper(t.key) } else { t.key });
    }
}

/// Appends already flattened text, applying the per-character options the
/// flattener would have applied (committed words keep no transformations).
pub(crate) fn append_text_with_options(text: &str, options: OutputOptions, out: &mut String) {
    if options.is_empty() {
        out.push_str(text);
        return;
    }
    for c in text.chars() {
        // Tone and mark tables are lowercase-only.
        let mut chr = lower(c);
        if options.contains(OutputOptions::TONE_LESS) {
            chr = add_tone_to_char(chr, 0);
        }
        if options.contains(OutputOptions::MARK_LESS) {
            chr = crate::phonetics::add_mark_to_toneless_char(add_tone_to_char(chr, 0), 0);
        }
        let keep_upper = !options.contains(OutputOptions::LOWER_CASE) && c != lower(c);
        out.push(if keep_upper { upper(chr) } else { chr });
    }
}

#[inline]
fn estimate_cap_bytes_slice(composition: &[Transformation], options: OutputOptions) -> usize {
    let char_count = if options.contains(OutputOptions::RAW) {
        composition.iter().filter(|t| t.key != '\0').count()
    } else {
        composition
            .iter()
            .filter(|t| t.effect_type == EffectType::Appending && t.key != '\0')
            .count()
    };
    char_count * 4
}

fn write_canvas_slice<S: CanvasSink>(
    composition: &[Transformation],
    options: OutputOptions,
    out: &mut S,
) {
    if composition.is_empty() {
        return;
    }

    let len = composition.len();
    debug_assert!(len <= MAX_ACTIVE_TRANS, "composition too long for stack canvas: {len}");
    if len > MAX_ACTIVE_TRANS {
        return;
    }

    const NO_EFFECT: u8 = 0xFF;
    let mut next_effect = [NO_EFFECT; MAX_ACTIVE_TRANS];
    let mut head_effect = [NO_EFFECT; MAX_ACTIVE_TRANS];
    let mut appending_idxs = [0u8; MAX_ACTIVE_TRANS];
    let mut appending_len = 0usize;

    for (idx, trans) in composition.iter().enumerate() {
        if (options.contains(OutputOptions::RAW) || trans.effect_type == EffectType::Appending)
            && trans.key != '\0'
        {
            if appending_len < MAX_ACTIVE_TRANS {
                appending_idxs[appending_len] = idx as u8;
                appending_len += 1;
            }
        } else if let Some(target) = trans.target()
            && (target as usize) < len
        {
            next_effect[idx] = head_effect[target as usize];
            head_effect[target as usize] = idx as u8;
        }
    }

    for &abs_u8 in appending_idxs.iter().take(appending_len) {
        let abs_idx = abs_u8 as usize;
        let appending_trans = &composition[abs_idx];

        let mut chr: char;
        if options.contains(OutputOptions::RAW) {
            chr = appending_trans.key;
        } else {
            chr = appending_trans.effect_on;

            let mut curr = head_effect[abs_idx];
            let mut effects = [0u8; MAX_ACTIVE_TRANS];
            let mut count = 0;
            while curr != NO_EFFECT {
                if count >= MAX_ACTIVE_TRANS {
                    debug_assert!(
                        false,
                        "flattener: effect chain exceeded MAX_ACTIVE_TRANS — possible cycle"
                    );
                    break;
                }
                effects[count] = curr;
                count += 1;
                curr = next_effect[curr as usize];
            }

            for &eff_idx in effects[..count].iter().rev() {
                let t = &composition[eff_idx as usize];

                match t.effect_type {
                    EffectType::MarkTransformation => {
                        if t.effect == Mark::Raw as u8 {
                            chr = appending_trans.key;
                        } else {
                            chr = add_mark_to_char(chr, t.effect);
                        }
                    }
                    EffectType::ToneTransformation => {
                        chr = add_tone_to_char(chr, t.effect);
                    }
                    _ => {}
                }
            }
        }

        if options.contains(OutputOptions::TONE_LESS) {
            chr = add_tone_to_char(chr, 0);
        }
        if options.contains(OutputOptions::MARK_LESS) {
            chr = crate::phonetics::add_mark_to_toneless_char(add_tone_to_char(chr, 0), 0);
        }

        let final_chr = if options.contains(OutputOptions::LOWER_CASE) {
            lower(chr)
        } else if appending_trans.is_upper_case {
            upper(chr)
        } else {
            chr
        };
        out.push_char(final_chr);
    }
}

pub(crate) fn first_canvas_char_in_suffix(
    composition: &[Transformation],
    start: usize,
    options: OutputOptions,
) -> Option<char> {
    let mut first: Option<(usize, &Transformation)> = None;
    for (idx, trans) in composition[start..].iter().enumerate() {
        let abs_idx = start + idx;
        if options.contains(OutputOptions::RAW) {
            if trans.key == '\0' {
                continue;
            }
            first = Some((abs_idx, trans));
            break;
        }
        if trans.effect_type == EffectType::Appending && trans.key != '\0' {
            first = Some((abs_idx, trans));
            break;
        }
    }

    let (target_abs_idx, appending_trans) = first?;
    let mut chr = if options.contains(OutputOptions::RAW) {
        appending_trans.key
    } else {
        let mut c = appending_trans.effect_on;
        for trans in &composition[start..] {
            if trans.target() != Some(target_abs_idx as u8) {
                continue;
            }
            match trans.effect_type {
                EffectType::MarkTransformation => {
                    if trans.effect == Mark::Raw as u8 {
                        c = appending_trans.key;
                    } else {
                        c = add_mark_to_char(c, trans.effect);
                    }
                }
                EffectType::ToneTransformation => {
                    c = add_tone_to_char(c, trans.effect);
                }
                _ => {}
            }
        }
        c
    };

    if options.contains(OutputOptions::TONE_LESS) {
        chr = add_tone_to_char(chr, 0);
    }
    if options.contains(OutputOptions::MARK_LESS) {
        chr = crate::phonetics::add_mark_to_toneless_char(add_tone_to_char(chr, 0), 0);
    }
    if options.contains(OutputOptions::LOWER_CASE) {
        chr = lower(chr);
    } else if appending_trans.is_upper_case {
        chr = upper(chr);
    }

    Some(chr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_method::EffectType;

    #[test]
    fn first_canvas_char_in_suffix_handles_offsets() {
        let t1 = Transformation::new('a', 'a', 'a', None, 0, EffectType::Appending, false);
        let t2 = Transformation::new(' ', ' ', ' ', None, 0, EffectType::Appending, false);
        let t3 = Transformation::new('w', 'w', 'w', None, 0, EffectType::Appending, false);
        let comp = vec![t1, t2, t3];
        assert_eq!(first_canvas_char_in_suffix(&comp, 1, OutputOptions::RAW), Some(' '));
        assert_eq!(first_canvas_char_in_suffix(&comp, 2, OutputOptions::NONE), Some('w'));
    }

    #[test]
    fn first_canvas_char_in_suffix_resolves_absolute_targets() {
        let x = Transformation::new('x', 'x', 'x', None, 0, EffectType::Appending, false);
        let o = Transformation::new('o', 'o', 'o', None, 0, EffectType::Appending, false);
        let mark_hat =
            Transformation::new('o', 'o', 'ô', Some(1), 1, EffectType::MarkTransformation, false);
        let comp = vec![x, o, mark_hat];
        assert_eq!(first_canvas_char_in_suffix(&comp, 1, OutputOptions::NONE), Some('ô'));
    }

    #[test]
    fn flatten_applies_mark_and_tone_in_order() {
        let o = Transformation::new('o', 'o', 'o', None, 0, EffectType::Appending, false);
        let hat =
            Transformation::new('o', 'o', 'ô', Some(0), 1, EffectType::MarkTransformation, false);
        let acute =
            Transformation::new('s', '\0', '\0', Some(0), 2, EffectType::ToneTransformation, false);
        let comp = vec![o, hat, acute];
        assert_eq!(flatten_slice(&comp, OutputOptions::NONE), "ố");
    }

    #[test]
    fn byte_sink_matches_string_sink() {
        let o = Transformation::new('o', 'o', 'o', None, 0, EffectType::Appending, true);
        let hat =
            Transformation::new('o', 'o', 'ô', Some(0), 1, EffectType::MarkTransformation, false);
        let acute =
            Transformation::new('s', '\0', '\0', Some(0), 2, EffectType::ToneTransformation, false);
        let comp = vec![o, hat, acute];
        let options = [
            OutputOptions::NONE,
            OutputOptions::RAW,
            OutputOptions::LOWER_CASE,
            OutputOptions::TONE_LESS,
            OutputOptions::MARK_LESS,
            OutputOptions::FULL_TEXT,
        ];
        for option in options {
            let mut bytes = Vec::new();
            append_flatten_bytes(&comp, option, &mut bytes);
            assert_eq!(bytes, flatten_slice(&comp, option).as_bytes(), "{option:?}");
        }
    }
}
