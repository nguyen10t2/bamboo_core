use bamboo_core::{Engine, Mode};

fn new_std_engine() -> bamboo_core::BambooEngine {
    let im = bamboo_core::parse_input_method("Telex 2");
    bamboo_core::BambooEngine::new(im, bamboo_core::ESTD_FLAGS)
}

#[test]
fn process_basic_strings() {
    let mut ng = new_std_engine();
    ng.process_str("aw", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "ă");

    ng.reset();
    ng.process_str("uw", Mode::VIETNAMESE);
    ng.process_str("o", Mode::VIETNAMESE);
    ng.process_str("w", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "ươ");

    ng.reset();
    ng.process_str("chuaarn", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "chuẩn");

    ng.reset();
    ng.process_str("giamaf", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "giầm");
}

#[test]
fn process_dd_and_validity() {
    let mut ng = new_std_engine();
    ng.process_str("dd", Mode::VIETNAMESE);
    assert!(ng.is_valid(false));

    ng.reset();
    ng.process_str("ddafi", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "đài");
}

#[test]
fn process_upper_and_remove_last_char() {
    let mut ng = new_std_engine();
    ng.process_str("VIEETJ", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "VIỆT");

    ng.remove_last_char(false);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "VIỆ");

    ng.process_key('Q', Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::ENGLISH), "VIEEJQ");
}

#[test]
fn process_double_w() {
    let mut ng = new_std_engine();
    ng.process_str("ww", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::ENGLISH), "w");
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "w");
}

#[test]
fn process_toowi_case() {
    let mut ng = new_std_engine();
    ng.process_str("toowi", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "tơi");
}

#[test]
fn process_aloo_case() {
    let mut ng = new_std_engine();
    ng.process_str("alo", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::ENGLISH | Mode::FULL_TEXT), "alo");
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "alo");

    ng.process_str("o", Mode::VIETNAMESE);
    // Telex 'o' should act as a mark key here (not a literal append-only key).
    assert_eq!(ng.get_processed_str(Mode::ENGLISH | Mode::FULL_TEXT), "aloo");
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "alô");
}

#[test]
fn process_muoiwq_and_mootj() {
    let mut ng = new_std_engine();
    ng.process_str("Muoiwq", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::ENGLISH), "Muoiwq");

    ng.reset();
    ng.process_str("mootj", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "một");
}

#[test]
fn process_refresh_combo() {
    let mut ng = new_std_engine();
    ng.process_str("reff", Mode::VIETNAMESE);
    ng.process_str("resh", Mode::ENGLISH);
    assert_eq!(ng.get_processed_str(Mode::ENGLISH), "reffresh");
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "refresh");
}

#[test]
fn process_double_w2() {
    let mut ng = new_std_engine();
    ng.process_str("wiw", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "uiw");
    assert_eq!(ng.get_processed_str(Mode::ENGLISH), "wiw");
}

#[test]
fn process_duwoi() {
    let mut ng = new_std_engine();
    ng.process_str("duwoi", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "dươi");
}

#[test]
fn process_kimso_and_toorr() {
    let mut ng = new_std_engine();
    ng.process_str("kimso", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "kímo");

    ng.reset();
    ng.process_str("toorr", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "tôr");
}

#[test]
fn process_z_processing() {
    let mut ng = new_std_engine();
    ng.process_str("loz", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "loz");

    ng.reset();
    ng.process_str("losz", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "lo");
    assert_eq!(ng.get_processed_str(Mode::ENGLISH), "losz");
}

#[test]
fn restore_last_word_basic() {
    let mut ng = new_std_engine();
    ng.process_str("duwongj tooi", Mode::VIETNAMESE);
    ng.restore_last_word(false);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "tooi");
}

#[test]
fn process_double_typing_linux() {
    let mut ng = new_std_engine();
    ng.process_str("linux", Mode::VIETNAMESE);
    ng.process_str("x", Mode::VIETNAMESE);
    assert_eq!(ng.get_processed_str(Mode::VIETNAMESE), "linux");
}
