//! The core engine that processes keypresses and maintains the IME state.

use crate::config::Config;
use crate::input_method::{InputMethod, Rule};
use crate::mode::{Mode, OutputOptions};

const MAX_ACTIVE_TRANS: usize = 32;

/// Represents a single keypress or a transformation derived from it (e.g., adding a mark or tone).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Transformation {
    /// The rule that was applied.
    pub rule: Rule,
    /// The index of the transformation in the composition that this transformation targets (if any).
    /// For example, a tone mark transformation targets an earlier vowel.
    pub target: Option<usize>,
    /// Whether the resulting character should be uppercase.
    pub is_upper_case: bool,
}

#[inline]
fn lower(c: char) -> char {
    if c.is_ascii() { c.to_ascii_lowercase() } else { c.to_lowercase().next().unwrap_or(c) }
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

/// The main stateful processor of the Vietnamese Input Method Engine.
///
/// It maintains an internal buffer of transformations and produces the correctly marked Vietnamese text.
pub struct Engine {
    committed_text: String,
    /// Stack-allocated buffer for the active composition to avoid heap allocations.
    active_buffer: [Transformation; MAX_ACTIVE_TRANS],
    active_len: usize,

    input_method: InputMethod,
    all_rules: Box<[Rule]>,
    ascii_rule_indices: [(u16, u16); 128],
    non_ascii_rule_indices: Box<[(char, (u16, u16))]>,
    ascii_effect_keys: [bool; 128],
    non_ascii_effect_keys: Vec<char>,
    config: Config,

    // Scratch buffers to avoid per-keystroke allocations.
    work_comp: Vec<Transformation>,
    scratch_comp: Vec<Transformation>,

    prev_preedit: String,
    delta_buf: String,
}

impl Engine {
    /// Creates a new engine with the specified input method and default configuration.
    pub fn new(input_method: InputMethod) -> Self {
        Self::with_config(input_method, Config::default())
    }

    /// Creates a new engine with a specific input method and configuration.
    pub fn with_config(input_method: InputMethod, config: Config) -> Self {
        let mut rules_by_key: std::collections::BTreeMap<char, Vec<Rule>> =
            std::collections::BTreeMap::new();
        for rule in &input_method.rules {
            let key = lower(rule.key);
            rules_by_key.entry(key).or_default().push(*rule);
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
            active_buffer: [Transformation::default(); MAX_ACTIVE_TRANS],
            active_len: 0,
            input_method,
            all_rules: all_rules_vec.into_boxed_slice(),
            ascii_rule_indices,
            non_ascii_rule_indices: non_ascii_indices_vec.into_boxed_slice(),
            ascii_effect_keys,
            non_ascii_effect_keys,
            config,

            work_comp: Vec::with_capacity(MAX_ACTIVE_TRANS),
            scratch_comp: Vec::with_capacity(MAX_ACTIVE_TRANS),

            prev_preedit: String::with_capacity(64),
            delta_buf: String::with_capacity(64),
        }
    }

    #[inline]
    fn active_slice(&self) -> &[Transformation] {
        &self.active_buffer[..self.active_len]
    }

    fn take_active_into(&mut self, out: &mut Vec<Transformation>) {
        out.clear();
        out.extend_from_slice(self.active_slice());
        self.active_len = 0;
    }

    fn set_active_from_vec(&mut self, src: &mut Vec<Transformation>) {
        self.active_len = src.len().min(MAX_ACTIVE_TRANS);
        self.active_buffer[..self.active_len].copy_from_slice(&src[..self.active_len]);
        src.clear();
    }

    /// Returns the current configuration of the engine.
    pub fn config(&self) -> Config {
        self.config
    }

    /// Updates the engine configuration.
    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    /// Returns a copy of the current input method.
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
            || (lower_key.is_ascii() && self.ascii_effect_keys[lower_key as usize])
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
        let mut transformations = crate::bamboo_util::generate_transformations(
            composition,
            self.get_applicable_rules(lower_key),
            self.config.to_flags(),
            lower_key,
            is_upper_case,
        );

        if transformations.is_empty() {
            transformations = crate::bamboo_util::generate_fallback_transformations(
                self.get_applicable_rules(lower_key),
                lower_key,
                is_upper_case,
            );
            let mut new_comp = composition.clone();
            new_comp.extend(transformations.clone());

            if !self.input_method.super_keys.is_empty() {
                let current_str = crate::flattener::flatten_slice(
                    &new_comp,
                    OutputOptions::TONE_LESS | OutputOptions::LOWER_CASE,
                );
                if uoh_tail_match(&current_str) {
                    let (target, rule) = crate::bamboo_util::find_target(
                        &new_comp,
                        self.get_applicable_rules(self.input_method.super_keys[0]),
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
        composition.extend(transformations.iter().cloned());
        if self.config.to_flags() & crate::bamboo_util::EFREE_TONE_MARKING != 0
            && self.is_valid_internal(composition, false)
        {
            let extra = crate::bamboo_util::refresh_last_tone_target(
                composition,
                self.config.to_flags() & crate::bamboo_util::ESTD_TONE_STYLE != 0,
            );
            composition.extend(extra);
        }
    }

    fn last_syllable_start(composition: &[Transformation]) -> usize {
        let mut idx = composition.len();
        let mut last_is_vowel = false;
        let mut found_vowel = false;

        while idx > 0 {
            let tmp = &composition[idx - 1];
            if tmp.target.is_none() {
                let is_v = crate::utils::is_vowel(tmp.rule.result);
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

        idx
    }

    fn new_composition_in_place(
        &self,
        composition: &mut Vec<Transformation>,
        scratch: &mut Vec<Transformation>,
        key: char,
        is_upper_case: bool,
    ) {
        let syllable_abs_start = Self::last_syllable_start(composition);

        scratch.clear();
        scratch.extend(composition.drain(syllable_abs_start..));

        let offset = syllable_abs_start;
        if offset != 0 {
            for t in scratch.iter_mut() {
                if let Some(target) = t.target {
                    t.target = Some(target.saturating_sub(offset));
                }
            }
        }

        self.generate_transformations(scratch, key, is_upper_case);

        if offset != 0 {
            for t in scratch.iter_mut() {
                if let Some(target) = t.target {
                    t.target = Some(target + offset);
                }
            }
        }

        composition.append(scratch);
    }

    /// Processes a string of characters and returns the current active word.
    pub fn process(&mut self, s: &str, mode: Mode) -> String {
        self.process_str(s, mode).output()
    }

    /// Processes a string of characters and returns a reference to the engine.
    pub fn process_str(&mut self, s: &str, mode: Mode) -> &Self {
        for key in s.chars() {
            self.process_key(key, mode);
        }
        self
    }

    fn lcp_chars_and_bytes(a: &str, b: &str) -> (usize, usize) {
        let mut lcp_chars = 0usize;
        let mut lcp_bytes = 0usize;
        for (ac, bc) in a.chars().zip(b.chars()) {
            if ac == bc {
                lcp_chars += 1;
                lcp_bytes += ac.len_utf8();
            } else {
                break;
            }
        }
        (lcp_chars, lcp_bytes)
    }

    /// Processes a single key and returns the "delta" change required for a text editor.
    ///
    /// This is useful for IMEs to update the preedit text efficiently without rewriting the entire word.
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. `backspaces_chars`: Number of characters to delete from the end of the previous preedit.
    /// 2. `backspaces_bytes`: Number of UTF-8 bytes to delete.
    /// 3. `inserted`: The new string to append after deletion.
    pub fn process_key_delta(&mut self, key: char, mode: Mode) -> (usize, usize, &str) {
        self.process_key(key, mode);

        let active_len = self.active_len;
        let active = &self.active_buffer[..active_len];
        crate::flattener::flatten_slice_into(active, OutputOptions::NONE, &mut self.delta_buf);

        let (lcp_chars, lcp_bytes) = Self::lcp_chars_and_bytes(&self.prev_preedit, &self.delta_buf);

        let prev_chars = self.prev_preedit.chars().count();

        let prev_bytes = self.prev_preedit.len();

        let backspaces_chars = prev_chars.saturating_sub(lcp_chars);
        let backspaces_bytes = prev_bytes.saturating_sub(lcp_bytes);

        std::mem::swap(&mut self.prev_preedit, &mut self.delta_buf);
        let inserted = &self.prev_preedit[lcp_bytes..];
        (backspaces_chars, backspaces_bytes, inserted)
    }

    /// Similar to [`Self::process_key_delta`], but writes the inserted string into a provided buffer.
    ///
    /// # Returns
    /// The number of backspaces (characters) to perform.
    pub fn process_key_delta_into(
        &mut self,
        key: char,
        mode: Mode,
        inserted: &mut String,
    ) -> usize {
        let (backspaces_chars, _backspaces_bytes, ins) = self.process_key_delta(key, mode);
        inserted.clear();
        inserted.push_str(ins);
        backspaces_chars
    }

    /// Processes a single character.
    ///
    /// The `mode` determines whether to apply Vietnamese transformation rules.
    pub fn process_key(&mut self, key: char, mode: Mode) {
        let lower_key = lower(key);
        let is_upper_case = is_upper(key);

        if mode == Mode::English || !self.can_process_key_raw(lower_key) {
            if crate::utils::is_word_break_symbol(lower_key) {
                self.commit();
            }
            let trans = crate::bamboo_util::new_appending_trans(lower_key, is_upper_case);
            self.push_active(trans);
            if crate::utils::is_word_break_symbol(lower_key) {
                self.commit();
            }
            return;
        }

        let mut work = std::mem::take(&mut self.work_comp);
        let mut scratch = std::mem::take(&mut self.scratch_comp);

        self.take_active_into(&mut work);
        self.new_composition_in_place(&mut work, &mut scratch, lower_key, is_upper_case);
        self.set_active_from_vec(&mut work);

        self.work_comp = work;
        self.scratch_comp = scratch;
    }

    fn push_active(&mut self, trans: Transformation) {
        if self.active_len < MAX_ACTIVE_TRANS {
            self.active_buffer[self.active_len] = trans;
            self.active_len += 1;
        }
    }

    /// Clears the active syllable buffer and appends it to the committed text.
    pub fn commit(&mut self) {
        if self.active_len == 0 {
            return;
        }
        let word = self.output();
        self.committed_text.push_str(&word);
        self.active_len = 0;
    }

    /// Returns the currently active syllable as a string.
    pub fn output(&self) -> String {
        crate::flattener::flatten_slice(self.active_slice(), OutputOptions::NONE)
    }

    /// Returns the processed string according to the specified options.
    ///
    /// This can be used to get the full text (committed + active) or variations like toneless text.
    pub fn get_processed_str(&self, options: OutputOptions) -> String {
        let active = self.active_slice();
        if options.contains(OutputOptions::FULL_TEXT) {
            let mut result = self.committed_text.clone();
            result.push_str(&crate::flattener::flatten_slice(active, options));
            return result;
        }
        if options.contains(OutputOptions::PUNCTUATION_MODE) {
            if active.is_empty() {
                return String::new();
            }
            let (_, tail) = crate::bamboo_util::extract_last_word_with_punctuation_marks(
                active,
                &self.input_method.keys,
            );
            return crate::flattener::flatten_slice(tail, OutputOptions::NONE);
        }
        crate::flattener::flatten_slice(active, options)
    }

    /// Checks if the current composition forms a valid Vietnamese syllable.
    pub fn is_valid(&self, input_is_full_complete: bool) -> bool {
        self.is_valid_internal(self.active_slice(), input_is_full_complete)
    }

    fn is_valid_internal(
        &self,
        composition: &[Transformation],
        input_is_full_complete: bool,
    ) -> bool {
        crate::bamboo_util::is_valid(composition, input_is_full_complete)
    }

    /// Restores the last word in the composition to its un-transformed state.
    ///
    /// If `to_vietnamese` is true, it attempts to re-apply Vietnamese transformations.
    pub fn restore_last_word(&mut self, to_vietnamese: bool) {
        let mut work = std::mem::take(&mut self.work_comp);
        let mut scratch = std::mem::take(&mut self.scratch_comp);

        self.take_active_into(&mut work);
        if work.is_empty() {
            self.set_active_from_vec(&mut work);
            self.work_comp = work;
            self.scratch_comp = scratch;
            return;
        }

        let (prev_slice, last) =
            crate::bamboo_util::extract_last_word(&work, Some(&self.input_method.keys));
        let mut previous = prev_slice.to_vec();

        if last.is_empty() {
            self.set_active_from_vec(&mut work);
            self.work_comp = work;
            self.scratch_comp = scratch;
            return;
        }
        if !to_vietnamese {
            previous.extend(crate::bamboo_util::break_composition_slice(last));
            self.set_active_from_vec(&mut previous);
            self.work_comp = work;
            self.scratch_comp = scratch;
            return;
        }

        let mut new_comp: Vec<Transformation> = Vec::new();
        for t in last {
            if t.rule.key == '\0' {
                continue;
            }
            self.new_composition_in_place(&mut new_comp, &mut scratch, t.rule.key, t.is_upper_case);
        }
        previous.extend(new_comp);

        self.set_active_from_vec(&mut previous);
        self.work_comp = work;
        self.scratch_comp = scratch;
    }

    /// Removes the last character from the active composition.
    pub fn remove_last_char(&mut self, refresh_last_tone_target: bool) {
        let mut work = std::mem::take(&mut self.work_comp);

        self.take_active_into(&mut work);
        let last_appending_idx = crate::bamboo_util::find_last_appending_trans_idx(&work);
        let Some(last_idx) = last_appending_idx else {
            self.set_active_from_vec(&mut work);
            self.work_comp = work;
            return;
        };

        let last_appending_key = work[last_idx].rule.key;
        if !self.can_process_key_raw(last_appending_key) {
            work.pop();
            self.set_active_from_vec(&mut work);
            self.work_comp = work;
            return;
        }

        if work.is_empty() {
            self.set_active_from_vec(&mut work);
            self.work_comp = work;
            return;
        }

        let (prev_slice, last_comb_slice) =
            crate::bamboo_util::extract_last_word(&work, Some(&self.input_method.keys));
        let mut previous = prev_slice.to_vec();
        let last_comb = last_comb_slice.to_vec();

        let mut new_comb: Vec<Transformation> = Vec::new();
        let prev_len = previous.len();
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
                self.config.to_flags() & crate::bamboo_util::ESTD_TONE_STYLE != 0,
            );
            new_comb.extend(extra);
        }

        previous.extend(new_comb);
        self.set_active_from_vec(&mut previous);
        self.work_comp = work;
    }

    /// Resets the engine state, clearing committed and active text.
    pub fn reset(&mut self) {
        self.committed_text.clear();
        self.active_len = 0;
        self.prev_preedit.clear();
        self.delta_buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_backspaces_and_inserted() {
        let telex = InputMethod::telex();
        let mut e = Engine::new(telex);

        let (bs1, _bb1, ins1) = e.process_key_delta('a', Mode::Vietnamese);
        assert_eq!(bs1, 0, "First 'a' should have 0 backspaces");
        assert_eq!(ins1, "a");

        let (bs2, _bb2, ins2) = e.process_key_delta('s', Mode::Vietnamese);
        assert_eq!(bs2, 1, "Adding 's' to 'a' should have 1 backspace for 'á'");
        assert_eq!(ins2, "á");

        let (bs3, _bb3, ins3) = e.process_key_delta(' ', Mode::Vietnamese);
        assert_eq!(bs3, 1, "Space should clear the preedit 'á'");
        assert_eq!(ins3, "");
    }
}
