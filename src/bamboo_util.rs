use crate::engine::Transformation;
use crate::flattener::flatten;
use crate::input_method::{EffectType, Mark, Rule, Tone};
use crate::mode::OutputOptions;
use crate::spelling::is_valid_cvc;
use crate::utils::{
    add_mark_to_char, add_tone_to_char, is_alpha, is_space, is_vowel,
};

/// Flag to enable free tone marking (allows placing tones at any time).
pub(crate) const EFREE_TONE_MARKING: u32 = 1 << 0;
/// Flag to use the standard Vietnamese tone style (e.g., placing tone on the second vowel in some cases).
pub(crate) const ESTD_TONE_STYLE: u32 = 1 << 1;

#[inline]
fn lower(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

#[inline]
fn is_upper(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_uppercase() } else { lower(c) != c }
}

fn in_key_list(keys: Option<&[char]>, key: char) -> bool {
    keys.map(|ks| ks.contains(&key)).unwrap_or(false)
}

/// Finds the index of the last transformation that resulted in an appended character.
pub(crate) fn find_last_appending_trans_idx(
    composition: &[Transformation],
) -> Option<usize> {
    composition
        .iter()
        .enumerate()
        .rev()
        .find(|(_, trans)| trans.rule.effect_type == EffectType::Appending)
        .map(|(idx, _)| idx)
}

/// Finds the last transformation in the composition that resulted in an appended character.
pub(crate) fn find_last_appending_trans<'a>(
    composition: &'a [&'a Transformation],
) -> Option<&'a Transformation> {
    composition
        .iter()
        .rev()
        .copied()
        .find(|trans| trans.rule.effect_type == EffectType::Appending)
}

/// Splits the composition into two parts: everything before the last syllable and the last syllable itself.
pub(crate) fn extract_last_syllable<'a>(
    composition: &'a [Transformation],
    _keys: Option<&[char]>,
) -> (Vec<&'a Transformation>, Vec<&'a Transformation>) {
    let mut idx = composition.len();
    let mut last_is_vowel = false;
    let mut found_vowel = false;

    while idx > 0 {
        let tmp = &composition[idx - 1];
        if tmp.target.is_none() {
            let is_v = is_vowel(tmp.rule.result);
            if found_vowel && !is_v && !last_is_vowel {
                break;
            }
            if is_v {
                found_vowel = true;
            }
            last_is_vowel = is_v;
        }
        idx -= 1;
    }

    (composition[..idx].iter().collect(), composition[idx..].iter().collect())
}

/// Creates a new transformation that simply appends a character.
pub(crate) fn new_appending_trans(
    key: char,
    is_upper_case: bool,
) -> Transformation {
    Transformation {
        is_upper_case,
        target: None,
        rule: Rule {
            key,
            effect_on: key,
            effect: 0,
            effect_type: EffectType::Appending,
            result: key,
            appended_rules: Box::default(),
        },
    }
}

/// Generates an appending transformation based on the provided rules and key.
pub(crate) fn generate_appending_trans(
    rules: &[Rule],
    lower_key: char,
    is_upper_case: bool,
) -> Transformation {
    for rule in rules {
        if rule.key == lower_key && rule.effect_type == EffectType::Appending {
            let mut rule = rule.clone();
            let mut _is_upper_case = is_upper_case || is_upper(rule.effect_on);
            rule.effect_on = lower(rule.effect_on);
            rule.result = rule.effect_on;
            _is_upper_case |= is_upper(rule.effect_on);
            return Transformation {
                is_upper_case: _is_upper_case,
                target: None,
                rule,
            };
        }
    }

    new_appending_trans(lower_key, is_upper_case)
}

fn filter_appending_composition<'a>(
    composition: &'a [&'a Transformation],
) -> Vec<&'a Transformation> {
    composition
        .iter()
        .copied()
        .filter(|t| t.rule.effect_type == EffectType::Appending)
        .collect()
}

fn find_root_target(
    composition: &[&Transformation],
    mut target: usize,
) -> usize {
    while let Some(t) = composition[target].target {
        target = t;
    }
    target
}

#[inline]
fn idx_of(
    composition: &[&Transformation],
    needle: &Transformation,
) -> Option<usize> {
    composition.iter().position(|t| std::ptr::eq(*t, needle))
}

/// Checks if the current composition represents a valid Vietnamese syllable.
pub(crate) fn is_valid(
    composition: &[&Transformation],
    input_is_full_complete: bool,
) -> bool {
    if composition.len() <= 1 {
        return true;
    }

    // last tone checking
    for trans in composition.iter().rev() {
        if trans.rule.effect_type == EffectType::ToneTransformation {
            let last_tone = match trans.rule.effect {
                1 => Tone::Grave,
                2 => Tone::Acute,
                3 => Tone::Hook,
                4 => Tone::Tilde,
                5 => Tone::Dot,
                _ => Tone::None,
            };
            if !has_valid_tone(composition, last_tone) {
                return false;
            }
            break;
        }
    }

    // spell checking
    let (fc, vo, lc) = extract_cvc_trans(composition);
    let flatten_mode = OutputOptions::NONE
        | OutputOptions::LOWER_CASE
        | OutputOptions::TONE_LESS;
    is_valid_cvc(
        &flatten(&fc, flatten_mode),
        &flatten(&vo, flatten_mode),
        &flatten(&lc, flatten_mode),
        input_is_full_complete,
    )
}

fn get_right_most_vowels<'a>(
    composition: &'a [&'a Transformation],
) -> Vec<&'a Transformation> {
    let (_, vo, _) = extract_cvc_trans(composition);
    vo
}

fn find_tone_target(
    composition: &[&Transformation],
    std_style: bool,
) -> Option<usize> {
    if composition.is_empty() {
        return None;
    }

    let (_, vo, lc) = extract_cvc_trans(composition);
    let vowels = filter_appending_composition(&vo);

    if vowels.len() == 1 {
        return idx_of(composition, vowels[0]);
    }

    if vowels.len() == 2 && std_style {
        let mut target: Option<usize> = None;
        for trans in &vo {
            if trans.rule.result == 'ơ' || trans.rule.result == 'ê' {
                target = Some(trans.target.unwrap_or_else(|| {
                    idx_of(composition, trans).unwrap_or(0)
                }));
            }
        }
        if target.is_none() {
            target = Some(if !lc.is_empty() {
                idx_of(composition, vowels[1]).unwrap_or(0)
            } else {
                idx_of(composition, vowels[0]).unwrap_or(0)
            });
        }
        return target;
    }

    if vowels.len() == 2 {
        if !lc.is_empty() {
            return idx_of(composition, vowels[1]);
        }

        let s = flatten(
            &vowels,
            OutputOptions::RAW
                | OutputOptions::LOWER_CASE
                | OutputOptions::TONE_LESS
                | OutputOptions::MARK_LESS,
        );
        return Some(
            if matches!(s.as_str(), "oa" | "oe" | "uy" | "ue" | "uo") {
                idx_of(composition, vowels[1]).unwrap_or(0)
            } else {
                idx_of(composition, vowels[0]).unwrap_or(0)
            },
        );
    }

    if vowels.len() == 3 {
        return Some(
            if flatten(
                &vowels,
                OutputOptions::RAW
                    | OutputOptions::LOWER_CASE
                    | OutputOptions::TONE_LESS
                    | OutputOptions::MARK_LESS,
            ) == "uye"
            {
                idx_of(composition, vowels[2]).unwrap_or(0)
            } else {
                idx_of(composition, vowels[1]).unwrap_or(0)
            },
        );
    }

    None
}

fn has_valid_tone(composition: &[&Transformation], tone: Tone) -> bool {
    if matches!(tone, Tone::None | Tone::Acute | Tone::Dot) {
        return true;
    }

    let (_, _, lc) = extract_cvc_trans(composition);
    if lc.is_empty() {
        return true;
    }

    let last_consonants =
        flatten(&lc, OutputOptions::RAW | OutputOptions::LOWER_CASE);
    for s in ["c", "k", "p", "t", "ch"] {
        if s == last_consonants {
            return false;
        }
    }

    true
}

fn get_last_tone_transformation<'a>(
    composition: &'a [&'a Transformation],
) -> Option<&'a Transformation> {
    composition.iter().rev().copied().find(|t| {
        t.rule.effect_type == EffectType::ToneTransformation
            && t.target.is_some()
    })
}

fn is_free(
    composition: &[&Transformation],
    trans: usize,
    effect_type: EffectType,
) -> bool {
    for t in composition {
        if let Some(target) = t.target
            && target == trans
            && t.rule.effect_type == effect_type
        {
            return false;
        }
    }
    true
}

fn extract_atomic_trans<'a, 'b>(
    composition: &'b [&'a Transformation],
    last_is_vowel: bool,
) -> (&'b [&'a Transformation], &'b [&'a Transformation]) {
    let mut idx = composition.len();

    while idx > 0 {
        let tmp = composition[idx - 1];
        if tmp.target.is_none() && (last_is_vowel != is_vowel(tmp.rule.result))
        {
            break;
        }
        idx -= 1;
    }

    (&composition[..idx], &composition[idx..])
}

fn extract_cvc_appending_trans<'a, 'b>(
    composition: &'b [&'a Transformation],
) -> (
    &'b [&'a Transformation],
    &'b [&'a Transformation],
    &'b [&'a Transformation],
) {
    let (head, last_consonant) = extract_atomic_trans(composition, false);
    let (first_head, mut vowel) = extract_atomic_trans(head, true);
    let mut first_consonant = first_head;

    let mut last_consonant = last_consonant;

    if !last_consonant.is_empty()
        && vowel.is_empty()
        && first_consonant.is_empty()
    {
        first_consonant = last_consonant;
        vowel = &[];
        last_consonant = &[];
    }

    // gi/qu consonant qualification.
    if (first_consonant.len() == 1
        && vowel.len() > 1
        && (last_consonant.is_empty() || vowel[1].rule.result != 'e')
        && vowel[0].rule.result == 'i'
        && first_consonant[0].rule.result == 'g')
        || (first_consonant.len() == 1
            && !vowel.is_empty()
            && first_consonant[0].rule.result == 'q'
            && vowel[0].rule.result == 'u')
    {
        first_consonant = &composition[..first_consonant.len() + 1];
        vowel = &vowel[1..];
    }

    (first_consonant, vowel, last_consonant)
}

fn extract_cvc_trans<'a>(
    composition: &[&'a Transformation],
) -> (Vec<&'a Transformation>, Vec<&'a Transformation>, Vec<&'a Transformation>)
{
    let mut appending_list: Vec<&Transformation> = Vec::new();
    for trans in composition {
        if trans.target.is_none() {
            appending_list.push(*trans);
        }
    }

    let (fc_app, vo_app, lc_app) = extract_cvc_appending_trans(&appending_list);

    let mut fc = fc_app.to_vec();
    let mut vo = vo_app.to_vec();
    let mut lc = lc_app.to_vec();

    // Re-attach effects by scanning composition once
    for trans in composition {
        if let Some(target_idx) = trans.target {
            let target_trans = composition[target_idx];
            if fc_app.iter().any(|&t| std::ptr::eq(t, target_trans)) {
                fc.push(trans);
            } else if vo_app.iter().any(|&t| std::ptr::eq(t, target_trans)) {
                vo.push(trans);
            } else if lc_app.iter().any(|&t| std::ptr::eq(t, target_trans)) {
                lc.push(trans);
            }
        }
    }

    (fc, vo, lc)
}

/// Extracts the last word along with its punctuation marks from the composition.
pub(crate) fn extract_last_word_with_punctuation_marks_refs<'a>(
    composition: &[&'a Transformation],
    _effect_keys: &[char],
) -> (Vec<&'a Transformation>, Vec<&'a Transformation>) {
    for i in (0..composition.len()).rev() {
        let Some(c) = first_canvas_char_in_suffix_refs(
            composition,
            i,
            OutputOptions::RAW,
        ) else {
            continue;
        };
        if is_space(c) {
            if i == composition.len() - 1 {
                return (composition.to_vec(), Vec::new());
            }
            return (
                composition[..i + 1].to_vec(),
                composition[i + 1..].to_vec(),
            );
        }
    }

    (Vec::new(), composition.to_vec())
}

/// Extracts the last word from the composition.
pub(crate) fn extract_last_word<'a>(
    composition: &[&'a Transformation],
    effect_keys: Option<&[char]>,
) -> (Vec<&'a Transformation>, Vec<&'a Transformation>) {
    for i in (0..composition.len()).rev() {
        let Some(c) = first_canvas_char_in_suffix_refs(
            composition,
            i,
            OutputOptions::NONE
                | OutputOptions::LOWER_CASE
                | OutputOptions::TONE_LESS
                | OutputOptions::MARK_LESS,
        ) else {
            continue;
        };
        if !is_alpha(c) && !in_key_list(effect_keys, c) {
            if i == composition.len() - 1 {
                return (composition.to_vec(), Vec::new());
            }
            let prev = composition[..i + 1].to_vec();
            let last = composition[i + 1..].to_vec();
            return (prev, last);
        }
    }

    (Vec::new(), composition.to_vec())
}

fn first_canvas_char_in_suffix_refs(
    composition: &[&Transformation],
    start: usize,
    options: OutputOptions,
) -> Option<char> {
    let mut first: Option<(usize, &Transformation)> = None;
    for (idx, trans) in composition[start..].iter().enumerate() {
        let abs_idx = start + idx;
        if options.contains(OutputOptions::RAW) {
            if trans.rule.key != '\0' {
                first = Some((abs_idx, *trans));
                break;
            }
            continue;
        }
        if trans.rule.effect_type == EffectType::Appending
            && trans.rule.key != '\0'
        {
            first = Some((abs_idx, *trans));
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

#[inline]
fn upper(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_uppercase()
    } else {
        c.to_uppercase().next().unwrap_or(c)
    }
}

fn find_mark_target(
    composition: &[&Transformation],
    rules: &[Rule],
) -> (Option<usize>, Option<Rule>) {
    let s = flatten(composition, OutputOptions::NONE);

    for trans in composition.iter().rev() {
        for rule in rules {
            if rule.effect_type != EffectType::MarkTransformation {
                continue;
            }
            if trans.rule.result == rule.effect_on && rule.effect > 0 {
                let Some(trans_idx) = idx_of(composition, trans) else {
                    continue;
                };
                let target = find_root_target(composition, trans_idx);

                let virtual_trans = Transformation {
                    rule: rule.clone(),
                    target: Some(target),
                    is_upper_case: false,
                };
                let mut tmp = composition.to_vec();
                tmp.push(&virtual_trans);
                if s == flatten(&tmp, OutputOptions::NONE) {
                    continue;
                }

                // Validate syllable if we apply this mark.
                let mut tmp2 = composition.to_vec();
                let virtual_trans2 = Transformation {
                    rule: rule.clone(),
                    target: Some(target),
                    is_upper_case: false,
                };
                tmp2.push(&virtual_trans2);
                if is_valid(&tmp2, false) {
                    return (Some(target), Some(rule.clone()));
                }
            }
        }
    }

    (None, None)
}

/// Finds the target for a given transformation rule within the current composition.
pub(crate) fn find_target(
    composition: &[&Transformation],
    applicable_rules: &[Rule],
    flags: u32,
) -> (Option<usize>, Option<Rule>) {
    let s = flatten(composition, OutputOptions::NONE);

    // find tone target
    for applicable_rule in applicable_rules {
        if applicable_rule.effect_type != EffectType::ToneTransformation {
            continue;
        }

        let mut target: Option<usize> = None;
        if (flags & EFREE_TONE_MARKING) != 0 {
            let tone = match applicable_rule.effect {
                1 => Tone::Grave,
                2 => Tone::Acute,
                3 => Tone::Hook,
                4 => Tone::Tilde,
                5 => Tone::Dot,
                _ => Tone::None,
            };
            if has_valid_tone(composition, tone) {
                target = find_tone_target(
                    composition,
                    (flags & ESTD_TONE_STYLE) != 0,
                );
            }
        } else if let Some(last_appending) =
            find_last_appending_trans(composition)
            && is_vowel(last_appending.rule.effect_on)
        {
            target = composition
                .iter()
                .position(|t| std::ptr::eq(*t, last_appending));
        }

        let virtual_trans = Transformation {
            rule: applicable_rule.clone(),
            target,
            is_upper_case: false,
        };
        let mut tmp = composition.to_vec();
        tmp.push(&virtual_trans);
        if s == flatten(&tmp, OutputOptions::NONE) {
            continue;
        }

        if applicable_rule.effect == Tone::None as u8
            && target.is_some()
            && is_free(
                composition,
                target.unwrap(),
                EffectType::ToneTransformation,
            )
            && add_tone_to_char(composition[target.unwrap()].rule.result, 0)
                == composition[target.unwrap()].rule.result
        {
            target = None;
        }

        return (target, Some(applicable_rule.clone()));
    }

    find_mark_target(composition, applicable_rules)
}

fn generate_undo_transformations(
    composition: &[&Transformation],
    rules: &[Rule],
    flags: u32,
) -> Vec<Transformation> {
    let mut transformations: Vec<Transformation> = Vec::new();
    let s = flatten(
        composition,
        OutputOptions::NONE
            | OutputOptions::TONE_LESS
            | OutputOptions::LOWER_CASE,
    );

    for rule in rules {
        if rule.effect_type == EffectType::ToneTransformation {
            let mut target: Option<usize> = None;
            if (flags & EFREE_TONE_MARKING) != 0 {
                let tone = match rule.effect {
                    1 => Tone::Grave,
                    2 => Tone::Acute,
                    3 => Tone::Hook,
                    4 => Tone::Tilde,
                    5 => Tone::Dot,
                    _ => Tone::None,
                };
                if has_valid_tone(composition, tone) {
                    target = find_tone_target(
                        composition,
                        (flags & ESTD_TONE_STYLE) != 0,
                    );
                }
            } else if let Some(last_appending) =
                find_last_appending_trans(composition)
                && is_vowel(last_appending.rule.effect_on)
            {
                target = composition
                    .iter()
                    .position(|t| std::ptr::eq(*t, last_appending));
            }

            let Some(target) = target else { continue };

            transformations.push(Transformation {
                target: Some(target),
                is_upper_case: false,
                rule: Rule {
                    effect_type: EffectType::ToneTransformation,
                    effect: 0,
                    key: '\0',
                    effect_on: '\0',
                    result: '\0',
                    appended_rules: Box::default(),
                },
            });
        } else if rule.effect_type == EffectType::MarkTransformation {
            for trans in composition.iter().rev() {
                if trans.rule.result == rule.effect_on {
                    let trans_idx = composition
                        .iter()
                        .position(|t| std::ptr::eq(*t, *trans))
                        .unwrap_or(0);
                    let target = find_root_target(composition, trans_idx);

                    let undo = Transformation {
                        target: Some(target),
                        is_upper_case: false,
                        rule: Rule {
                            key: '\0',
                            effect_type: EffectType::MarkTransformation,
                            effect: 0,
                            effect_on: '\0',
                            result: '\0',
                            appended_rules: Box::default(),
                        },
                    };

                    let mut tmp = composition.to_vec();
                    tmp.push(&undo);
                    if s == flatten(
                        &tmp,
                        OutputOptions::NONE
                            | OutputOptions::TONE_LESS
                            | OutputOptions::LOWER_CASE,
                    ) {
                        continue;
                    }

                    transformations.push(undo);
                }
            }
        }
    }

    transformations
}

fn contains_uho(s: &str) -> bool {
    s.contains("ưo") || s.contains("ươ")
}

/// Generates a list of transformations to apply based on the current composition and rules.
pub(crate) fn generate_transformations(
    composition: &[&Transformation],
    applicable_rules: &[Rule],
    flags: u32,
    lower_key: char,
    is_upper_case: bool,
) -> Vec<Transformation> {
    let mut transformations: Vec<Transformation> = Vec::new();

    // Double typing an effect key undoes it and its effects, e.g. w + w -> w (Telex 2)
    if let Some(last) = composition.last() {
        let rule = &last.rule;
        if rule.effect_type == EffectType::Appending
            && rule.key == lower_key
            && rule.key != rule.result
        {
            transformations.push(Transformation {
                rule: Rule {
                    effect_type: EffectType::MarkTransformation,
                    effect: Mark::Raw as u8,
                    key: '\0',
                    effect_on: '\0',
                    result: '\0',
                    appended_rules: Box::default(),
                },
                target: Some(composition.len() - 1),
                is_upper_case: false,
            });
            return transformations;
        }
    }

    if let (Some(target), Some(applicable_rule)) =
        find_target(composition, applicable_rules, flags)
    {
        transformations.push(Transformation {
            rule: applicable_rule.clone(),
            target: Some(target),
            is_upper_case,
        });

        if applicable_rule.effect_type != EffectType::MarkTransformation {
            return transformations;
        }

        let mut new_comp = composition.to_vec();
        new_comp.push(&transformations[0]);

        if is_valid(&new_comp, true) {
            return transformations;
        }

        // uow shortcut: create a virtual Mark.HORN rule that targets 'u' or 'o'.
        if let (Some(target2), Some(mut virtual_rule)) =
            find_target(&new_comp, applicable_rules, flags)
        {
            virtual_rule.key = '\0';
            transformations.push(Transformation {
                rule: virtual_rule,
                target: Some(target2),
                is_upper_case: false,
            });
            return transformations;
        }
    } else {
        // Implement ươ/ưo(i/c/ng) + o -> uô
        let flat = flatten(
            composition,
            OutputOptions::NONE
                | OutputOptions::TONE_LESS
                | OutputOptions::LOWER_CASE,
        );
        if contains_uho(&flat) {
            let rightmost = get_right_most_vowels(composition);
            let vowels = filter_appending_composition(&rightmost);
            if !vowels.is_empty() {
                let trans = Transformation {
                    target: composition
                        .iter()
                        .position(|t| std::ptr::eq(*t, vowels[0])),
                    is_upper_case: false,
                    rule: Rule {
                        effect_type: EffectType::MarkTransformation,
                        key: '\0',
                        effect: 0,
                        effect_on: '\0',
                        result: '\0',
                        appended_rules: Box::default(),
                    },
                };

                let mut tmp = composition.to_vec();
                tmp.push(&trans);

                if let (Some(target), Some(applicable_rule)) =
                    find_target(&tmp, applicable_rules, flags)
                    && composition
                        .iter()
                        .position(|t| std::ptr::eq(*t, vowels[0]))
                        != Some(target)
                {
                    transformations.push(trans);
                    transformations.push(Transformation {
                        rule: applicable_rule,
                        target: Some(target),
                        is_upper_case,
                    });
                    return transformations;
                }
            }
        }

        let undo =
            generate_undo_transformations(composition, applicable_rules, flags);
        if !undo.is_empty() {
            transformations.extend(undo);
            transformations.push(new_appending_trans(lower_key, is_upper_case));
        }
    }

    transformations
}

/// Generates fallback transformations when no specific rules match.
pub(crate) fn generate_fallback_transformations(
    applicable_rules: &[Rule],
    lower_key: char,
    is_upper_case: bool,
) -> Vec<Transformation> {
    let mut transformations: Vec<Transformation> = Vec::new();

    let trans =
        generate_appending_trans(applicable_rules, lower_key, is_upper_case);
    transformations.push(trans);

    let appended = transformations[0].rule.appended_rules.clone();
    for mut appended_rule in appended {
        let _is_upper_case = is_upper_case || is_upper(appended_rule.effect_on);
        appended_rule.key = '\0'; // virtual key
        appended_rule.effect_on = lower(appended_rule.effect_on);
        appended_rule.result = appended_rule.effect_on;
        transformations.push(Transformation {
            rule: appended_rule,
            target: None,
            is_upper_case: _is_upper_case,
        });
    }

    transformations
}

/// "Breaks" a composition by converting all non-virtual transformations into simple appending ones.
pub(crate) fn break_composition_slice(
    composition: &[Transformation],
) -> Vec<Transformation> {
    let mut result: Vec<Transformation> = Vec::with_capacity(composition.len());
    for trans in composition {
        if trans.rule.key == '\0' {
            continue;
        }
        result.push(new_appending_trans(trans.rule.key, trans.is_upper_case));
    }
    result
}

/// Updates the tone target in the composition based on the current syllable structure and tone style.
pub(crate) fn refresh_last_tone_target(
    composition: &mut [Transformation],
    std_style: bool,
) -> Vec<Transformation> {
    // Compute tone retargeting using immutable borrows first,
    // then mutate `composition` after those borrows are dropped.
    let (new_tone_target, last_tone_idx) = {
        let refs: Vec<&Transformation> = composition.iter().collect();
        let rightmost_vowels = get_right_most_vowels(&refs);
        if rightmost_vowels.is_empty()
            || get_last_tone_transformation(&refs).is_none()
        {
            return Vec::new();
        }

        let new_tone_target = find_tone_target(&refs, std_style);

        let mut last_tone_idx: Option<usize> = None;
        for (i, t) in composition.iter().enumerate().rev() {
            if t.rule.effect_type == EffectType::ToneTransformation
                && t.target.is_some()
            {
                last_tone_idx = Some(i);
                break;
            }
        }

        (new_tone_target, last_tone_idx)
    };

    let Some(idx) = last_tone_idx else { return Vec::new() };

    let last_target = composition[idx].target;
    if last_target == new_tone_target {
        return Vec::new();
    }

    composition[idx].target = new_tone_target;

    let mut transformations: Vec<Transformation> = Vec::new();

    if let Some(t) = last_target {
        transformations.push(Transformation {
            target: Some(t),
            is_upper_case: false,
            rule: Rule {
                key: '\0',
                effect_type: EffectType::ToneTransformation,
                effect: Tone::None as u8,
                effect_on: '\0',
                result: '\0',
                appended_rules: Box::default(),
            },
        });
    }

    if let Some(new_tone_target) = new_tone_target {
        let mut override_rule = composition[idx].rule.clone();
        override_rule.key = '\0';
        transformations.push(Transformation {
            target: Some(new_tone_target),
            is_upper_case: false,
            rule: override_rule,
        });
    }

    transformations
}
