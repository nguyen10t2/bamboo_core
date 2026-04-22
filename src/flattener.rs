//! Functions for converting a sequence of transformations into a final string.

use crate::engine::Transformation;
use crate::input_method::{EffectType, Mark};
use crate::mode::OutputOptions;
use crate::utils::{add_mark_to_char, add_tone_to_char};

const MAX_COMPOSITION: usize = 32;

#[inline]
fn lower(c: char) -> char {
    if c.is_ascii() { c.to_ascii_lowercase() } else { c.to_lowercase().next().unwrap_or(c) }
}

#[inline]
fn upper(c: char) -> char {
    if c.is_ascii() { c.to_ascii_uppercase() } else { c.to_uppercase().next().unwrap_or(c) }
}

/// Converts a slice of transformations into a string based on the provided options.
pub(crate) fn flatten_slice(composition: &[Transformation], options: OutputOptions) -> String {
    let mut out = String::with_capacity(estimate_cap_bytes_slice(composition, options));
    write_canvas_slice(composition, options, &mut out);
    out
}

/// Similar to [`flatten_slice`], but writes the result into an existing string buffer.
pub(crate) fn flatten_slice_into(
    composition: &[Transformation],
    options: OutputOptions,
    out: &mut String,
) {
    out.clear();
    out.reserve(estimate_cap_bytes_slice(composition, options));
    write_canvas_slice(composition, options, out);
}

#[inline]
fn estimate_cap_bytes_slice(composition: &[Transformation], options: OutputOptions) -> usize {
    let char_count = if options.contains(OutputOptions::RAW) {
        composition.iter().filter(|t| t.rule.key != '\0').count()
    } else {
        composition
            .iter()
            .filter(|t| t.rule.effect_type == EffectType::Appending && t.rule.key != '\0')
            .count()
    };
    char_count * 4
}

fn write_canvas_slice(composition: &[Transformation], options: OutputOptions, out: &mut String) {
    if composition.is_empty() {
        return;
    }

    let len = composition.len();
    debug_assert!(len <= MAX_COMPOSITION, "composition too long for stack canvas: {len}");
    if len > MAX_COMPOSITION {
        return;
    }

    let mut next_effect: [Option<usize>; MAX_COMPOSITION] = [None; MAX_COMPOSITION];
    let mut head_effect: [Option<usize>; MAX_COMPOSITION] = [None; MAX_COMPOSITION];
    let mut appending_idxs = [0usize; MAX_COMPOSITION];
    let mut appending_len = 0usize;

    for (idx, trans) in composition.iter().enumerate() {
        if (options.contains(OutputOptions::RAW) || trans.rule.effect_type == EffectType::Appending)
            && trans.rule.key != '\0'
        {
            appending_idxs[appending_len] = idx;
            appending_len += 1;
        } else if let Some(target) = trans.target
            && target < len
        {
            next_effect[idx] = head_effect[target];
            head_effect[target] = Some(idx);
        }
    }

    for &abs_idx in appending_idxs.iter().take(appending_len) {
        let appending_trans = &composition[abs_idx];

        let mut chr: char;
        if options.contains(OutputOptions::RAW) {
            chr = appending_trans.rule.key;
        } else {
            chr = appending_trans.rule.effect_on;

            let mut curr = head_effect[abs_idx];
            let mut effects = [None; 8];
            let mut count = 0;
            while let Some(idx) = curr {
                if count < 8 {
                    effects[count] = Some(idx);
                    count += 1;
                }
                curr = next_effect[idx];
            }

            for i in (0..count).rev() {
                let t = &composition[effects[i].unwrap()];
                match t.rule.effect_type {
                    EffectType::MarkTransformation => {
                        if t.rule.effect == Mark::Raw as u8 {
                            chr = appending_trans.rule.key;
                        } else {
                            chr = add_mark_to_char(chr, t.rule.effect);
                        }
                    }
                    EffectType::ToneTransformation => {
                        chr = add_tone_to_char(chr, t.rule.effect);
                    }
                    _ => {}
                }
            }
        }

        if options.contains(OutputOptions::TONE_LESS) {
            chr = add_tone_to_char(chr, 0);
        }
        if options.contains(OutputOptions::MARK_LESS) {
            chr = crate::utils::add_mark_to_toneless_char(add_tone_to_char(chr, 0), 0);
        }

        if options.contains(OutputOptions::LOWER_CASE) {
            out.push(lower(chr));
        } else if appending_trans.is_upper_case {
            out.push(upper(chr));
        } else {
            out.push(chr);
        }
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
            if trans.rule.key == '\0' {
                continue;
            }
            first = Some((abs_idx, trans));
            break;
        }
        if trans.rule.effect_type == EffectType::Appending && trans.rule.key != '\0' {
            first = Some((abs_idx, trans));
            break;
        }
    }

    let (target_abs_idx, appending_trans) = first?;
    let mut chr = if options.contains(OutputOptions::RAW) {
        appending_trans.rule.key
    } else {
        let mut c = appending_trans.rule.effect_on;
        for trans in &composition[start..] {
            if trans.target != Some(target_abs_idx) {
                continue;
            }
            match trans.rule.effect_type {
                EffectType::MarkTransformation => {
                    if trans.rule.effect == Mark::Raw as u8 {
                        c = appending_trans.rule.key;
                    } else {
                        c = add_mark_to_char(c, trans.rule.effect);
                    }
                }
                EffectType::ToneTransformation => {
                    c = add_tone_to_char(c, trans.rule.effect);
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
        chr = crate::utils::add_mark_to_toneless_char(add_tone_to_char(chr, 0), 0);
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
    use crate::input_method::{EffectType, Rule};

    #[test]
    fn first_canvas_char_in_suffix_handles_offsets() {
        let t1 = Transformation {
            rule: Rule {
                key: 'a',
                effect: 0,
                effect_type: EffectType::Appending,
                effect_on: 'a',
                result: 'a',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: None,
            is_upper_case: false,
        };
        let t2 = Transformation {
            rule: Rule {
                key: ' ',
                effect: 0,
                effect_type: EffectType::Appending,
                effect_on: ' ',
                result: ' ',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: None,
            is_upper_case: false,
        };
        let t3 = Transformation {
            rule: Rule {
                key: 'w',
                effect: 0,
                effect_type: EffectType::Appending,
                effect_on: 'w',
                result: 'w',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: None,
            is_upper_case: false,
        };
        let comp = vec![t1, t2, t3];
        assert_eq!(first_canvas_char_in_suffix(&comp, 1, OutputOptions::RAW), Some(' '));
        assert_eq!(first_canvas_char_in_suffix(&comp, 2, OutputOptions::NONE), Some('w'));
    }

    #[test]
    fn first_canvas_char_in_suffix_resolves_absolute_targets() {
        let x = Transformation {
            rule: Rule {
                key: 'x',
                effect: 0,
                effect_type: EffectType::Appending,
                effect_on: 'x',
                result: 'x',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: None,
            is_upper_case: false,
        };
        let o = Transformation {
            rule: Rule {
                key: 'o',
                effect: 0,
                effect_type: EffectType::Appending,
                effect_on: 'o',
                result: 'o',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: None,
            is_upper_case: false,
        };
        let mark_hat = Transformation {
            rule: Rule {
                key: 'o',
                effect: 1, // Mark::Hat
                effect_type: EffectType::MarkTransformation,
                effect_on: 'o',
                result: 'ô',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: Some(1),
            is_upper_case: false,
        };
        let comp = vec![x, o, mark_hat];
        assert_eq!(first_canvas_char_in_suffix(&comp, 1, OutputOptions::NONE), Some('ô'));
    }

    #[test]
    fn flatten_applies_mark_and_tone_in_order() {
        let o = Transformation {
            rule: Rule {
                key: 'o',
                effect: 0,
                effect_type: EffectType::Appending,
                effect_on: 'o',
                result: 'o',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: None,
            is_upper_case: false,
        };
        let hat = Transformation {
            rule: Rule {
                key: 'o',
                effect: 1, // Mark::Hat
                effect_type: EffectType::MarkTransformation,
                effect_on: 'o',
                result: 'ô',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: Some(0),
            is_upper_case: false,
        };
        let acute = Transformation {
            rule: Rule {
                key: 's',
                effect: 2, // Tone::Acute
                effect_type: EffectType::ToneTransformation,
                effect_on: '\0',
                result: '\0',
                appended: ['\0'; 2],
                appended_len: 0,
            },
            target: Some(0),
            is_upper_case: false,
        };
        let comp = vec![o, hat, acute];
        assert_eq!(flatten_slice(&comp, OutputOptions::NONE), "ố");
    }
}
