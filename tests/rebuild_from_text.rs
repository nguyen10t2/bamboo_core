//! `Engine::rebuild_from_text`, checked against the Go bamboo-core
//! `RebuildEngineFromText` (rebuild_test.go and the same calls replayed on Go).

use bamboo_core::{Config, Engine, InputMethod, Mode, OutputOptions};

// The Go core has no auto-correct, so compare with it turned off.
fn engine(im: InputMethod) -> Engine {
    Engine::with_config(im, Config::builder().auto_correct(false).build())
}

fn full_text(e: &Engine) -> String {
    e.get_processed_str(OutputOptions::FULL_TEXT)
}

/// Rebuilds from `text`, then replays `ops`: `<` removes the last output
/// char, `!` restores the last word, anything else is a Vietnamese key.
/// Returns (full text, active word).
fn replay(im: InputMethod, text: &str, ops: &str) -> (String, String) {
    let mut e = engine(im);
    e.rebuild_from_text(text);
    for op in ops.chars() {
        match op {
            '<' => e.remove_last_output_char(),
            '!' => e.restore_last_word(false),
            k => e.process_key(k, Mode::Vietnamese),
        }
    }
    (full_text(&e), e.output().into_owned())
}

#[test]
fn rebuild_keeps_text() {
    let cases = [
        "goo",
        "chào",
        "việt",
        "google",
        "đường",
        "người",
        "as",
        "được",
        "những",
        "ước",
        "ươi",
        "Việt",
        "OO",
        "DD",
        "Nội",
        "vãi",
        "vãi.",
        "vãi ",
        "vãi. ",
        "chào.",
        "chào. ",
        "chào, ",
        "chào. Xin",
        "vãi, ",
        "vãi! ",
        "vãi? ",
        "vãi; ",
        "vãi: ",
        "Đường",
        "",
    ];
    for text in cases {
        let mut e = engine(InputMethod::telex());
        e.rebuild_from_text(text);
        assert_eq!(full_text(&e), text, "rebuild {text:?}");
    }
}

#[test]
fn rebuild_resets_previous_state() {
    let mut e = engine(InputMethod::telex());
    e.process_str("xin chaof", Mode::Vietnamese);
    for text in ["vãi", "vãi.", "vãi ", "vãi. ", "chào.", "chào. "] {
        e.rebuild_from_text(text);
        assert_eq!(full_text(&e), text, "rebuild {text:?}");
    }
}

#[test]
fn rebuild_puts_tone_where_the_engine_would() {
    // Go keeps only the last tone and places it by the tone style.
    assert_eq!(replay(InputMethod::telex(), "hoà", ""), ("hòa".into(), "hòa".into()));
    assert_eq!(replay(InputMethod::telex(), "hoà xin", ""), ("hòa xin".into(), "xin".into()));
}

#[test]
fn active_word_is_text_after_last_break() {
    let cases = [
        ("tiếng", "tiếng"),
        ("vãi.", ""),
        ("vãi. ", ""),
        ("hello world", "world"),
        ("chào. Xin", "Xin"),
        ("a1b", "b"),
        ("ab@cd", "cd"),
    ];
    for (text, word) in cases {
        assert_eq!(replay(InputMethod::telex(), text, "").1, word, "rebuild {text:?}");
    }
}

#[test]
fn keys_after_rebuild_match_go() {
    // (text, keys, full text, active word) as the Go core returns them.
    let cases = [
        ("go", "s", "gó", "gó"),
        ("goo", "s", "góo", "góo"),
        ("tieng", "s", "tiéng", "tiéng"),
        ("tiếng", "f", "tiềng", "tiềng"),
        ("tiếng", "s", "tiêngs", "tiêngs"),
        ("tiếng", "ff", "tiêngf", "tiêngf"),
        ("tiêng", "e", "tienge", "tienge"),
        ("tiếng", "w", "tiếngw", "tiếngw"),
        ("Việt", "j", "Viêtj", "Viêtj"),
        ("Việt", "f", "Việtf", "Việtf"),
        ("hoà", "f", "hoaf", "hoaf"),
        ("vãi.", "s", "vãi.s", "s"),
        ("vãi. ", "s", "vãi. s", "s"),
        ("chào. Xin", "f", "chào. Xìn", "Xìn"),
        ("đường", "f", "đươngf", "đươngf"),
        ("đ", "d", "dd", "dd"),
        ("d", "d", "đ", "đ"),
        ("tuong", "w", "tương", "tương"),
        ("tương", "w", "tuongw", "tuongw"),
        ("nguyen", "ee", "nguyene", "nguyene"),
        ("nguyên", "x", "nguyễn", "nguyễn"),
        ("người", "j", "ngượi", "ngượi"),
        ("NGƯỜI", "s", "NGƯỚI", "NGƯỚI"),
        ("ĐƯỜNG", "f", "ĐƯƠNGf", "ĐƯƠNGf"),
        ("quý", "f", "quỳ", "quỳ"),
        ("gìn", "s", "gín", "gín"),
        ("giờ", "z", "giơ", "giơ"),
        ("toán", "z", "toan", "toan"),
        ("ab@cd", "s", "ab@cds", "cds"),
        ("a1b", "s", "a1bs", "bs"),
        ("", "s", "s", "s"),
    ];
    for (text, keys, full, word) in cases {
        assert_eq!(
            replay(InputMethod::telex(), text, keys),
            (full.into(), word.into()),
            "rebuild {text:?} then {keys:?}"
        );
    }
}

#[test]
fn keys_after_rebuild_match_go_vni() {
    let cases = [("tiếng", "2", "tiềng"), ("tieng", "1", "tiéng"), ("duong", "7", "dương")];
    for (text, keys, word) in cases {
        let (full, active) = replay(InputMethod::vni(), text, keys);
        assert_eq!((full.as_str(), active.as_str()), (word, word), "rebuild {text:?}+{keys:?}");
    }
}

#[test]
fn keys_after_rebuild_match_typing() {
    // The rebuilt word reacts to keys like the same word typed in Telex.
    // "tieengse" gives "tienge" here but "tiénge" in Go, typed or rebuilt.
    let cases = [
        ("tiếng", "tieengs", "f"),
        ("tiếng", "tieengs", "s"),
        ("tiếng", "tieengs", "e"),
        ("Việt", "Vieetj", "j"),
        ("đường", "dduwowngf", "f"),
        ("nguyên", "nguyeen", "x"),
        ("người", "nguwowfi", "j"),
        ("tương", "tuowng", "w"),
    ];
    for (text, typed, keys) in cases {
        let mut e = engine(InputMethod::telex());
        e.process_str(typed, Mode::Vietnamese);
        e.process_str(keys, Mode::Vietnamese);
        let want = e.output().into_owned();
        assert_eq!(
            replay(InputMethod::telex(), text, keys).1,
            want,
            "{text:?} vs {typed:?}+{keys:?}"
        );
    }
}

#[test]
fn backspace_after_rebuild_keeps_marks() {
    let cases = [
        ("tiếng", "<", "tiến"),
        ("tiếng", "<<", "tiế"),
        ("Việt", "<<", "Vi"),
        ("đường", "<", "đườn"),
    ];
    for (text, ops, want) in cases {
        assert_eq!(
            replay(InputMethod::telex(), text, ops).1,
            want,
            "rebuild {text:?} then {ops:?}"
        );
    }
    assert_eq!(replay(InputMethod::telex(), "chào, xin", "<").0, "chào, xi");
}

#[test]
fn restore_after_rebuild_drops_tone_and_marks_from_keys() {
    // Go has no keystrokes for a rebuilt word: restoring gives its letters.
    assert_eq!(replay(InputMethod::telex(), "tieng", "!").1, "tieng");
    assert_eq!(replay(InputMethod::telex(), "tiếng", "!").1, "tiêng");
}

#[test]
fn word_too_long_for_composition_stays_text() {
    let text = "xin internationalization";
    let mut e = engine(InputMethod::telex());
    e.rebuild_from_text(text);
    assert_eq!(full_text(&e), text);
    assert_eq!(e.output(), "");
    assert_eq!(e.get_processed_str(OutputOptions::RAW | OutputOptions::FULL_TEXT), text);
}

#[test]
fn rebuild_keeps_raw_keys_of_committed_words() {
    let mut e = engine(InputMethod::telex());
    e.rebuild_from_text("xin tiếng");
    // Go keeps a toned ê whole, so its key is ê, not e.
    assert_eq!(e.get_processed_str(OutputOptions::RAW | OutputOptions::FULL_TEXT), "xin tiêng");
}
