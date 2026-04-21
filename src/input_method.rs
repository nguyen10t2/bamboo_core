use phf::{Map, phf_map};

use crate::input_method_def::InputMethodDef;
use crate::utils::{add_mark_to_toneless_char, add_tone_to_char, is_vowel};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    None = 0,
    Grave = 1,
    Acute = 2,
    Hook = 3,
    Tilde = 4,
    Dot = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    None = 0,
    Hat = 1,
    Breve = 2,
    Horn = 3,
    Dash = 4,
    /// Not used by the current DSL, kept for parity with other ports.
    Raw = 5,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectType {
    Appending = 0,
    MarkTransformation = 1,
    ToneTransformation = 2,
    Replacing = 3,
}

static TONES: Map<&'static str, Tone> = phf_map! {
    "XoaDauThanh" => Tone::None,
    "DauSac" => Tone::Acute,
    "DauHuyen" => Tone::Grave,
    "DauNga" => Tone::Tilde,
    "DauNang" => Tone::Dot,
    "DauHoi" => Tone::Hook,
};

#[derive(Clone, Debug)]
pub struct Rule {
    pub key: char,
    /// Effect value:
    /// - if `effect_type == ToneTransformation`: this is a `Tone` as `u8`
    /// - if `effect_type == MarkTransformation`: this is a `Mark` as `u8`
    pub effect: u8,
    pub effect_type: EffectType,
    pub effect_on: char,
    pub result: char,
    pub appended_rules: Vec<Rule>,
}

impl Rule {
    pub fn set_tone(&mut self, tone: Tone) {
        self.effect = tone as u8;
    }

    pub fn set_mark(&mut self, mark: Mark) {
        self.effect = mark as u8;
    }

    pub fn get_tone(&self) -> Tone {
        // Safety: effect is created by parser or engine.
        match self.effect {
            1 => Tone::Grave,
            2 => Tone::Acute,
            3 => Tone::Hook,
            4 => Tone::Tilde,
            5 => Tone::Dot,
            _ => Tone::None,
        }
    }

    pub fn get_mark(&self) -> Mark {
        match self.effect {
            1 => Mark::Hat,
            2 => Mark::Breve,
            3 => Mark::Horn,
            4 => Mark::Dash,
            5 => Mark::Raw,
            _ => Mark::None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct InputMethod {
    pub name: String,
    pub rules: Vec<Rule>,
    pub super_keys: Vec<char>,
    pub tone_keys: Vec<char>,
    pub appending_keys: Vec<char>,
    pub keys: Vec<char>,
}

impl InputMethod {
    pub fn telex() -> Self {
        parse_input_method("Telex")
    }

    pub fn vni() -> Self {
        parse_input_method("VNI")
    }

    pub fn viqr() -> Self {
        parse_input_method("VIQR")
    }

    pub fn microsoft_layout() -> Self {
        parse_input_method("Microsoft layout")
    }

    pub fn telex_2() -> Self {
        parse_input_method("Telex 2")
    }

    pub fn telex_vni() -> Self {
        parse_input_method("Telex + VNI")
    }

    pub fn telex_vni_viqr() -> Self {
        parse_input_method("Telex + VNI + VIQR")
    }

    pub fn vni_french_layout() -> Self {
        parse_input_method("VNI Bàn phím tiếng Pháp")
    }

    pub fn telex_w() -> Self {
        parse_input_method("Telex W")
    }
}

/// Parse a known input method by name from the built-in definitions.
pub(crate) fn parse_input_method(im_name: &str) -> InputMethod {
    let defs = crate::input_method_def::get_input_method_definitions();
    defs.get(im_name)
        .copied()
        .map(|def| parse_input_method_def(im_name, def))
        .unwrap_or_default()
}

pub(crate) fn parse_input_method_def(
    im_name: &str,
    im_def: &InputMethodDef,
) -> InputMethod {
    let mut im =
        InputMethod { name: im_name.to_string(), ..Default::default() };

    for (key_str, line) in im_def.entries() {
        let key = match key_str.chars().next() {
            Some(c) => c,
            None => continue,
        };

        im.rules.extend(parse_rules(key, line));

        if contains_uo_case_insensitive(line) {
            im.super_keys.push(key);
        }
        im.keys.push(key);
    }

    for rule in &im.rules {
        if rule.effect_type == EffectType::Appending {
            im.appending_keys.push(rule.key);
        }
        if rule.effect_type == EffectType::ToneTransformation {
            im.tone_keys.push(rule.key);
        }
    }

    im
}

#[inline]
fn contains_uo_case_insensitive(s: &str) -> bool {
    let mut prev_u = false;
    for c in s.chars() {
        let lc = c.to_ascii_lowercase();
        if prev_u && lc == 'o' {
            return true;
        }
        prev_u = lc == 'u';
    }
    false
}

pub(crate) fn parse_rules(key: char, line: &str) -> Vec<Rule> {
    if let Some(tone) = TONES.get(line).copied() {
        return vec![Rule {
            key,
            effect_type: EffectType::ToneTransformation,
            effect: tone as u8,
            effect_on: '\0',
            result: '\0',
            appended_rules: Vec::new(),
        }];
    }

    parse_toneless_rules(key, line)
}

pub(crate) fn parse_toneless_rules(key: char, line: &str) -> Vec<Rule> {
    let lower = line.to_lowercase();

    if let Some((effective_ons, results, rest)) = parse_dsl(&lower) {
        let mut rules = Vec::new();
        for (effective_on, result) in
            effective_ons.into_iter().zip(results.into_iter())
        {
            let Some(effect) = find_mark_from_char(result) else {
                continue;
            };
            rules.extend(parse_toneless_rule(
                key,
                effective_on,
                result,
                effect,
            ));
        }

        if let Some(rule) = get_appending_rule(key, rest) {
            rules.push(rule);
        }

        return rules;
    }

    if let Some(rule) = get_appending_rule(key, line) {
        return vec![rule];
    }

    Vec::new()
}

fn parse_toneless_rule(
    key: char,
    effective_on: char,
    result: char,
    effect: Mark,
) -> Vec<Rule> {
    let mut rules = Vec::new();

    for chr in get_mark_family(effective_on) {
        if chr == result {
            rules.push(Rule {
                key,
                effect_type: EffectType::MarkTransformation,
                effect: 0,
                effect_on: result,
                result: effective_on,
                appended_rules: Vec::new(),
            });
            continue;
        }

        if is_vowel(chr) {
            for tone in 0u8..=5 {
                rules.push(Rule {
                    key,
                    effect_type: EffectType::MarkTransformation,
                    effect_on: add_tone_to_char(chr, tone),
                    effect: effect as u8,
                    result: add_tone_to_char(result, tone),
                    appended_rules: Vec::new(),
                });
            }
        } else {
            rules.push(Rule {
                key,
                effect_type: EffectType::MarkTransformation,
                effect_on: chr,
                effect: effect as u8,
                result,
                appended_rules: Vec::new(),
            });
        }
    }

    rules
}

/// Parse: `([a-zA-Z]+)_(\p{L}+)([_\p{L}]*)`.
fn parse_dsl(s: &str) -> Option<(Vec<char>, Vec<char>, &str)> {
    let (left, right) = s.split_once('_')?;
    if left.is_empty() || !left.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }

    let mut results = Vec::new();
    let mut rest_start_byte = right.len();

    for (byte_idx, ch) in right.char_indices() {
        if ch.is_alphabetic() {
            results.push(ch);
            continue;
        }
        rest_start_byte = byte_idx;
        break;
    }

    if results.is_empty() {
        return None;
    }

    let rest = &right[rest_start_byte..];
    Some((left.chars().collect(), results, rest))
}

/// Parse: `(_?)_(\p{L}+)`.
fn get_appending_rule(key: char, value: &str) -> Option<Rule> {
    if !value.starts_with('_') {
        return None;
    }

    // "_x" or "__x" forms.
    let start = if value.starts_with("__") { 2 } else { 1 };
    let tail = value.get(start..)?;

    let mut letters = Vec::new();
    for ch in tail.chars() {
        if ch.is_alphabetic() {
            letters.push(ch);
        } else {
            break;
        }
    }

    let first = *letters.first()?;

    let mut rule = Rule {
        key,
        effect_type: EffectType::Appending,
        effect: 0,
        effect_on: first,
        result: first,
        appended_rules: Vec::new(),
    };

    for &ch in letters.iter().skip(1) {
        rule.appended_rules.push(Rule {
            key,
            effect_type: EffectType::Appending,
            effect: 0,
            effect_on: ch,
            result: ch,
            appended_rules: Vec::new(),
        });
    }

    Some(rule)
}

fn get_mark_family(c: char) -> Vec<char> {
    let base = add_tone_to_char(c, 0);
    let canonical = add_mark_to_toneless_char(base, 0);

    // Marks are 0..=4 in utils' internal mark table.
    let mut family: Vec<char> =
        (0u8..=4).map(|m| add_mark_to_toneless_char(canonical, m)).collect();

    family.sort_unstable();
    family.dedup();
    family
}

fn find_mark_from_char(c: char) -> Option<Mark> {
    let c = c.to_lowercase().next().unwrap_or(c);
    let toneless = add_tone_to_char(c, 0);
    let base = add_mark_to_toneless_char(toneless, 0);

    for m in 0u8..=4 {
        if add_mark_to_toneless_char(base, m) == toneless {
            return Some(match m {
                1 => Mark::Hat,
                2 => Mark::Breve,
                3 => Mark::Horn,
                4 => Mark::Dash,
                _ => Mark::None,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tone_rules() {
        let rules = parse_rules('z', "XoaDauThanh");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].effect_type, EffectType::ToneTransformation);
        assert_eq!(rules[0].effect, Tone::None as u8);

        let rules = parse_rules('x', "DauNga");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].effect_type, EffectType::ToneTransformation);
        assert_eq!(rules[0].get_tone(), Tone::Tilde);
    }

    #[test]
    fn parse_toneless_rules_cases() {
        let rules = parse_toneless_rules('d', "D_Đ");
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[0].effect, Mark::Dash as u8);
        assert_eq!(rules[0].effect_on, 'd');

        let rules = parse_toneless_rules('{', "_Ư");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].effect_type, EffectType::Appending);
        assert_eq!(rules[0].effect_on, 'Ư');

        let rules = parse_toneless_rules('w', "UOA_ƯƠĂ");
        assert_eq!(rules.len(), 33);
        assert_eq!(rules[0].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[0].get_mark(), Mark::Horn);
        assert_eq!(rules[0].effect_on, 'u');
        assert_eq!(rules[7].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[7].get_mark(), Mark::Horn);
        assert_eq!(rules[7].effect_on, 'o');
        assert_eq!(rules[20].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[20].get_mark(), Mark::Breve);
        assert_eq!(rules[20].effect_on, 'a');

        let rules = parse_toneless_rules('w', "UOA_ƯƠĂ__Ư");
        assert_eq!(rules.len(), 34);
        assert_eq!(rules[20].effect_type, EffectType::MarkTransformation);
        assert_eq!(rules[20].get_mark(), Mark::Breve);
        assert_eq!(rules[20].effect_on, 'a');
        assert_eq!(rules[33].effect_type, EffectType::Appending);
        assert_eq!(rules[33].effect_on, 'ư');
    }

    #[test]
    fn parse_append_rule() {
        let rules = parse_toneless_rules('[', "__ươ");
        assert_eq!(rules.len(), 1);
        let append_rules = &rules[0].appended_rules;
        assert_eq!(append_rules.len(), 1);
        assert_eq!(append_rules[0].effect_type, EffectType::Appending);
        assert_eq!(append_rules[0].effect_on, 'ơ');

        let rules = parse_toneless_rules('{', "__ƯƠ");
        assert_eq!(rules.len(), 1);
        let append_rules = &rules[0].appended_rules;
        assert_eq!(append_rules.len(), 1);
        assert_eq!(append_rules[0].effect_type, EffectType::Appending);
        assert_eq!(append_rules[0].effect_on, 'Ơ');
    }

    #[test]
    fn parse_input_method_super_key_detection() {
        let im = parse_input_method("Telex");
        assert!(im.super_keys.contains(&'w'));
    }

    #[test]
    fn parse_telex_o_hat_rule_exists() {
        // In Telex, typing 'o' after an existing 'o' should be able to mark it as 'ô'.
        let rules = parse_toneless_rules('o', "O_Ô");
        assert!(rules.iter().any(|r| {
            r.effect_type == EffectType::MarkTransformation
                && r.get_mark() == Mark::Hat
                && r.effect_on == 'o'
                && r.result == 'ô'
        }));
        assert!(!rules.iter().any(|r| r.effect_type == EffectType::Appending));
    }

    #[test]
    fn telex2_has_no_appending_rule_for_o() {
        let im = parse_input_method("Telex 2");
        let o_rules: Vec<_> =
            im.rules.iter().filter(|r| r.key == 'o').collect();
        assert!(!o_rules.is_empty());
        assert!(
            !o_rules.iter().any(|r| r.effect_type == EffectType::Appending)
        );
    }
}
