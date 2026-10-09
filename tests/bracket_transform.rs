//! `[`, `]`, `{` and `}` type `ơ`, `ư`, `Ơ` and `Ư`, depending on `BracketMode`.
//! Expected strings come from the Go bamboo-core used by fcitx5-lotus (`SetBracketTransformMode`).

use bamboo_core::{BracketMode, Config, Engine, InputMethod, Mode, OutputOptions};

// auto_correct would turn syllables such as `ơa` back into raw keys; Go leaves that to the caller.
fn engine(im: InputMethod, mode: BracketMode) -> Engine {
    Engine::with_config(im, Config::builder().bracket_mode(mode).auto_correct(false).build())
}

// Full text, because a bracket the engine does not process is a word break and gets committed.
fn typed(im: InputMethod, mode: BracketMode, keys: &str) -> String {
    let mut engine = engine(im, mode);
    engine.process_str(keys, Mode::Vietnamese);
    engine.get_processed_str(OutputOptions::FULL_TEXT)
}

fn check(im: &InputMethod, mode: BracketMode, cases: &[(&str, &str)]) {
    for &(keys, expected) in cases {
        assert_eq!(typed(im.clone(), mode, keys), expected, "{mode:?} keys {keys:?}");
    }
}

#[test]
fn default_is_disabled() {
    assert_eq!(Config::default().bracket_mode, BracketMode::Disabled);
    let mut engine = Engine::new(InputMethod::telex());
    engine.process_str("m[f", Mode::Vietnamese);
    assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "m[f");
}

#[test]
fn telex_disabled() {
    check(
        &InputMethod::telex(),
        BracketMode::Disabled,
        &[("[", "["), ("{", "{"), ("t[", "t["), ("m[f", "m[f"), ("d]s", "d]s"), ("[[", "[[")],
    );
}

#[test]
fn telex_everywhere() {
    check(
        &InputMethod::telex(),
        BracketMode::Everywhere,
        &[
            ("[", "ơ"),
            ("]", "ư"),
            ("{", "Ơ"),
            ("}", "Ư"),
            ("[[", "["),
            ("]]", "]"),
            ("{{", "{"),
            ("}}", "}"),
            ("[{", "["),
            ("{[", "{"),
            ("][", "ươ"),
            ("t[", "tơ"),
            ("t]", "tư"),
            ("m[f", "mờ"),
            ("d]s", "dứ"),
            ("{s", "Ớ"),
            ("}f", "Ừ"),
            ("[s", "ớ"),
            ("h[[", "h["),
            ("gi]", "giư"),
            ("ch]a", "chưa"),
            ("tr]ng", "trưng"),
            ("t]ngf", "từng"),
            ("ng][i", "ngươi"),
            ("NG}{I", "NGƯƠI"),
        ],
    );
}

#[test]
fn telex_non_start() {
    check(
        &InputMethod::telex(),
        BracketMode::NonStart,
        &[
            ("[", "["),
            ("]", "]"),
            ("{s", "{s"),
            ("a[", "aơ"),
            ("t[", "tơ"),
            ("m[f", "mờ"),
            ("d]s", "dứ"),
            ("h[[", "h["),
            ("tr]ng", "trưng"),
            ("NG}{I", "NGƯƠI"),
        ],
    );
}

#[test]
fn vni_everywhere() {
    check(
        &InputMethod::vni(),
        BracketMode::Everywhere,
        &[("[", "ơ"), ("{", "Ơ"), ("[[", "["), ("tr]ng", "trưng"), ("NG}{I", "NGƯƠI")],
    );
}

// Telex 2 maps the brackets itself; the mode must not change it.
#[test]
fn telex_2_unchanged() {
    let keys = ["[", "]", "{", "}", "[[", "{{", "[{", "t[", "m[f", "tr]ng", "NG}{I", "h[[a"];
    for mode in [BracketMode::NonStart, BracketMode::Everywhere] {
        for keys in keys {
            assert_eq!(
                typed(InputMethod::telex_2(), mode, keys),
                typed(InputMethod::telex_2(), BracketMode::Disabled, keys),
                "{mode:?} keys {keys:?}"
            );
        }
    }
}

#[test]
fn can_process_key() {
    let brackets = ['[', ']', '{', '}'];
    let check = |im: InputMethod, mode: BracketMode, prefix: &str, expected: bool| {
        let mut engine = engine(im, mode);
        engine.process_str(prefix, Mode::Vietnamese);
        for key in brackets {
            assert_eq!(engine.can_process_key(key), expected, "{mode:?} {prefix:?} {key:?}");
        }
        for key in ['a', 'A', 'z', 'w'] {
            assert!(engine.can_process_key(key), "{mode:?} {key:?}");
        }
        for key in ['€', ',', '1', ' '] {
            assert!(!engine.can_process_key(key), "{mode:?} {key:?}");
        }
    };
    check(InputMethod::telex(), BracketMode::Disabled, "", false);
    check(InputMethod::telex(), BracketMode::Disabled, "a", false);
    check(InputMethod::telex(), BracketMode::NonStart, "", false);
    check(InputMethod::telex(), BracketMode::NonStart, "a", true);
    check(InputMethod::telex(), BracketMode::Everywhere, "", true);
    check(InputMethod::telex(), BracketMode::Everywhere, "a", true);
    check(InputMethod::telex_2(), BracketMode::Disabled, "", true);

    let vni = engine(InputMethod::vni(), BracketMode::Disabled);
    assert!(vni.can_process_key('1'));
}

#[test]
fn mode_survives_flags() {
    for mode in [BracketMode::Disabled, BracketMode::NonStart, BracketMode::Everywhere] {
        let config = Config::builder().bracket_mode(mode).std_tone_style(false).build();
        assert_eq!(Config::from_flags(config.to_flags()), config);
    }
    assert_eq!(Config::from_flags(0x07 | 0x20).bracket_mode, BracketMode::NonStart);
    assert_eq!(Config::from_flags(0x07 | 0x60).bracket_mode, BracketMode::Everywhere);
    assert_eq!(Config::from_flags(0x1f).bracket_mode, BracketMode::Disabled);
}

#[test]
fn set_config_changes_mode() {
    let mut engine = engine(InputMethod::telex(), BracketMode::Everywhere);
    engine.process_str("t[", Mode::Vietnamese);
    assert_eq!(engine.output(), "tơ");
    engine.reset();
    engine.set_config(Config::default());
    engine.process_str("t[", Mode::Vietnamese);
    assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "t[");
}

// The second word reuses cached transitions; `{` must still come out upper case.
#[test]
fn cached_transitions_keep_case() {
    let mut engine = engine(InputMethod::telex(), BracketMode::Everywhere);
    for _ in 0..2 {
        engine.process_str("t{ ", Mode::Vietnamese);
    }
    engine.process_str("t[", Mode::Vietnamese);
    assert_eq!(engine.get_processed_str(OutputOptions::FULL_TEXT), "tƠ tƠ tơ");
}
