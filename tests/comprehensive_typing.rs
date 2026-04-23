use bamboo_core::{Engine, InputMethod, Mode, OutputOptions};

#[test]
fn test_complex_vietnamese_words() {
    let mut engine = Engine::new(InputMethod::telex());

    let cases = [
        ("nghieengs", "nghiếng"),
        ("thuyeets", "thuyết"),
        ("truwowjt", "trượt"),
        ("uoons", "uốn"),
        ("hoas", "hóa"),
        ("quyeets", "quyết"),
    ];

    for (input, expected) in cases {
        engine.reset();
        engine.process_str(input, Mode::Vietnamese);
        assert_eq!(engine.output(), expected, "Input: {}", input);
    }
}

#[test]
fn test_mixed_viet_english_typing() {
    let mut engine = Engine::new(InputMethod::telex());
    engine.warm_up();

    // In Vietnamese mode, "rust" becomes "rú" + "t" or "rúst" depending on timing
    let typing_flow = [
        ('h', "h"),
        ('o', "ho"),
        ('c', "hoc"),
        (' ', "hoc "),
        ('r', "hoc r"),
        ('u', "hoc ru"),
        ('s', "hoc rú"),
        ('t', "hoc rút"),
    ];

    for (key, expected_full) in typing_flow {
        engine.process_key(key, Mode::Vietnamese);
        let current = engine.get_processed_str(OutputOptions::FULL_TEXT);
        assert_eq!(current.to_lowercase(), expected_full);
    }
}

#[test]
fn test_numbers_and_punctuation_integrity() {
    let mut engine = Engine::new(InputMethod::telex());

    engine.process_str("a1s2", Mode::Vietnamese);
    assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "a1s2");
}

#[test]
fn test_dfa_state_consistency_after_backspace() {
    let mut engine = Engine::new(InputMethod::telex());

    // tieeng -> tiêng
    engine.process_str("tieeng", Mode::Vietnamese);
    assert_eq!(engine.output(), "tiêng");

    // s -> tiếng
    engine.process_key('s', Mode::Vietnamese);
    assert_eq!(engine.output(), "tiếng");

    // remove_last_char removes 's' -> should be back to "tiêng"
    engine.remove_last_char(true);
    let out = engine.output();
    assert_eq!(out, "tiêng", "After removing 's' from 'tiếng', expected 'tiêng', got '{}'", out);
}
