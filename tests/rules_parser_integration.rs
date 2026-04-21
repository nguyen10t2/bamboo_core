use bamboo_core_rust::rules_parser::{
    parse_input_method, parse_rules, parse_toneless_rules, EffectType, Mark, Tone,
};

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
