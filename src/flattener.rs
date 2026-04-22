use crate::engine::Transformation;
use crate::input_method::{EffectType, Mark};
use crate::mode::OutputOptions;
use crate::utils::{add_mark_to_char, add_tone_to_char};

#[inline]
fn lower(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

#[inline]
fn upper(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_uppercase()
    } else {
        c.to_uppercase().next().unwrap_or(c)
    }
}

/// Flattens a composition of transformation references into a single string.
///
/// # Arguments
///
/// * `composition` - A slice of references to [`Transformation`] objects.
/// * `options` - [`OutputOptions`] to customize the resulting string.
///
/// # Returns
///
/// A `String` representing the processed text.
pub fn flatten(
    composition: &[&Transformation],
    options: OutputOptions,
) -> String {
    let mut out =
        String::with_capacity(estimate_len_refs(composition, options));
    write_canvas_refs(composition, options, &mut out);
    out
}

/// Flattens a slice of transformation objects into a single string.
///
/// # Arguments
///
/// * `composition` - A slice of [`Transformation`] objects.
/// * `options` - [`OutputOptions`] to customize the resulting string.
///
/// # Returns
///
/// A `String` representing the processed text.
pub(crate) fn flatten_slice(
    composition: &[Transformation],
    options: OutputOptions,
) -> String {
    let mut out =
        String::with_capacity(estimate_len_slice(composition, options));
    write_canvas_slice(composition, options, &mut out);
    out
}

fn estimate_len_refs(
    composition: &[&Transformation],
    options: OutputOptions,
) -> usize {
    if options.contains(OutputOptions::RAW) {
        composition.iter().filter(|t| t.rule.key != '\0').count()
    } else {
        composition
            .iter()
            .filter(|t| {
                t.rule.effect_type == EffectType::Appending
                    && t.rule.key != '\0'
            })
            .count()
    }
}

fn estimate_len_slice(
    composition: &[Transformation],
    options: OutputOptions,
) -> usize {
    if options.contains(OutputOptions::RAW) {
        composition.iter().filter(|t| t.rule.key != '\0').count()
    } else {
        composition
            .iter()
            .filter(|t| {
                t.rule.effect_type == EffectType::Appending
                    && t.rule.key != '\0'
            })
            .count()
    }
}

fn write_canvas_refs(
    composition: &[&Transformation],
    options: OutputOptions,
    out: &mut String,
) {
    if composition.is_empty() {
        return;
    }

    // linked-list implementation in a flat array to avoid Vec<Vec>
    // next_effect[i] stores the index of the next transformation targeting the same character
    let mut next_effect = vec![None; composition.len()];
    let mut head_effect = vec![None; composition.len()];
    let mut appending_list = Vec::with_capacity(composition.len());

    for (idx, trans) in composition.iter().enumerate() {
        if (options.contains(OutputOptions::RAW)
            || trans.rule.effect_type == EffectType::Appending)
            && trans.rule.key != '\0'
        {
            appending_list.push((idx, *trans));
        } else if let Some(target) = trans.target
            && target < head_effect.len()
        {
            next_effect[idx] = head_effect[target];
            head_effect[target] = Some(idx);
        }
    }

    for (abs_idx, appending_trans) in appending_list {
        let mut chr: char;
        if options.contains(OutputOptions::RAW) {
            chr = appending_trans.rule.key;
        } else {
            chr = appending_trans.rule.effect_on;
            // Iterate through effects targeting this character (in reverse order because of linked list)
            let mut curr = head_effect[abs_idx];
            let mut effects = [None; 8]; // Maximum 8 effects per character is plenty
            let mut count = 0;
            while let Some(idx) = curr {
                if count < 8 {
                    effects[count] = Some(idx);
                    count += 1;
                }
                curr = next_effect[idx];
            }

            // Apply in original order (effects were added to linked list in original order,
            // so head_effect points to the LAST effect). We iterate backwards.
            for i in (0..count).rev() {
                let t = composition[effects[i].unwrap()];
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
            chr = add_mark_to_char(chr, 0);
        }
        if options.contains(OutputOptions::LOWER_CASE) {
            chr = lower(chr);
        } else if appending_trans.is_upper_case {
            chr = upper(chr);
        }
        out.push(chr);
    }
}

fn write_canvas_slice(
    composition: &[Transformation],
    options: OutputOptions,
    out: &mut String,
) {
    if composition.is_empty() {
        return;
    }

    let mut next_effect = vec![None; composition.len()];
    let mut head_effect = vec![None; composition.len()];
    let mut appending_list = Vec::with_capacity(composition.len());

    for (idx, trans) in composition.iter().enumerate() {
        if (options.contains(OutputOptions::RAW)
            || trans.rule.effect_type == EffectType::Appending)
            && trans.rule.key != '\0'
        {
            appending_list.push((idx, trans));
        } else if let Some(target) = trans.target
            && target < head_effect.len()
        {
            next_effect[idx] = head_effect[target];
            head_effect[target] = Some(idx);
        }
    }

    for (abs_idx, appending_trans) in appending_list {
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
            chr = add_mark_to_char(chr, 0);
        }
        if options.contains(OutputOptions::LOWER_CASE) {
            chr = lower(chr);
        } else if appending_trans.is_upper_case {
            chr = upper(chr);
        }
        out.push(chr);
    }
}

/// Finds the first character that would be visible in the output starting from a specific index in the composition.
///
/// This function resolves the character by applying all relevant transformations that target it
/// within the suffix starting at `start`.
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
        if trans.rule.effect_type == EffectType::Appending
            && trans.rule.key != '\0'
        {
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
        chr = add_mark_to_char(chr, 0);
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
                appended_rules: Box::default(),
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
                appended_rules: Box::default(),
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
                appended_rules: Box::default(),
            },
            target: None,
            is_upper_case: false,
        };
        let comp = vec![t1, t2, t3];
        assert_eq!(
            first_canvas_char_in_suffix(&comp, 1, OutputOptions::RAW),
            Some(' ')
        );
        assert_eq!(
            first_canvas_char_in_suffix(&comp, 2, OutputOptions::NONE),
            Some('w')
        );
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
                appended_rules: Box::default(),
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
                appended_rules: Box::default(),
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
                appended_rules: Box::default(),
            },
            target: Some(1),
            is_upper_case: false,
        };
        let comp = vec![x, o, mark_hat];
        assert_eq!(
            first_canvas_char_in_suffix(&comp, 1, OutputOptions::NONE),
            Some('ô')
        );
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
                appended_rules: Box::default(),
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
                appended_rules: Box::default(),
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
                appended_rules: Box::default(),
            },
            target: Some(0),
            is_upper_case: false,
        };
        let comp = vec![o, hat, acute];
        assert_eq!(flatten_slice(&comp, OutputOptions::NONE), "ố");
    }
}
