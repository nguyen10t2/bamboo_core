use bamboo_core::{EffectType, Mark, Tone, parse_input_method};

#[test]
fn parse_tone_rules() {
    let rules = bamboo_core::parse_rules('z', "XoaDauThanh");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].effect_type, EffectType::ToneTransformation);
    assert_eq!(rules[0].effect, Tone::None as u8);

    let rules = bamboo_core::parse_rules('x', "DauNga");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].effect_type, EffectType::ToneTransformation);
    assert_eq!(rules[0].get_tone(), Tone::Tilde);
}

#[test]
fn parse_toneless_rules_cases() {
    let rules = bamboo_core::parse_toneless_rules('d', "D_Đ");
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].effect_type, EffectType::MarkTransformation);
    assert_eq!(rules[0].effect, Mark::Dash as u8);
    assert_eq!(rules[0].effect_on, 'd');

    let rules = bamboo_core::parse_toneless_rules('{', "_Ư");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].effect_type, EffectType::Appending);
    assert_eq!(rules[0].effect_on, 'Ư');

    let rules = bamboo_core::parse_toneless_rules('w', "UOA_ƯƠĂ");
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

    let rules = bamboo_core::parse_toneless_rules('w', "UOA_ƯƠĂ__Ư");
    assert_eq!(rules.len(), 34);
    assert_eq!(rules[20].effect_type, EffectType::MarkTransformation);
    assert_eq!(rules[20].get_mark(), Mark::Breve);
    assert_eq!(rules[20].effect_on, 'a');
    assert_eq!(rules[33].effect_type, EffectType::Appending);
    assert_eq!(rules[33].effect_on, 'ư');
}

#[test]
fn parse_append_rule() {
    let rules = bamboo_core::parse_toneless_rules('[', "__ươ");
    assert_eq!(rules.len(), 1);
    let append_rules = &rules[0].appended_rules;
    assert_eq!(append_rules.len(), 1);
    assert_eq!(append_rules[0].effect_type, EffectType::Appending);
    assert_eq!(append_rules[0].effect_on, 'ơ');

    let rules = bamboo_core::parse_toneless_rules('{', "__ƯƠ");
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
    let rules = bamboo_core::parse_toneless_rules('o', "O_Ô");
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
    let o_rules: Vec<_> = im.rules.iter().filter(|r| r.key == 'o').collect();
    assert!(!o_rules.is_empty());
    assert!(!o_rules.iter().any(|r| r.effect_type == EffectType::Appending));
}
