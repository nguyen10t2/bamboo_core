use bamboo_core::{Config, Engine, InputMethod, Mode, OutputOptions};

#[test]
fn test_buffer_overflow_auto_commit() {
    let mut engine = Engine::new(InputMethod::telex());

    // Type a string that won't trigger marks (all 'q')
    // MAX is 16. Type 20.
    let long_input = "qqqqqqqqqqqqqqqqqqqq";
    engine.process_str(long_input, Mode::Vietnamese);

    let output = engine.get_processed_str(OutputOptions::FULL_TEXT);
    // Should contain at least 16 chars (first chunk)
    assert!(output.len() >= 16);
    // Check if characters are preserved
    assert!(output.contains("qqqqqqqqqqqqqqqq"));
}

#[test]
fn test_complex_vietnamese_words() {
    let mut engine = Engine::new(InputMethod::telex());
    // Use standard tone style for 'khuỵu'
    engine.set_config(Config::default()); // ensure default

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
fn test_backspace_recovery() {
    let mut engine = Engine::new(InputMethod::telex());

    // Test basic backspace
    engine.process_str("as", Mode::Vietnamese);
    assert_eq!(engine.output(), "á");
    engine.remove_last_char(true);
    assert_eq!(engine.output(), "a");

    // Test complex backspace (marks)
    engine.reset();
    engine.process_str("tieengs", Mode::Vietnamese);
    assert_eq!(engine.output(), "tiếng");
    engine.remove_last_char(true); // removes 's'
    assert_eq!(engine.output(), "tiêng");
    engine.remove_last_char(true); // removes 'g'
    assert_eq!(engine.output(), "tiên");
}
