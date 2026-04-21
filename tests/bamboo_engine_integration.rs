use bamboo_core_rust::bamboo::{
    BambooEngine, ENGLISH_MODE, ESTD_FLAGS, IEngine, VIETNAMESE_MODE,
};
use bamboo_core_rust::rules_parser::parse_input_method;

fn new_std_engine() -> BambooEngine {
    let im = parse_input_method("Telex 2");
    BambooEngine::new(im, ESTD_FLAGS)
}

#[test]
fn process_basic_strings() {
    let mut ng = new_std_engine();
    ng.process_str("aw", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "ă");

    ng.reset();
    ng.process_str("uw", VIETNAMESE_MODE);
    ng.process_str("o", VIETNAMESE_MODE);
    ng.process_str("w", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "ươ");

    ng.reset();
    ng.process_str("chuaarn", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "chuẩn");

    ng.reset();
    ng.process_str("giamaf", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "giầm");
}

#[test]
fn process_dd_and_validity() {
    let mut ng = new_std_engine();
    ng.process_str("dd", VIETNAMESE_MODE);
    assert!(ng.is_valid(false));

    ng.reset();
    ng.process_str("ddafi", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "đài");
}

#[test]
fn process_upper_and_remove_last_char() {
    let mut ng = new_std_engine();
    ng.process_str("VIEETJ", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "VIỆT");

    ng.remove_last_char(false);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "VIỆ");

    ng.process_key('Q', VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(ENGLISH_MODE), "VIEEJQ");
}

#[test]
fn process_double_w() {
    let mut ng = new_std_engine();
    ng.process_str("ww", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(ENGLISH_MODE), "w");
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "w");
}

#[test]
fn process_toowi_case() {
    let mut ng = new_std_engine();
    ng.process_str("toowi", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "tơi");
}

#[test]
fn process_aloo_case() {
    let mut ng = new_std_engine();
    ng.process_str("aloo", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "alô");
}

#[test]
fn process_muoiwq_and_mootj() {
    let mut ng = new_std_engine();
    ng.process_str("Muoiwq", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(ENGLISH_MODE), "Muoiwq");

    ng.reset();
    ng.process_str("mootj", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "một");
}

#[test]
fn process_refresh_combo() {
    let mut ng = new_std_engine();
    ng.process_str("reff", VIETNAMESE_MODE);
    ng.process_str("resh", ENGLISH_MODE);
    assert_eq!(ng.get_processed_str(ENGLISH_MODE), "reffresh");
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "refresh");
}

#[test]
fn process_double_w2() {
    let mut ng = new_std_engine();
    ng.process_str("wiw", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "uiw");
    assert_eq!(ng.get_processed_str(ENGLISH_MODE), "wiw");
}

#[test]
fn process_duwoi() {
    let mut ng = new_std_engine();
    ng.process_str("duwoi", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "dươi");
}

#[test]
fn process_kimso_and_toorr() {
    let mut ng = new_std_engine();
    ng.process_str("kimso", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "kímo");

    ng.reset();
    ng.process_str("toorr", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "tôr");
}

#[test]
fn process_z_processing() {
    let mut ng = new_std_engine();
    ng.process_str("loz", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "loz");

    ng.reset();
    ng.process_str("losz", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "lo");
    assert_eq!(ng.get_processed_str(ENGLISH_MODE), "losz");
}

#[test]
fn restore_last_word_basic() {
    let mut ng = new_std_engine();
    ng.process_str("duwongj tooi", VIETNAMESE_MODE);
    ng.restore_last_word(false);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "tooi");
}

#[test]
fn process_double_typing_linux() {
    let mut ng = new_std_engine();
    ng.process_str("linux", VIETNAMESE_MODE);
    ng.process_str("x", VIETNAMESE_MODE);
    assert_eq!(ng.get_processed_str(VIETNAMESE_MODE), "linux");
}
