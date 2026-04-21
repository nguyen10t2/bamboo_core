use crate::bamboo::Transformation;
use crate::bamboo::{ENGLISH_MODE, LOWER_CASE, MARK_LESS, Mode, TONE_LESS};
use crate::rules_parser::{EffectType, Mark};
use crate::utils::{add_mark_to_char, add_tone_to_char};

#[inline]
fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

#[inline]
fn upper(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

pub fn flatten(composition: &[&Transformation], mode: Mode) -> String {
    get_canvas(composition, mode).into_iter().collect()
}

pub(crate) fn get_canvas(
    composition: &[&Transformation],
    mode: Mode,
) -> Vec<char> {
    let mut canvas: Vec<char> = Vec::new();

    let mut effects_by_target: Vec<Vec<&Transformation>> =
        vec![Vec::new(); composition.len()];
    let mut appending_list: Vec<(usize, &Transformation)> = Vec::new();

    for (idx, trans) in composition.iter().enumerate() {
        if mode.contains(ENGLISH_MODE) {
            if trans.rule.key == '\0' {
                continue; // ignore virtual key in raw output
            }
            appending_list.push((idx, *trans));
        } else if trans.rule.effect_type == EffectType::Appending {
            if trans.rule.key == '\0' {
                continue; // ignore virtual appending key
            }
            appending_list.push((idx, *trans));
        } else if let Some(target) = trans.target
            && target < effects_by_target.len()
        {
            effects_by_target[target].push(*trans);
        }
    }

    for (idx, appending_trans) in appending_list {
        let mut chr: char;
        let trans_list: &[&Transformation] =
            effects_by_target.get(idx).map(Vec::as_slice).unwrap_or(&[]);

        if mode.contains(ENGLISH_MODE) {
            chr = appending_trans.rule.key;
        } else {
            chr = appending_trans.rule.effect_on;
            for trans in trans_list {
                match trans.rule.effect_type {
                    EffectType::MarkTransformation => {
                        if trans.rule.effect == Mark::Raw as u8 {
                            chr = appending_trans.rule.key;
                        } else {
                            chr = add_mark_to_char(chr, trans.rule.effect);
                        }
                    }
                    EffectType::ToneTransformation => {
                        chr = add_tone_to_char(chr, trans.rule.effect);
                    }
                    _ => {}
                }
            }
        }

        if mode.contains(TONE_LESS) {
            chr = add_tone_to_char(chr, 0);
        }
        if mode.contains(MARK_LESS) {
            chr = add_mark_to_char(chr, 0);
        }
        if mode.contains(LOWER_CASE) {
            chr = lower(chr);
        } else if appending_trans.is_upper_case {
            chr = upper(chr);
        }

        canvas.push(chr);
    }

    canvas
}

pub(crate) fn first_canvas_char_in_suffix(
    composition: &[Transformation],
    start: usize,
    mode: Mode,
) -> Option<char> {
    let mut first: Option<(usize, &Transformation)> = None;
    for (idx, trans) in composition[start..].iter().enumerate() {
        let abs_idx = start + idx;
        if mode.contains(ENGLISH_MODE) {
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
    let mut chr = if mode.contains(ENGLISH_MODE) {
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

    if mode.contains(TONE_LESS) {
        chr = add_tone_to_char(chr, 0);
    }
    if mode.contains(MARK_LESS) {
        chr = add_mark_to_char(chr, 0);
    }
    if mode.contains(LOWER_CASE) {
        chr = lower(chr);
    } else if appending_trans.is_upper_case {
        chr = upper(chr);
    }

    Some(chr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bamboo::VIETNAMESE_MODE;
    use crate::rules_parser::{EffectType, Rule};

    #[test]
    fn first_canvas_char_in_suffix_handles_offsets() {
        let t1 = Transformation {
            rule: Rule {
                key: 'a',
                effect: 0,
                effect_type: EffectType::Appending,
                effect_on: 'a',
                result: 'a',
                appended_rules: Vec::new(),
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
                appended_rules: Vec::new(),
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
                appended_rules: Vec::new(),
            },
            target: None,
            is_upper_case: false,
        };
        let comp = vec![t1, t2, t3];
        assert_eq!(first_canvas_char_in_suffix(&comp, 1, ENGLISH_MODE), Some(' '));
        assert_eq!(first_canvas_char_in_suffix(&comp, 2, VIETNAMESE_MODE), Some('w'));
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
                appended_rules: Vec::new(),
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
                appended_rules: Vec::new(),
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
                appended_rules: Vec::new(),
            },
            target: Some(1),
            is_upper_case: false,
        };
        let comp = vec![x, o, mark_hat];
        assert_eq!(first_canvas_char_in_suffix(&comp, 1, VIETNAMESE_MODE), Some('ô'));
    }
}
