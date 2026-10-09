use bamboo_core::{Engine, InputMethod, Mode};

fn check(im: InputMethod, cases: &[(&str, &str)]) {
    for (keys, expected) in cases {
        let mut engine = Engine::new(im.clone());
        engine.process_str(keys, Mode::Vietnamese);
        assert_eq!(engine.output(), *expected, "keys: {keys}");
    }
}

#[test]
fn telex_horn_on_o_when_nothing_follows_uo() {
    // `uow` types the documented Telex diphthong step `uơ` (as in `truowng`
    // -> `trương`): the horn lands on the nearest vowel `o`, and a following
    // coda still resolves it (`uowng` -> `ương`). A mid-word preedit is not
    // judged by dictionary rimes (`uơ` is no standalone word, but neither is
    // any other unfinished preedit); what matters is that completed words
    // come out right (`truowcs` -> `trước`, see the vi/uvie suite).
    check(InputMethod::telex(), &[("khuow", "khuơ"), ("uowr", "uở")]);
}

#[test]
fn telex_horn_spreads_to_u_when_letter_follows() {
    check(InputMethod::telex(), &[("huouw", "hươu"), ("thuoiwx", "thưỡi")]);
}

#[test]
fn telex_uow_shortcut_still_works() {
    check(
        InputMethod::telex(),
        &[("huwowu", "hươu"), ("truwowjt", "trượt"), ("dduowngf", "đường"), ("nguwowif", "người")],
    );
}

#[test]
fn vni_horn_placement_after_uo() {
    check(InputMethod::vni(), &[("khuo7", "khuơ"), ("uo73", "uở"), ("huou7", "hươu")]);
}
