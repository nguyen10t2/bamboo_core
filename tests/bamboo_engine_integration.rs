use bamboo_core::{Engine, InputMethod, Mode, OutputOptions};

fn new_std_engine() -> Engine {
    Engine::new(InputMethod::telex_2())
}

#[test]
fn process_basic_strings() {
    let mut ng = new_std_engine();
    ng.process_str("aw", Mode::Vietnamese);
    assert_eq!(ng.output(), "ă");

    ng.reset();
    ng.process_str("uw", Mode::Vietnamese);
    ng.process_str("o", Mode::Vietnamese);
    ng.process_str("w", Mode::Vietnamese);
    assert_eq!(ng.output(), "ươ");

    ng.reset();
    ng.process_str("chuaarn", Mode::Vietnamese);
    assert_eq!(ng.output(), "chuẩn");

    ng.reset();
    ng.process_str("giamaf", Mode::Vietnamese);
    assert_eq!(ng.output(), "giầm");
}

#[test]
fn process_dd_and_validity() {
    let mut ng = new_std_engine();
    ng.process_str("dd", Mode::Vietnamese);
    assert!(ng.is_valid(false));

    ng.reset();
    ng.process_str("ddafi", Mode::Vietnamese);
    assert_eq!(ng.output(), "đài");
}

#[test]
fn process_upper_and_remove_last_char() {
    let mut ng = new_std_engine();
    ng.process_str("VIEETJ", Mode::Vietnamese);
    assert_eq!(ng.output(), "VIỆT");

    ng.remove_last_char(false);
    assert_eq!(ng.output(), "VIỆ");

    ng.process_key('Q', Mode::Vietnamese);
    assert_eq!(ng.get_processed_str(OutputOptions::RAW), "VIEEJQ");
}

#[test]
fn process_double_w() {
    let mut ng = new_std_engine();
    ng.process_str("ww", Mode::Vietnamese);
    assert_eq!(ng.get_processed_str(OutputOptions::RAW), "w");
    assert_eq!(ng.output(), "w");
}

#[test]
fn process_toowi_case() {
    let mut ng = new_std_engine();
    ng.process_str("toowi", Mode::Vietnamese);
    assert_eq!(ng.output(), "tơi");
}

#[test]
fn process_aloo_case() {
    let mut ng = new_std_engine();
    ng.process_str("alo", Mode::Vietnamese);
    assert_eq!(
        ng.get_processed_str(OutputOptions::RAW | OutputOptions::FULL_TEXT),
        "alo"
    );
    assert_eq!(ng.output(), "alo");

    ng.process_str("o", Mode::Vietnamese);
    // Telex 'o' should act as a mark key here (not a literal append-only key).
    assert_eq!(
        ng.get_processed_str(OutputOptions::RAW | OutputOptions::FULL_TEXT),
        "aloo"
    );
    assert_eq!(ng.output(), "alô");
}

#[test]
fn process_muoiwq_and_mootj() {
    let mut ng = new_std_engine();
    ng.process_str("Muoiwq", Mode::Vietnamese);
    assert_eq!(ng.get_processed_str(OutputOptions::RAW), "Muoiwq");

    ng.reset();
    ng.process_str("mootj", Mode::Vietnamese);
    assert_eq!(ng.output(), "một");
}

#[test]
fn process_refresh_combo() {
    let mut ng = new_std_engine();
    ng.process_str("reff", Mode::Vietnamese);
    ng.process_str("resh", Mode::English);
    assert_eq!(ng.get_processed_str(OutputOptions::RAW), "reffresh");
    assert_eq!(ng.output(), "refresh");
}

#[test]
fn process_double_w2() {
    let mut ng = new_std_engine();
    ng.process_str("wiw", Mode::Vietnamese);
    assert_eq!(ng.output(), "uiw");
    assert_eq!(ng.get_processed_str(OutputOptions::RAW), "wiw");
}

#[test]
fn process_duwoi() {
    let mut ng = new_std_engine();
    ng.process_str("duwoi", Mode::Vietnamese);
    assert_eq!(ng.output(), "dươi");
}

#[test]
fn process_kimso_and_toorr() {
    let mut ng = new_std_engine();
    ng.process_str("kimso", Mode::Vietnamese);
    assert_eq!(ng.output(), "kímo");

    ng.reset();
    ng.process_str("toorr", Mode::Vietnamese);
    assert_eq!(ng.output(), "tôr");
}

#[test]
fn process_z_processing() {
    let mut ng = new_std_engine();
    ng.process_str("loz", Mode::Vietnamese);
    assert_eq!(ng.output(), "loz");

    ng.reset();
    ng.process_str("losz", Mode::Vietnamese);
    assert_eq!(ng.output(), "lo");
    assert_eq!(ng.get_processed_str(OutputOptions::RAW), "losz");
}

#[test]
fn restore_last_word_basic() {
    let mut ng = new_std_engine();
    ng.process_str("duwongj tooi", Mode::Vietnamese);
    ng.restore_last_word(false);
    assert_eq!(ng.output(), "tooi");
}

#[test]
fn process_double_typing_linux() {
    let mut ng = new_std_engine();
    ng.process_str("linux", Mode::Vietnamese);
    ng.process_str("x", Mode::Vietnamese);
    assert_eq!(ng.output(), "linux");
}
