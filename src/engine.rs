use crate::config::Config;
use crate::input_method::{InputMethod, Rule};
use crate::mode::{Mode, OutputOptions};

const MAX_ACTIVE_TRANS: usize = 32;

/// Represents a single keypress or a transformation derived from it.
#[derive(Clone, Debug)]
pub struct Transformation {
    pub rule: Rule,
    pub target: Option<usize>,
    pub is_upper_case: bool,
}

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

/// The main entry point for the Vietnamese Input Method Engine.
pub struct Engine {
    committed_text: String,
    /// Stack-allocated buffer for the current syllable to avoid heap allocations.
    active_buffer: [Option<Transformation>; MAX_ACTIVE_TRANS],
    active_len: usize,

    input_method: InputMethod,
    all_rules: Box<[Rule]>,
    ascii_rule_indices: [(u16, u16); 128],
    non_ascii_rule_indices: Box<[(char, (u16, u16))]>,
    ascii_effect_keys: [bool; 128],
    non_ascii_effect_keys: Vec<char>,
    config: Config,
}

impl Engine {
    pub fn new(input_method: InputMethod) -> Self {
        Self::with_config(input_method, Config::default())
    }

    pub fn with_config(input_method: InputMethod, config: Config) -> Self {
        let mut rules_by_key: std::collections::BTreeMap<char, Vec<Rule>> =
            std::collections::BTreeMap::new();
        for rule in &input_method.rules {
            let key = lower(rule.key);
            rules_by_key.entry(key).or_default().push(rule.clone());
        }

        let total_rules: usize = rules_by_key.values().map(|v| v.len()).sum();
        let mut all_rules_vec = Vec::with_capacity(total_rules);
        let mut ascii_rule_indices = [(0u16, 0u16); 128];
        let mut non_ascii_indices_vec = Vec::new();

        for (key, rules) in rules_by_key {
            let start = all_rules_vec.len() as u16;
            all_rules_vec.extend(rules);
            let end = all_rules_vec.len() as u16;
            if key.is_ascii() {
                ascii_rule_indices[key as usize] = (start, end);
            } else {
                non_ascii_indices_vec.push((key, (start, end)));
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
            committed_text: String::new(),
            active_buffer: std::array::from_fn(|_| None),
            active_len: 0,
            input_method,
            all_rules: all_rules_vec.into_boxed_slice(),
            ascii_rule_indices,
            non_ascii_rule_indices: non_ascii_indices_vec.into_boxed_slice(),
            ascii_effect_keys,
            non_ascii_effect_keys,
            config,
        }
    }

    /// Internal helper to get active composition as a slice of references.
    fn active_composition(&self) -> Vec<&Transformation> {
        self.active_buffer[..self.active_len]
            .iter()
            .map(|opt| opt.as_ref().unwrap())
            .collect()
    }

    /// Internal helper to get active composition as a Vec for mutation.
    fn active_composition_owned(&self) -> Vec<Transformation> {
        self.active_buffer[..self.active_len]
            .iter()
            .map(|opt| opt.as_ref().unwrap().clone())
            .collect()
    }

    fn set_active_composition(&mut self, comp: Vec<Transformation>) {
        self.active_len = comp.len().min(MAX_ACTIVE_TRANS);
        for (i, t) in comp.into_iter().enumerate().take(MAX_ACTIVE_TRANS) {
            self.active_buffer[i] = Some(t);
        }
    }

    pub fn config(&self) -> Config {
        self.config
    }
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }
    pub fn input_method(&self) -> InputMethod {
        self.input_method.clone()
    }

    fn get_applicable_rules(&self, key: char) -> &[Rule] {
        let key = lower(key);
        if key.is_ascii() {
            let (start, end) = self.ascii_rule_indices[key as usize];
            &self.all_rules[start as usize..end as usize]
        } else {
            self.non_ascii_rule_indices
                .binary_search_by_key(&key, |(k, _)| *k)
                .map(|idx| {
                    let (start, end) = self.non_ascii_rule_indices[idx].1;
                    &self.all_rules[start as usize..end as usize]
                })
                .unwrap_or(&[])
        }
    }

    fn can_process_key_raw(&self, lower_key: char) -> bool {
        if crate::utils::is_alpha(lower_key)
            || (lower_key.is_ascii()
                && self.ascii_effect_keys[lower_key as usize])
            || self.non_ascii_effect_keys.binary_search(&lower_key).is_ok()
        {
            return true;
        }
        if crate::utils::is_word_break_symbol(lower_key) {
            return false;
        }
        crate::utils::is_vietnamese_rune(lower_key)
    }

    fn generate_transformations(
        &self,
        composition: &mut Vec<Transformation>,
        key: char,
        is_upper_case: bool,
    ) {
        let lower_key = lower(key);
        let refs: Vec<&Transformation> = composition.iter().collect();
        let mut transformations = crate::bamboo_util::generate_transformations(
            &refs,
            self.get_applicable_rules(lower_key),
            self.config.to_flags(),
            lower_key,
            is_upper_case,
        );

        if transformations.is_empty() {
            transformations =
                crate::bamboo_util::generate_fallback_transformations(
                    self.get_applicable_rules(lower_key),
                    lower_key,
                    is_upper_case,
                );
            let mut new_comp = composition.clone();
            new_comp.extend(transformations.clone());
            let new_refs: Vec<&Transformation> = new_comp.iter().collect();

            if !self.input_method.super_keys.is_empty() {
                let current_str = crate::flattener::flatten(
                    &new_refs,
                    OutputOptions::TONE_LESS | OutputOptions::LOWER_CASE,
                );
                if uoh_tail_match(&current_str) {
                    let (target, rule) = crate::bamboo_util::find_target(
                        &new_refs,
                        self.get_applicable_rules(
                            self.input_method.super_keys[0],
                        ),
                        self.config.to_flags(),
                    );
                    if let (Some(target), Some(mut rule)) = (target, rule) {
                        rule.key = '\0';
                        transformations.push(Transformation {
                            rule,
                            target: Some(target),
                            is_upper_case: false,
                        });
                    }
                }
            }
        }
        composition.extend(transformations);
        if self.config.to_flags() & crate::bamboo_util::EFREE_TONE_MARKING != 0
            && self.is_valid_internal(composition, false)
        {
            let extra = crate::bamboo_util::refresh_last_tone_target(
                composition,
                self.config.to_flags() & crate::bamboo_util::ESTD_TONE_STYLE
                    != 0,
            );
            composition.extend(extra);
        }
    }

    fn new_composition(
        &self,
        mut composition: Vec<Transformation>,
        key: char,
        is_upper_case: bool,
    ) -> Vec<Transformation> {
        let (prev_refs, _) = crate::bamboo_util::extract_last_syllable(
            &composition,
            Some(&self.input_method.keys),
        );
        let syllable_abs_start = prev_refs.len();
        let mut syllable = composition.split_off(syllable_abs_start);
        let mut previous = composition;

        let offset = syllable_abs_start;
        if offset != 0 {
            for t in &mut syllable {
                if let Some(target) = t.target {
                    t.target = Some(target.saturating_sub(offset));
                }
            }
        }
        self.generate_transformations(&mut syllable, key, is_upper_case);
        if offset != 0 {
            for t in &mut syllable {
                if let Some(target) = t.target {
                    t.target = Some(target + offset);
                }
            }
        }
        previous.extend(syllable);
        previous
    }

    pub fn process(&mut self, s: &str, mode: Mode) -> String {
        self.process_str(s, mode).output()
    }
    pub fn process_str(&mut self, s: &str, mode: Mode) -> &Self {
        for key in s.chars() {
            self.process_key(key, mode);
        }
        self
    }

    pub fn process_key(&mut self, key: char, mode: Mode) {
        let lower_key = lower(key);
        let is_upper_case = is_upper(key);

        if mode == Mode::English || !self.can_process_key_raw(lower_key) {
            if crate::utils::is_word_break_symbol(lower_key) {
                self.commit();
            }
            let trans = crate::bamboo_util::new_appending_trans(
                lower_key,
                is_upper_case,
            );
            self.push_active(trans);
            if crate::utils::is_word_break_symbol(lower_key) {
                self.commit();
            }
            return;
        }

        let current = self.active_composition_owned();
        let next = self.new_composition(current, lower_key, is_upper_case);
        self.set_active_composition(next);
    }

    fn push_active(&mut self, trans: Transformation) {
        if self.active_len < MAX_ACTIVE_TRANS {
            self.active_buffer[self.active_len] = Some(trans);
            self.active_len += 1;
        }
    }

    pub fn commit(&mut self) {
        if self.active_len == 0 {
            return;
        }
        let word = self.output();
        self.committed_text.push_str(&word);
        self.active_len = 0;
    }

    pub fn output(&self) -> String {
        let comp = self.active_composition_owned();
        crate::flattener::flatten_slice(&comp, OutputOptions::NONE)
    }

    pub fn get_processed_str(&self, options: OutputOptions) -> String {
        let active_comp = self.active_composition_owned();
        if options.contains(OutputOptions::FULL_TEXT) {
            let mut result = self.committed_text.clone();
            result.push_str(&crate::flattener::flatten_slice(
                &active_comp,
                options,
            ));
            return result;
        }
        if options.contains(OutputOptions::PUNCTUATION_MODE) {
            let refs = self.active_composition();
            let (_, tail) = crate::bamboo_util::extract_last_word_with_punctuation_marks_refs(&refs, &self.input_method.keys);
            return crate::flattener::flatten(&tail, OutputOptions::NONE);
        }
        crate::flattener::flatten_slice(&active_comp, options)
    }

    pub fn is_valid(&self, input_is_full_complete: bool) -> bool {
        let comp = self.active_composition_owned();
        self.is_valid_internal(&comp, input_is_full_complete)
    }

    fn is_valid_internal(
        &self,
        composition: &[Transformation],
        input_is_full_complete: bool,
    ) -> bool {
        let refs: Vec<&Transformation> = composition.iter().collect();
        crate::bamboo_util::is_valid(&refs, input_is_full_complete)
    }

    pub fn restore_last_word(&mut self, to_vietnamese: bool) {
        let comp = self.active_composition_owned();
        let refs: Vec<&Transformation> = comp.iter().collect();
        let (prev_refs, _) = crate::bamboo_util::extract_last_word(
            &refs,
            Some(&self.input_method.keys),
        );
        let prev_len = prev_refs.len();

        let mut active = comp;
        let last = active.split_off(prev_len);
        let mut previous = active;

        if last.is_empty() {
            self.set_active_composition(previous);
            return;
        }
        if !to_vietnamese {
            previous.extend(crate::bamboo_util::break_composition_slice(&last));
            self.set_active_composition(previous);
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
        self.set_active_composition(previous);
    }

    pub fn remove_last_char(&mut self, refresh_last_tone_target: bool) {
        let comp = self.active_composition_owned();
        let last_appending_idx =
            crate::bamboo_util::find_last_appending_trans_idx(&comp);
        let Some(last_idx) = last_appending_idx else {
            return;
        };

        let last_appending_key = comp[last_idx].rule.key;
        if !self.can_process_key_raw(last_appending_key) {
            let mut next = comp;
            next.pop();
            self.set_active_composition(next);
            return;
        }

        let refs: Vec<&Transformation> = comp.iter().collect();
        let (previous_slice, _) = crate::bamboo_util::extract_last_word(
            &refs,
            Some(&self.input_method.keys),
        );
        let prev_len = previous_slice.len();

        let mut previous = comp;
        let last_comb = previous.split_off(prev_len);

        let mut new_comb: Vec<Transformation> = Vec::new();
        for (i, t) in last_comb.into_iter().enumerate() {
            let actual_idx = prev_len + i;
            if actual_idx == last_idx {
                continue;
            }
            if let Some(target) = t.target
                && target == last_idx
            {
                continue;
            }
            new_comb.push(t);
        }

        if refresh_last_tone_target {
            let extra = crate::bamboo_util::refresh_last_tone_target(
                &mut new_comb,
                self.config.to_flags() & crate::bamboo_util::ESTD_TONE_STYLE
                    != 0,
            );
            new_comb.extend(extra);
        }

        previous.extend(new_comb);
        self.set_active_composition(previous);
    }

    pub fn reset(&mut self) {
        self.committed_text.clear();
        self.active_len = 0;
    }
}
