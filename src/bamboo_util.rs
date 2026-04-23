//! Internal utility functions for Vietnamese syllable analysis and transformation generation.

use crate::engine::{MAX_ACTIVE_TRANS, Transformation, TransformationStack};
use crate::flattener::flatten_slice;
use crate::input_method::{EffectType, Mark, Rule, Tone};
use crate::mode::OutputOptions;
use crate::spelling::{is_valid_cvc, is_valid_cvc_chars};
use crate::utils::{add_mark_to_char, add_tone_to_char, is_alpha, is_space, is_vowel};

/// Flag to enable free tone marking (allows placing tones at any time).
pub(crate) const EFREE_TONE_MARKING: u32 = 1 << 0;
/// Flag to use the standard Vietnamese tone style (e.g., placing tone on the second vowel in some cases).
pub(crate) const ESTD_TONE_STYLE: u32 = 1 << 1;

#[inline]
fn lower(c: char) -> char {
    if c.is_ascii() { c.to_ascii_lowercase() } else { c.to_lowercase().next().unwrap_or(c) }
}

#[inline]
fn is_upper(c: char) -> bool {
    if c.is_ascii() { c.is_ascii_uppercase() } else { lower(c) != c }
}

fn in_key_list(keys: Option<&[char]>, key: char) -> bool {
    keys.map(|ks| ks.contains(&key)).unwrap_or(false)
}

/// Finds the last transformation in the composition that resulted in an appended character.
pub(crate) fn find_last_appending_trans(composition: &[Transformation]) -> Option<Transformation> {
    composition.iter().rev().find(|trans| trans.rule.effect_type == EffectType::Appending).copied()
}

/// Creates a new transformation that simply appends a character.
pub(crate) fn new_appending_trans(key: char, is_upper_case: bool) -> Transformation {
    Transformation {
        is_upper_case,
        target: None,
        rule: Rule {
            key,
            effect_on: key,
            effect: 0,
            effect_type: EffectType::Appending,
            result: key,
            appended: ['\0'; 2],
            appended_len: 0,
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
            let mut rule = *rule;
            let mut _is_upper_case = is_upper_case || is_upper(rule.effect_on);
            rule.effect_on = lower(rule.effect_on);
            rule.result = rule.effect_on;
            _is_upper_case |= is_upper(rule.effect_on);
            return Transformation { is_upper_case: _is_upper_case, target: None, rule };
        }
    }

    new_appending_trans(lower_key, is_upper_case)
}

fn find_root_target(composition: &[Transformation], mut target: usize) -> usize {
    while let Some(t) = composition[target].target {
        target = t;
    }
    target
}

/// Checks if the current composition represents a valid Vietnamese syllable.
pub(crate) fn is_valid(composition: &[Transformation], input_is_full_complete: bool) -> bool {
    if composition.len() <= 1 {
        return true;
    }

    // last tone checking
    for trans in composition.iter().rev() {
        if trans.rule.effect_type == EffectType::ToneTransformation {
            let last_tone = trans.rule.get_tone();
            if !has_valid_tone(composition, last_tone) {
                return false;
            }
            break;
        }
    }

    // spell checking (fast path: no heap for engine's bounded composition)
    if composition.len() <= MAX_ACTIVE_TRANS {
        let mut app_abs = [0usize; MAX_ACTIVE_TRANS];
        let mut app_len = 0usize;

        for (abs_idx, t) in composition.iter().enumerate() {
            if t.target.is_none() {
                app_abs[app_len] = abs_idx;
                app_len += 1;
            }
        }

        let app_indices = &app_abs[..app_len];
        let (fc_idxs, vo_idxs, lc_idxs) = extract_cvc_appending_indices(composition, app_indices);

        let mut fc_chars = ['\0'; MAX_ACTIVE_TRANS];
        let mut vo_chars = ['\0'; MAX_ACTIVE_TRANS];
        let mut lc_chars = ['\0'; MAX_ACTIVE_TRANS];

        // Resolve all characters in a single pass O(N)
        let mut resolved = ['\0'; MAX_ACTIVE_TRANS];
        for &abs_idx in app_indices.iter() {
            resolved[abs_idx] = composition[abs_idx].rule.effect_on;
        }

        for t in composition {
            if let Some(target) = t.target
                && target < MAX_ACTIVE_TRANS
            {
                match t.rule.effect_type {
                    EffectType::MarkTransformation => {
                        if t.rule.effect == Mark::Raw as u8 {
                            resolved[target] = composition[target].rule.key;
                        } else {
                            resolved[target] = add_mark_to_char(resolved[target], t.rule.effect);
                        }
                    }
                    EffectType::ToneTransformation => {
                        resolved[target] = add_tone_to_char(resolved[target], t.rule.effect);
                    }
                    _ => {}
                }
            }
        }

        for (i, &abs) in fc_idxs.iter().enumerate() {
            fc_chars[i] = lower(add_tone_to_char(resolved[abs], 0));
        }
        for (i, &abs) in vo_idxs.iter().enumerate() {
            vo_chars[i] = lower(add_tone_to_char(resolved[abs], 0));
        }
        for (i, &abs) in lc_idxs.iter().enumerate() {
            lc_chars[i] = lower(add_tone_to_char(resolved[abs], 0));
        }

        return is_valid_cvc_chars(
            &fc_chars[..fc_idxs.len()],
            &vo_chars[..vo_idxs.len()],
            &lc_chars[..lc_idxs.len()],
            input_is_full_complete,
        );
    }

    // fallback for uncommon long compositions
    let cvc = extract_cvc_trans(composition);
    let flatten_mode = OutputOptions::NONE | OutputOptions::LOWER_CASE | OutputOptions::TONE_LESS;
    is_valid_cvc(
        &flatten_slice(cvc.fc_slice(), flatten_mode),
        &flatten_slice(cvc.vo_slice(), flatten_mode),
        &flatten_slice(cvc.lc_slice(), flatten_mode),
        input_is_full_complete,
    )
}

/// Represents the broken-down parts of a Vietnamese syllable (Consonant-Vowel-Consonant).
#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct Cvc {
    /// Transformations in the first consonant part.
    pub fc: [Transformation; 8],
    /// Number of transformations in `fc`.
    pub fc_len: u8,
    /// Transformations in the vowel part.
    pub vo: [Transformation; 8],
    /// Number of transformations in `vo`.
    pub vo_len: u8,
    /// Transformations in the last consonant part.
    pub lc: [Transformation; 8],
    /// Number of transformations in `lc`.
    pub lc_len: u8,
}

impl Cvc {
    /// Returns the transformations for the first consonant.
    pub fn fc_slice(&self) -> &[Transformation] {
        &self.fc[..self.fc_len as usize]
    }

    /// Returns the transformations for the vowel part.
    pub fn vo_slice(&self) -> &[Transformation] {
        &self.vo[..self.vo_len as usize]
    }

    /// Returns the transformations for the last consonant.
    pub fn lc_slice(&self) -> &[Transformation] {
        &self.lc[..self.lc_len as usize]
    }
}

fn find_tone_target(composition: &[Transformation], std_style: bool) -> Option<usize> {
    if composition.is_empty() {
        return None;
    }

    let cvc = extract_cvc_trans(composition);
    let vowels = cvc.vo_slice();
    let lc = cvc.lc_slice();

    let mut appending_vowels = [Transformation::default(); 8];
    let mut appending_vowels_len = 0usize;
    for t in vowels {
        if t.target.is_none() {
            appending_vowels[appending_vowels_len] = *t;
            appending_vowels_len += 1;
        }
    }
    let app_vowels = &appending_vowels[..appending_vowels_len];

    if appending_vowels_len == 1 {
        return composition.iter().position(|t| *t == app_vowels[0]);
    }

    if appending_vowels_len == 2 && std_style {
        let mut target: Option<usize> = None;
        for trans in vowels {
            if trans.rule.result == 'ơ' || trans.rule.result == 'ê' {
                target =
                    Some(trans.target.unwrap_or_else(|| {
                        composition.iter().position(|t| t == trans).unwrap_or(0)
                    }));
            }
        }
        if target.is_none() {
            target = Some(if !lc.is_empty() {
                composition.iter().position(|t| *t == app_vowels[1]).unwrap_or(0)
            } else {
                composition.iter().position(|t| *t == app_vowels[0]).unwrap_or(0)
            });
        }
        return target;
    }

    if appending_vowels_len == 2 {
        if !lc.is_empty() {
            return composition.iter().position(|t| *t == app_vowels[1]);
        }

        let s = flatten_slice(
            app_vowels,
            OutputOptions::RAW
                | OutputOptions::LOWER_CASE
                | OutputOptions::TONE_LESS
                | OutputOptions::MARK_LESS,
        );
        return Some(if matches!(s.as_str(), "oa" | "oe" | "uy" | "ue" | "uo") {
            composition.iter().position(|t| *t == app_vowels[1]).unwrap_or(0)
        } else {
            composition.iter().position(|t| *t == app_vowels[0]).unwrap_or(0)
        });
    }

    if appending_vowels_len == 3 {
        return Some(
            if flatten_slice(
                app_vowels,
                OutputOptions::RAW
                    | OutputOptions::LOWER_CASE
                    | OutputOptions::TONE_LESS
                    | OutputOptions::MARK_LESS,
            ) == "uye"
            {
                composition.iter().position(|t| *t == app_vowels[2]).unwrap_or(0)
            } else {
                composition.iter().position(|t| *t == app_vowels[1]).unwrap_or(0)
            },
        );
    }

    None
}

fn has_valid_tone(composition: &[Transformation], tone: Tone) -> bool {
    if matches!(tone, Tone::None | Tone::Acute | Tone::Dot) {
        return true;
    }
    if composition.is_empty() {
        return true;
    }

    let cvc = extract_cvc_trans(composition);
    if cvc.lc_len == 0 {
        return true;
    }

    let last_consonants =
        flatten_slice(cvc.lc_slice(), OutputOptions::RAW | OutputOptions::LOWER_CASE);
    !matches!(last_consonants.as_str(), "c" | "k" | "p" | "t" | "ch")
}

fn get_last_tone_transformation(composition: &[Transformation]) -> Option<Transformation> {
    composition
        .iter()
        .rev()
        .find(|t| t.rule.effect_type == EffectType::ToneTransformation && t.target.is_some())
        .copied()
}

fn is_free(composition: &[Transformation], trans_idx: usize, effect_type: EffectType) -> bool {
    for t in composition {
        if let Some(target) = t.target
            && target == trans_idx
            && t.rule.effect_type == effect_type
        {
            return false;
        }
    }
    true
}

fn extract_cvc_appending_indices<'a>(
    _composition: &[Transformation],
    app_indices: &'a [usize],
) -> (&'a [usize], &'a [usize], &'a [usize]) {
    let mut results = ['\0'; MAX_ACTIVE_TRANS];
    for (i, &idx) in app_indices.iter().enumerate() {
        results[i] = _composition[idx].rule.result;
    }

    let (head, lc) = {
        let mut idx = app_indices.len();
        while idx > 0 {
            if is_vowel(results[idx - 1]) {
                break;
            }
            idx -= 1;
        }
        (&app_indices[..idx], &app_indices[idx..])
    };

    let (fc, vo) = {
        let mut idx = head.len();
        while idx > 0 {
            if !is_vowel(results[idx - 1]) {
                break;
            }
            idx -= 1;
        }
        (&head[..idx], &head[idx..])
    };

    if fc.is_empty() && vo.is_empty() && !lc.is_empty() {
        return (lc, &[], &[]);
    }

    let mut fc_final = fc;
    let mut vo_final = vo;
    if (fc.len() == 1
        && vo.len() > 1
        && (lc.is_empty() || (fc.len() + 1 < results.len() && results[fc.len() + 1] != 'e'))
        && results[fc.len()] == 'i'
        && results[fc[0]] == 'g')
        || (fc.len() == 1 && !vo.is_empty() && results[fc[0]] == 'q' && results[vo[0]] == 'u')
    {
        fc_final = &app_indices[..fc.len() + 1];
        vo_final = &vo[1..];
    }

    (fc_final, vo_final, lc)
}

fn extract_cvc_trans(composition: &[Transformation]) -> Cvc {
    let mut app_indices = [0usize; MAX_ACTIVE_TRANS];
    let mut app_len = 0usize;
    for (i, t) in composition.iter().enumerate() {
        if t.target.is_none() && app_len < MAX_ACTIVE_TRANS {
            app_indices[app_len] = i;
            app_len += 1;
        }
    }

    let (fc_idxs, vo_idxs, lc_idxs) =
        extract_cvc_appending_indices(composition, &app_indices[..app_len]);

    let mut res = Cvc::default();

    for &i in fc_idxs {
        if (res.fc_len as usize) < res.fc.len() {
            res.fc[res.fc_len as usize] = composition[i];
            res.fc_len += 1;
        }
    }
    for &i in vo_idxs {
        if (res.vo_len as usize) < res.vo.len() {
            res.vo[res.vo_len as usize] = composition[i];
            res.vo_len += 1;
        }
    }
    for &i in lc_idxs {
        if (res.lc_len as usize) < res.lc.len() {
            res.lc[res.lc_len as usize] = composition[i];
            res.lc_len += 1;
        }
    }

    for trans in composition {
        if let Some(target_idx) = trans.target {
            if fc_idxs.contains(&target_idx) {
                if (res.fc_len as usize) < res.fc.len() {
                    res.fc[res.fc_len as usize] = *trans;
                    res.fc_len += 1;
                }
            } else if vo_idxs.contains(&target_idx) {
                if (res.vo_len as usize) < res.vo.len() {
                    res.vo[res.vo_len as usize] = *trans;
                    res.vo_len += 1;
                }
            } else if lc_idxs.contains(&target_idx) && (res.lc_len as usize) < res.lc.len() {
                res.lc[res.lc_len as usize] = *trans;
                res.lc_len += 1;
            }
        }
    }

    res
}

/// Extracts the last word along with its punctuation marks from the composition.
pub(crate) fn extract_last_word_with_punctuation_marks<'a>(
    composition: &'a [Transformation],
    _effect_keys: &[char],
) -> (&'a [Transformation], &'a [Transformation]) {
    for i in (0..composition.len()).rev() {
        let Some(c) =
            crate::flattener::first_canvas_char_in_suffix(composition, i, OutputOptions::RAW)
        else {
            continue;
        };
        if is_space(c) {
            if i == composition.len() - 1 {
                return (composition, &[]);
            }
            return (&composition[..i + 1], &composition[i + 1..]);
        }
    }

    (&[], composition)
}

/// Extracts the last word from the composition.
pub(crate) fn extract_last_word<'a>(
    composition: &'a [Transformation],
    effect_keys: Option<&[char]>,
) -> (&'a [Transformation], &'a [Transformation]) {
    for i in (0..composition.len()).rev() {
        let Some(c) = crate::flattener::first_canvas_char_in_suffix(
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
                return (composition, &[]);
            }
            return (&composition[..i + 1], &composition[i + 1..]);
        }
    }

    (&[], composition)
}

fn is_effective(composition: &[Transformation], target_idx: usize, new_rule: &Rule) -> bool {
    for t in composition {
        if t.target == Some(target_idx) && t.rule.effect_type == new_rule.effect_type {
            if t.rule.effect == new_rule.effect {
                return false;
            }
            return true;
        }
    }

    let appending_idx = find_root_target(composition, target_idx);
    let appending = &composition[appending_idx];

    let mut current_char = appending.rule.effect_on;
    for t in composition {
        if t.target == Some(appending_idx) {
            match t.rule.effect_type {
                EffectType::MarkTransformation => {
                    if t.rule.effect == Mark::Raw as u8 {
                        current_char = appending.rule.key;
                    } else {
                        current_char = add_mark_to_char(current_char, t.rule.effect);
                    }
                }
                EffectType::ToneTransformation => {
                    current_char = add_tone_to_char(current_char, t.rule.effect);
                }
                _ => {}
            }
        }
    }

    let mut next_char = current_char;
    match new_rule.effect_type {
        EffectType::MarkTransformation => {
            if new_rule.effect == Mark::Raw as u8 {
                next_char = appending.rule.key;
            } else {
                next_char = add_mark_to_char(current_char, new_rule.effect);
            }
        }
        EffectType::ToneTransformation => {
            next_char = add_tone_to_char(current_char, new_rule.effect);
        }
        _ => {}
    }

    next_char != current_char
}

fn find_mark_target(
    composition: &[Transformation],
    rules: &[Rule],
) -> (Option<usize>, Option<Rule>) {
    for (idx, trans) in composition.iter().enumerate().rev() {
        for rule in rules {
            if rule.effect_type != EffectType::MarkTransformation {
                continue;
            }
            if trans.rule.result == rule.effect_on && rule.effect > 0 {
                let target = find_root_target(composition, idx);

                if !is_effective(composition, target, rule) {
                    continue;
                }

                let mut tmp = [Transformation::default(); MAX_ACTIVE_TRANS];
                let mut tmp_len = composition.len();
                tmp[..tmp_len].copy_from_slice(composition);
                tmp[tmp_len] =
                    Transformation { rule: *rule, target: Some(target), is_upper_case: false };
                tmp_len += 1;

                if is_valid(&tmp[..tmp_len], false) {
                    return (Some(target), Some(*rule));
                }
            }
        }
    }

    (None, None)
}

/// Finds the target for a given transformation rule within the current composition.
pub(crate) fn find_target(
    composition: &[Transformation],
    applicable_rules: &[Rule],
    flags: u32,
) -> (Option<usize>, Option<Rule>) {
    for applicable_rule in applicable_rules {
        if applicable_rule.effect_type != EffectType::ToneTransformation {
            continue;
        }

        let mut target: Option<usize> = None;
        if (flags & EFREE_TONE_MARKING) != 0 {
            let tone = applicable_rule.get_tone();
            if has_valid_tone(composition, tone) {
                target = find_tone_target(composition, (flags & ESTD_TONE_STYLE) != 0);
            }
        } else if let Some(last_appending) = find_last_appending_trans(composition)
            && is_vowel(last_appending.rule.effect_on)
        {
            for (idx, t) in composition.iter().enumerate() {
                if *t == last_appending {
                    target = Some(idx);
                    break;
                }
            }
        }

        let Some(t_idx) = target else { continue };
        let effective = is_effective(composition, t_idx, applicable_rule);
        if !effective {
            continue;
        }

        if applicable_rule.effect == Tone::None as u8
            && is_free(composition, t_idx, EffectType::ToneTransformation)
            && add_tone_to_char(composition[t_idx].rule.result, 0) == composition[t_idx].rule.result
        {
            target = None;
        }

        if target.is_some() {
            return (target, Some(*applicable_rule));
        }
    }

    find_mark_target(composition, applicable_rules)
}

fn generate_undo_transformations(
    composition: &[Transformation],
    rules: &[Rule],
    flags: u32,
    out: &mut TransformationStack,
) {
    for rule in rules {
        if rule.effect_type == EffectType::ToneTransformation {
            let mut target: Option<usize> = None;
            if (flags & EFREE_TONE_MARKING) != 0 {
                let tone = rule.get_tone();
                if has_valid_tone(composition, tone) {
                    target = find_tone_target(composition, (flags & ESTD_TONE_STYLE) != 0);
                }
            } else if let Some(last_appending) = find_last_appending_trans(composition)
                && is_vowel(last_appending.rule.effect_on)
            {
                for (idx, t) in composition.iter().enumerate() {
                    if *t == last_appending {
                        target = Some(idx);
                        break;
                    }
                }
            }

            let Some(target) = target else { continue };
            let undo_rule = Rule {
                effect_type: EffectType::ToneTransformation,
                effect: 0,
                key: '\0',
                effect_on: '\0',
                result: '\0',
                appended: ['\0'; 2],
                appended_len: 0,
            };

            if is_effective(composition, target, &undo_rule) {
                out.push(Transformation {
                    target: Some(target),
                    is_upper_case: false,
                    rule: undo_rule,
                });
            }
        } else if rule.effect_type == EffectType::MarkTransformation {
            for (idx, trans) in composition.iter().enumerate().rev() {
                if trans.rule.result == rule.effect_on {
                    let target = find_root_target(composition, idx);

                    let undo_rule = Rule {
                        key: '\0',
                        effect_type: EffectType::MarkTransformation,
                        effect: 0,
                        effect_on: '\0',
                        result: '\0',
                        appended: ['\0'; 2],
                        appended_len: 0,
                    };

                    if is_effective(composition, target, &undo_rule) {
                        out.push(Transformation {
                            target: Some(target),
                            is_upper_case: false,
                            rule: undo_rule,
                        });
                    }
                }
            }
        }
    }
}

fn contains_uho(s: &str) -> bool {
    s.contains("ưo") || s.contains("ươ")
}

/// Generates a list of transformations to apply based on the current composition and rules.
pub(crate) fn generate_transformations(
    composition: &[Transformation],
    applicable_rules: &[Rule],
    flags: u32,
    lower_key: char,
    is_upper_case: bool,
    out: &mut TransformationStack,
) {
    if let Some(last) = composition.last() {
        let rule = &last.rule;
        if rule.effect_type == EffectType::Appending
            && rule.key == lower_key
            && rule.key != rule.result
        {
            out.push(Transformation {
                rule: Rule {
                    effect_type: EffectType::MarkTransformation,
                    effect: Mark::Raw as u8,
                    key: '\0',
                    effect_on: '\0',
                    result: '\0',
                    appended: ['\0'; 2],
                    appended_len: 0,
                },
                target: Some(composition.len() - 1),
                is_upper_case: false,
            });
        }
    }

    if let (Some(target), Some(applicable_rule)) = find_target(composition, applicable_rules, flags)
    {
        out.push(Transformation { rule: applicable_rule, target: Some(target), is_upper_case });

        if applicable_rule.effect_type != EffectType::MarkTransformation {
            return;
        }

        let mut new_comp = [Transformation::default(); MAX_ACTIVE_TRANS];
        let mut new_len = composition.len();
        new_comp[..new_len].copy_from_slice(composition);
        new_comp[new_len] = out.as_slice()[0];
        new_len += 1;

        if is_valid(&new_comp[..new_len], true) {
        } else if let (Some(target2), Some(mut virtual_rule)) =
            find_target(&new_comp[..new_len], applicable_rules, flags)
        {
            virtual_rule.key = '\0';
            out.push(Transformation {
                rule: virtual_rule,
                target: Some(target2),
                is_upper_case: false,
            });
        }
    } else {
        let flat = flatten_slice(
            composition,
            OutputOptions::NONE | OutputOptions::TONE_LESS | OutputOptions::LOWER_CASE,
        );
        if contains_uho(&flat) {
            let cvc = extract_cvc_trans(composition);
            let vowels = cvc.vo_slice();
            let mut app_vowels = [Transformation::default(); 8];
            let mut app_vowels_len = 0usize;
            for t in vowels {
                if t.target.is_none() {
                    app_vowels[app_vowels_len] = *t;
                    app_vowels_len += 1;
                }
            }

            if app_vowels_len > 0 {
                let target_idx = composition.iter().position(|t| *t == app_vowels[0]);
                let trans = Transformation {
                    target: target_idx,
                    is_upper_case: false,
                    rule: Rule {
                        effect_type: EffectType::MarkTransformation,
                        key: '\0',
                        effect: 0,
                        effect_on: '\0',
                        result: '\0',
                        appended: ['\0'; 2],
                        appended_len: 0,
                    },
                };

                let mut tmp = [Transformation::default(); MAX_ACTIVE_TRANS];
                let mut tmp_len = composition.len();
                tmp[..tmp_len].copy_from_slice(composition);
                tmp[tmp_len] = trans;
                tmp_len += 1;

                if let (Some(target), Some(applicable_rule)) =
                    find_target(&tmp[..tmp_len], applicable_rules, flags)
                    && target_idx != Some(target)
                {
                    out.push(trans);
                    out.push(Transformation {
                        rule: applicable_rule,
                        target: Some(target),
                        is_upper_case,
                    });
                    return;
                }
            }
        }

        generate_undo_transformations(composition, applicable_rules, flags, out);
        if !out.is_empty() {
            out.push(new_appending_trans(lower_key, is_upper_case));
        }
    }
}

/// Generates fallback transformations when no specific rules match.
pub(crate) fn generate_fallback_transformations(
    applicable_rules: &[Rule],
    lower_key: char,
    is_upper_case: bool,
    out: &mut TransformationStack,
) {
    let trans = generate_appending_trans(applicable_rules, lower_key, is_upper_case);
    out.push(trans);

    for i in 0..trans.rule.appended_len {
        let appended_char = trans.rule.appended[i as usize];
        let _is_upper_case = is_upper_case || is_upper(appended_char);
        let appended_rule = Rule {
            key: '\0',
            effect_type: EffectType::Appending,
            effect: 0,
            effect_on: lower(appended_char),
            result: lower(appended_char),
            appended: ['\0'; 2],
            appended_len: 0,
        };
        out.push(Transformation {
            rule: appended_rule,
            target: None,
            is_upper_case: _is_upper_case,
        });
    }
}

/// "Breaks" a composition by converting all non-virtual transformations into simple appending ones.
pub(crate) fn break_composition_slice(
    composition: &[Transformation],
) -> [Transformation; MAX_ACTIVE_TRANS] {
    let mut result = [Transformation::default(); MAX_ACTIVE_TRANS];
    let mut len = 0;
    for trans in composition {
        if trans.rule.key == '\0' {
            continue;
        }
        if len < MAX_ACTIVE_TRANS {
            result[len] = new_appending_trans(trans.rule.key, trans.is_upper_case);
            len += 1;
        }
    }
    result
}

/// Updates the tone target in the composition based on the current syllable structure and tone style.
pub(crate) fn refresh_last_tone_target_into(
    composition: &mut [Transformation],
    std_style: bool,
    out: &mut TransformationStack,
) {
    let (new_tone_target, last_tone_idx) = {
        let cvc = extract_cvc_trans(composition);
        if cvc.vo_len == 0 || get_last_tone_transformation(composition).is_none() {
            return;
        }

        let new_tone_target = find_tone_target(composition, std_style);

        let mut last_tone_idx: Option<usize> = None;
        for (i, t) in composition.iter().enumerate().rev() {
            if t.rule.effect_type == EffectType::ToneTransformation && t.target.is_some() {
                last_tone_idx = Some(i);
                break;
            }
        }

        (new_tone_target, last_tone_idx)
    };

    let Some(idx) = last_tone_idx else { return };

    let last_target = composition[idx].target;
    if last_target == new_tone_target {
        return;
    }

    composition[idx].target = new_tone_target;

    if let Some(t) = last_target {
        out.push(Transformation {
            target: Some(t),
            is_upper_case: false,
            rule: Rule {
                key: '\0',
                effect_type: EffectType::ToneTransformation,
                effect: Tone::None as u8,
                effect_on: '\0',
                result: '\0',
                appended: ['\0'; 2],
                appended_len: 0,
            },
        });
    }

    if let Some(new_tone_target) = new_tone_target {
        let mut override_rule = composition[idx].rule;
        override_rule.key = '\0';
        out.push(Transformation {
            target: Some(new_tone_target),
            is_upper_case: false,
            rule: override_rule,
        });
    }
}
