use bitflags::bitflags;
use std::array;

use crate::rules_parser::{InputMethod, Rule};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Mode: u32 {
        const VIETNAMESE = 1 << 0;
        const ENGLISH = 1 << 1;
        const TONE_LESS = 1 << 2;
        const MARK_LESS = 1 << 3;
        const LOWER_CASE = 1 << 4;
        const FULL_TEXT = 1 << 5;
        const PUNCTUATION_MODE = 1 << 6;
        const IN_REVERSE_ORDER = 1 << 7;
    }
}

pub const VIETNAMESE_MODE: Mode = Mode::VIETNAMESE;
pub const ENGLISH_MODE: Mode = Mode::ENGLISH;
pub const TONE_LESS: Mode = Mode::TONE_LESS;
pub const MARK_LESS: Mode = Mode::MARK_LESS;
pub const LOWER_CASE: Mode = Mode::LOWER_CASE;
pub const FULL_TEXT: Mode = Mode::FULL_TEXT;
pub const PUNCTUATION_MODE: Mode = Mode::PUNCTUATION_MODE;
pub const IN_REVERSE_ORDER: Mode = Mode::IN_REVERSE_ORDER;

pub const EFREE_TONE_MARKING: u32 = 1 << 0;
pub const ESTD_TONE_STYLE: u32 = 1 << 1;
pub const EAUTO_CORRECT_ENABLED: u32 = 1 << 2;
pub const ESTD_FLAGS: u32 =
    EFREE_TONE_MARKING | ESTD_TONE_STYLE | EAUTO_CORRECT_ENABLED;

pub struct Transformation {
    pub rule: Rule,
    /// Index of an earlier transformation in the same composition.
    pub target: Option<usize>,
    pub is_upper_case: bool,
}

pub trait Engine {
    fn set_flag(&mut self, flag: u32);
    fn get_input_method(&self) -> InputMethod;
    fn process_key(&mut self, key: char, mode: Mode);
    fn process_str(&mut self, s: &str, mode: Mode);
    fn get_processed_str(&self, mode: Mode) -> String;
    fn is_valid(&self, input_is_full_complete: bool) -> bool;
    fn can_process_key(&self, key: char) -> bool;
    fn remove_last_char(&mut self, refresh_last_tone_target: bool);
    fn restore_last_word(&mut self, to_vietnamese: bool);
    fn reset(&mut self);
}

#[inline]
fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

#[inline]
fn is_upper(c: char) -> bool {
    lower(c) != c
}

fn uoh_tail_match(s: &str) -> bool {
    for pat in ["uơ", "ưo"] {
        if let Some(idx) = s.find(pat) {
            let after = &s[idx + pat.len()..];
            if after.chars().next().is_some_and(|c| c.is_alphabetic()) {
                return true;
            }
        }
    }
    false
}

pub struct BambooEngine {
    composition: Vec<Transformation>,
    input_method: InputMethod,
    ascii_rules_by_key: [Vec<Rule>; 128],
    non_ascii_rules_by_key: Vec<(char, Vec<Rule>)>,
    ascii_effect_keys: [bool; 128],
    non_ascii_effect_keys: Vec<char>,
    flags: u32,
}

pub fn new_engine(input_method: InputMethod, flags: u32) -> BambooEngine {
    BambooEngine::new(input_method, flags)
}

impl BambooEngine {
    pub fn new(input_method: InputMethod, flags: u32) -> Self {
        let mut ascii_rules_by_key: [Vec<Rule>; 128] =
            array::from_fn(|_| Vec::new());
        let mut non_ascii_rules_by_key: Vec<(char, Vec<Rule>)> = Vec::new();
        let mut non_ascii_key_to_idx: std::collections::BTreeMap<char, usize> =
            std::collections::BTreeMap::new();
        for rule in &input_method.rules {
            let key = lower(rule.key);
            if key.is_ascii() {
                ascii_rules_by_key[key as usize].push(rule.clone());
            } else {
                let idx =
                    *non_ascii_key_to_idx.entry(key).or_insert_with(|| {
                        let idx = non_ascii_rules_by_key.len();
                        non_ascii_rules_by_key.push((key, Vec::new()));
                        idx
                    });
                non_ascii_rules_by_key[idx].1.push(rule.clone());
            }
        }

        let mut ascii_effect_keys = [false; 128];
        let mut non_ascii_effect_keys: Vec<char> = Vec::new();
        for key in &input_method.keys {
            if key.is_ascii() {
                ascii_effect_keys[*key as usize] = true;
            } else {
                non_ascii_effect_keys.push(*key);
            }
        }
        non_ascii_effect_keys.sort_unstable();
        non_ascii_effect_keys.dedup();

        Self {
            composition: Vec::new(),
            input_method,
            ascii_rules_by_key,
            non_ascii_rules_by_key,
            ascii_effect_keys,
            non_ascii_effect_keys,
            flags,
        }
    }

    pub fn get_input_method(&self) -> InputMethod {
        self.input_method.clone()
    }

    pub fn set_flag(&mut self, flags: u32) {
        self.flags = flags;
    }

    pub fn get_flag(&self) -> u32 {
        self.flags
    }

    fn get_applicable_rules(&self, key: char) -> &[Rule] {
        let key = lower(key);
        if key.is_ascii() {
            self.ascii_rules_by_key[key as usize].as_slice()
        } else {
            self.non_ascii_rules_by_key
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, rules)| rules.as_slice())
                .unwrap_or(&[])
        }
    }

    fn can_process_key_raw(&self, lower_key: char) -> bool {
        if crate::utils::is_alpha(lower_key)
            || (lower_key.is_ascii()
                && self.ascii_effect_keys[lower_key as usize])
            || self.non_ascii_effect_keys.contains(&lower_key)
        {
            return true;
        }
        if crate::utils::is_word_break_symbol(lower_key) {
            return false;
        }
        crate::utils::is_vietnamese_rune(lower_key)
    }

    fn find_target_by_key(
        &self,
        composition: &[&Transformation],
        key: char,
    ) -> (Option<usize>, Option<Rule>) {
        crate::bamboo_util::find_target(
            composition,
            self.get_applicable_rules(key),
            self.flags,
        )
    }

    fn apply_uow_shortcut(
        &self,
        syllable: &[Transformation],
    ) -> Option<Transformation> {
        let s =
            crate::flattener::flatten_slice(syllable, TONE_LESS | LOWER_CASE);
        if !self.input_method.super_keys.is_empty() && uoh_tail_match(&s) {
            let refs: Vec<&Transformation> = syllable.iter().collect();
            let (target, missing_rule) =
                self.find_target_by_key(&refs, self.input_method.super_keys[0]);
            if let (Some(target), Some(mut missing_rule)) =
                (target, missing_rule)
            {
                missing_rule.key = '\0';
                return Some(Transformation {
                    rule: missing_rule,
                    target: Some(target),
                    is_upper_case: false,
                });
            }
        }
        None
    }

    fn refresh_last_tone_target(
        &self,
        syllable: &mut [Transformation],
    ) -> Vec<Transformation> {
        let refs: Vec<&Transformation> = syllable.iter().collect();
        if (self.flags & EFREE_TONE_MARKING) != 0
            && crate::bamboo_util::is_valid(&refs, false)
        {
            return crate::bamboo_util::refresh_last_tone_target(
                syllable,
                (self.flags & ESTD_TONE_STYLE) != 0,
            );
        }
        Vec::new()
    }

    fn generate_transformations(
        &self,
        syllable: &mut Vec<Transformation>,
        lower_key: char,
        is_upper_case: bool,
    ) {
        let refs: Vec<&Transformation> = syllable.iter().collect();
        let applicable = self.get_applicable_rules(lower_key);

        let mut trans = crate::bamboo_util::generate_transformations(
            &refs,
            applicable,
            self.flags,
            lower_key,
            is_upper_case,
        );

        if trans.is_empty() {
            trans = crate::bamboo_util::generate_fallback_transformations(
                applicable,
                lower_key,
                is_upper_case,
            );
            syllable.extend(trans);

            if let Some(virtual_trans) = self.apply_uow_shortcut(syllable) {
                syllable.push(virtual_trans);
            }
        } else {
            syllable.extend(trans);
        }

        let extra = self.refresh_last_tone_target(syllable);
        syllable.extend(extra);
    }

    fn new_composition(
        &self,
        mut composition: Vec<Transformation>,
        key: char,
        is_upper_case: bool,
    ) -> Vec<Transformation> {
        // We generate transformations on the last syllable, but targets must remain
        // consistent within the *last word* (so that later rendering of the last word
        // applies effects to the right character).
        let (previous_slice, last_refs) =
            crate::bamboo_util::extract_last_word(&composition, None);
        let word_start = previous_slice.len();

        // Recompute syllable start within the last word using validity checks.
        let mut anchor = 0usize;
        for i in 0..last_refs.len() {
            if !crate::bamboo_util::is_valid(&last_refs[anchor..=i], false) {
                anchor = i;
            }
        }
        let syllable_abs_start = word_start + anchor;
        let syllable_word_offset = anchor;

        let mut syllable = composition.split_off(syllable_abs_start);
        let mut previous = composition;

        // Rebase existing targets in the syllable to be syllable-local (0-based).
        if syllable_word_offset != 0 {
            for t in &mut syllable {
                if let Some(target) = t.target {
                    t.target =
                        Some(target.saturating_sub(syllable_word_offset));
                }
            }
        }

        self.generate_transformations(&mut syllable, key, is_upper_case);

        // Convert targets back to be word-local (relative to the last word start).
        if syllable_word_offset != 0 {
            for t in &mut syllable {
                if let Some(target) = t.target {
                    t.target = Some(target + syllable_word_offset);
                }
            }
        }

        previous.extend(syllable);
        previous
    }
}

impl Engine for BambooEngine {
    fn set_flag(&mut self, flag: u32) {
        self.flags = flag;
    }

    fn get_input_method(&self) -> InputMethod {
        self.input_method.clone()
    }

    fn process_str(&mut self, s: &str, mode: Mode) {
        for key in s.chars() {
            self.process_key(key, mode);
        }
    }

    fn process_key(&mut self, key: char, mode: Mode) {
        let lower_key = lower(key);
        let is_upper_case = is_upper(key);

        if mode.contains(ENGLISH_MODE) || !self.can_process_key_raw(lower_key) {
            let trans = crate::bamboo_util::new_appending_trans(
                lower_key,
                is_upper_case,
            );
            if mode.contains(IN_REVERSE_ORDER) {
                self.composition.insert(0, trans);
            } else {
                self.composition.push(trans);
            }
            return;
        }

        let current = std::mem::take(&mut self.composition);
        self.composition =
            self.new_composition(current, lower_key, is_upper_case);
    }

    fn get_processed_str(&self, mode: Mode) -> String {
        if mode.contains(FULL_TEXT) {
            return crate::flattener::flatten_slice(&self.composition, mode);
        }

        if mode.contains(PUNCTUATION_MODE) {
            let (_, tail) =
                crate::bamboo_util::extract_last_word_with_punctuation_marks(
                    &self.composition,
                    &self.input_method.keys,
                );
            return crate::flattener::flatten(&tail, VIETNAMESE_MODE);
        }

        let (_, tail) = crate::bamboo_util::extract_last_word(
            &self.composition,
            Some(&self.input_method.keys),
        );
        crate::flattener::flatten(&tail, mode)
    }

    fn is_valid(&self, input_is_full_complete: bool) -> bool {
        let (_, last) = crate::bamboo_util::extract_last_word(
            &self.composition,
            Some(&self.input_method.keys),
        );
        crate::bamboo_util::is_valid(&last, input_is_full_complete)
    }

    fn can_process_key(&self, key: char) -> bool {
        self.can_process_key_raw(lower(key))
    }

    fn remove_last_char(&mut self, refresh_last_tone_target: bool) {
        // Find last appending
        let mut last_appending_idx: Option<usize> = None;
        for (idx, t) in self.composition.iter().enumerate().rev() {
            if t.rule.effect_type == crate::rules_parser::EffectType::Appending
            {
                last_appending_idx = Some(idx);
                break;
            }
        }

        let Some(last_idx) = last_appending_idx else { return };
        if !self.can_process_key_raw(self.composition[last_idx].rule.key) {
            self.composition.pop();
            return;
        }

        let (previous_slice, _last_refs) =
            crate::bamboo_util::extract_last_word(
                &self.composition,
                Some(&self.input_method.keys),
            );
        let prev_len = previous_slice.len();

        let mut last = self.composition.split_off(prev_len);
        let mut previous = std::mem::take(&mut self.composition);

        let mut new_comb: Vec<Transformation> = Vec::new();
        for (idx, t) in last.drain(..).enumerate() {
            if idx == (last_idx - prev_len) {
                continue;
            }
            if let Some(target) = t.target
                && target == (last_idx - prev_len)
            {
                continue;
            }
            new_comb.push(t);
        }

        if refresh_last_tone_target {
            let extra = self.refresh_last_tone_target(&mut new_comb);
            new_comb.extend(extra);
        }

        previous.extend(new_comb);
        self.composition = previous;
    }

    fn restore_last_word(&mut self, to_vietnamese: bool) {
        let (previous_slice, _last_refs) =
            crate::bamboo_util::extract_last_word(
                &self.composition,
                Some(&self.input_method.keys),
            );
        let prev_len = previous_slice.len();

        let last = self.composition.split_off(prev_len);
        let mut previous = std::mem::take(&mut self.composition);

        if last.is_empty() {
            self.composition = previous;
            return;
        }

        if !to_vietnamese {
            previous.extend(crate::bamboo_util::break_composition_slice(&last));
            self.composition = previous;
            return;
        }

        let mut new_comp: Vec<Transformation> = Vec::new();
        for t in last {
            if t.rule.key == '\0' {
                continue;
            }
            new_comp =
                self.new_composition(new_comp, t.rule.key, t.is_upper_case);
        }

        previous.extend(new_comp);
        self.composition = previous;
    }

    fn reset(&mut self) {
        self.composition.clear();
    }
}
