use bamboo_core::{Engine, InputMethod, Mode};

fn type_phrase(engine: &mut Engine, input: &str) -> String {
    engine.reset();
    let mut full = String::new();
    for ch in input.chars() {
        if ch == ' ' {
            full.push_str(&engine.output());
            full.push(' ');
            engine.reset();
        } else {
            engine.process_key(ch, Mode::Vietnamese);
        }
    }
    full.push_str(&engine.output());
    full
}

fn type_word(engine: &mut Engine, input: &str) -> String {
    engine.reset();
    for ch in input.chars() {
        engine.process_key(ch, Mode::Vietnamese);
    }
    engine.output().to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. VI-RS (ZeroX-DG) TELEX TEST DATA
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_vi_crate_simple_telex_sentences() {
    let mut e = Engine::new(InputMethod::telex_w());

    // Sentences from simple_telex.txt
    assert_eq!(type_phrase(&mut e, "tooi laf ai"), "tôi là ai");
    assert_eq!(type_phrase(&mut e, "ddaay laf ddaau"), "đây là đâu");
    assert_eq!(type_phrase(&mut e, "theem mootj cais nuwax"), "thêm một cái nữa");
    assert_eq!(type_phrase(&mut e, "vis tieenf"), "ví tiền");
    assert_eq!(type_phrase(&mut e, "CHAOf"), "CHÀO");
    assert_eq!(type_phrase(&mut e, "vow"), "vơ");
    assert_eq!(type_phrase(&mut e, "ru"), "ru");
    assert_eq!(type_phrase(&mut e, "vuown"), "vươn");
    assert_eq!(type_phrase(&mut e, "chEe"), "chÊ");
    assert_eq!(type_phrase(&mut e, "chejech"), "chệch");
    assert_eq!(type_phrase(&mut e, "vuonw"), "vươn");
    assert_eq!(type_phrase(&mut e, "hoangfr"), "hoảng");
    assert_eq!(
        type_phrase(
            &mut e,
            "dduwowcj phats trienre bowir coongj ddoofng laapj trinhf vieen hafng ddaafu VN"
        ),
        "được phát triển bởi cộng đồng lập trình viên hàng đầu VN"
    );
    assert_eq!(type_phrase(&mut e, "gif"), "gì");
    assert_eq!(type_phrase(&mut e, "duocwdj"), "được");
    assert_eq!(type_phrase(&mut e, "anw comw chuaw"), "ăn cơm chưa");
    assert_eq!(type_phrase(&mut e, "DDaay laf"), "Đây là");
    assert_eq!(type_phrase(&mut e, "xuwr lys"), "xử lý");
    assert_eq!(type_phrase(&mut e, "baay giowf"), "bây giờ");
    assert_eq!(type_phrase(&mut e, "thees giowis"), "thế giới");
    assert_eq!(type_phrase(&mut e, "hoacwj"), "hoặc");
    assert_eq!(type_phrase(&mut e, "con cuuwf"), "con cừu");
    assert_eq!(type_phrase(&mut e, "duwowis"), "dưới");
    assert_eq!(type_phrase(&mut e, "dwowis"), "dưới");
    assert_eq!(type_phrase(&mut e, "huowjsng"), "hướng");
    assert_eq!(type_phrase(&mut e, "chuyeejn"), "chuyện");
    assert_eq!(type_phrase(&mut e, "quangw"), "quăng");
    assert_eq!(type_phrase(&mut e, "tieseng viejet"), "tiếng việt");
    assert_eq!(type_phrase(&mut e, "vijete nam"), "việt nam");
    assert_eq!(type_phrase(&mut e, "gifang owi"), "giàng ơi");
    assert_eq!(type_phrase(&mut e, "Gifang owi"), "Giàng ơi");
    // Deliberately not `ươ` (skey/vi): `uow` is the mechanical Telex step to
    // `uơ`, matching Go/uvie and the documented `uow | truowng | trương`
    // mapping. See `telex_horn_on_o_when_nothing_follows_uo` for why the
    // dictionary does not overrule preedit mechanics here.
    assert_eq!(type_phrase(&mut e, "uow"), "uơ");
    assert_eq!(type_phrase(&mut e, "uwo"), "ưo");
    assert_eq!(type_phrase(&mut e, "uwon"), "ươn");
    assert_eq!(type_phrase(&mut e, "huwo"), "hưo");
    assert_eq!(type_phrase(&mut e, "huow"), "huơ");
    assert_eq!(type_phrase(&mut e, "thuowr"), "thuở");
    assert_eq!(type_phrase(&mut e, "dduwocj"), "được");
    assert_eq!(type_phrase(&mut e, "truowcs"), "trước");
    assert_eq!(type_phrase(&mut e, "ngoao"), "ngoao");
    assert_eq!(type_phrase(&mut e, "ngoeo"), "ngoeo");
    assert_eq!(type_phrase(&mut e, "ngoaos"), "ngoáo");
    assert_eq!(type_phrase(&mut e, "ngoseo"), "ngoéo");
    assert_eq!(type_phrase(&mut e, "giwax"), "giữa");
    assert_eq!(type_phrase(&mut e, "trwo"), "trưo");
    assert_eq!(type_phrase(&mut e, "trwongf"), "trường");
    assert_eq!(type_phrase(&mut e, "woi"), "ươi");
    assert_eq!(type_phrase(&mut e, "cwoif"), "cười");
    assert_eq!(type_phrase(&mut e, "hufwong"), "hường");
    assert_eq!(type_phrase(&mut e, "chuawr"), "chửa");
    assert_eq!(type_phrase(&mut e, "waf"), "ừa");
    assert_eq!(type_phrase(&mut e, "ww"), "w");
    assert_eq!(type_phrase(&mut e, "w"), "ư");
    assert_eq!(type_phrase(&mut e, "W"), "Ư");
}

#[test]
fn test_vi_crate_simple_vni_sentences() {
    let mut e = Engine::new(InputMethod::vni());

    assert_eq!(type_phrase(&mut e, "toi6 la2 ai"), "tôi là ai");
    assert_eq!(type_phrase(&mut e, "day96 la2 dau96"), "đây là đâu");
    assert_eq!(type_phrase(&mut e, "them6 mot65 cai1 nua74"), "thêm một cái nữa");
    assert_eq!(type_phrase(&mut e, "vi1 tien62"), "ví tiền");
    assert_eq!(type_phrase(&mut e, "chao2"), "chào");
    assert_eq!(type_phrase(&mut e, "vo7"), "vơ");
    assert_eq!(type_phrase(&mut e, "ru"), "ru");
    assert_eq!(type_phrase(&mut e, "vuon7"), "vươn");
    assert_eq!(type_phrase(&mut e, "chE6"), "chÊ");
    assert_eq!(type_phrase(&mut e, "a68"), "ă");
    assert_eq!(type_phrase(&mut e, "che6ch5"), "chệch");
    assert_eq!(type_phrase(&mut e, "vuo7n"), "vươn");
    assert_eq!(type_phrase(&mut e, "hoang23"), "hoảng");
    assert_eq!(type_phrase(&mut e, "gi2"), "gì");
    assert_eq!(type_phrase(&mut e, "vi5e6t nam"), "việt nam");
}

#[test]
fn test_vi_crate_non_vietnamese_passthrough() {
    let mut e = Engine::new(InputMethod::telex());

    assert_eq!(type_word(&mut e, "samuwrite"), "samuwrite");
    assert_eq!(type_word(&mut e, "cat"), "cat");
    assert_eq!(type_word(&mut e, "doggo"), "doggo");
    assert_eq!(type_word(&mut e, "land"), "land");
    assert_eq!(type_word(&mut e, "overflow"), "overflow");
    assert_eq!(type_word(&mut e, "old"), "old");
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. UVIE (thuupx) EDGE CASES & PHONOTACTICS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_uvie_double_tone_and_undo() {
    let mut e = Engine::new(InputMethod::telex());

    // Double tone at end of word: bass -> bas
    assert_eq!(type_word(&mut e, "bass"), "bas");

    // Double w undo: aww -> aw, ddoww -> đow
    assert_eq!(type_word(&mut e, "aww"), "aw");
    assert_eq!(type_word(&mut e, "ddoww"), "đow");

    // English words with tone keys
    assert_eq!(type_word(&mut e, "stress"), "stress");
    assert_eq!(type_word(&mut e, "jazz"), "jazz");
    assert_eq!(type_word(&mut e, "txt"), "txt");
    assert_eq!(type_word(&mut e, "rx"), "rx");
    assert_eq!(type_word(&mut e, "sx"), "sx");
}

#[test]
fn test_uvie_tone_placement_rules() {
    let mut e = Engine::new(InputMethod::telex());

    // ươi -> tone on ơ
    assert_eq!(type_word(&mut e, "huowis"), "hưới");
    // ươn -> tone on ơ
    assert_eq!(type_word(&mut e, "huowns"), "hướn");
    // âu -> tone on â
    assert_eq!(type_word(&mut e, "daauf"), "dầu");
    // ây -> tone on â
    assert_eq!(type_word(&mut e, "daays"), "dấy");
}

#[test]
fn test_uvie_free_style_typing() {
    let mut e = Engine::new(InputMethod::telex());

    assert_eq!(type_word(&mut e, "tieengs"), "tiếng");
    assert_eq!(type_word(&mut e, "moiws"), "mới");
    assert_eq!(type_word(&mut e, "dduowcj"), "được");
    assert_eq!(type_word(&mut e, "nguowif"), "người");
    assert_eq!(type_word(&mut e, "nawms"), "nắm");
    assert_eq!(type_word(&mut e, "khoongf"), "khồng");
    assert_eq!(type_word(&mut e, "ddeepj"), "đệp");
    assert_eq!(type_word(&mut e, "chaof"), "chào");
}

#[test]
fn test_uvie_english_passthrough_words() {
    let mut e = Engine::with_config(
        InputMethod::telex_w(),
        bamboo_core::Config { auto_correct: true, ..Default::default() },
    );

    assert_eq!(type_word(&mut e, "electronic"), "electronic");
    assert_eq!(type_word(&mut e, "depend"), "depend");
    assert_eq!(type_word(&mut e, "banana"), "banana");
    assert_eq!(type_word(&mut e, "wwork"), "work");
    assert_eq!(type_word(&mut e, "reboot"), "reboot");
}

#[test]
fn test_uvie_valid_and_invalid_onsets() {
    let mut e = Engine::new(InputMethod::telex());

    // Valid onsets
    assert_eq!(type_word(&mut e, "tras"), "trá");
    assert_eq!(type_word(&mut e, "phas"), "phá");
    assert_eq!(type_word(&mut e, "khas"), "khá");
    assert_eq!(type_word(&mut e, "nghes"), "nghé");

    // Tone restrictions on stopped codas (ch, c, p, t): only sắc/nặng
    assert_eq!(type_word(&mut e, "achs"), "ách");
    assert_eq!(type_word(&mut e, "achj"), "ạch");
    assert_eq!(type_word(&mut e, "ieecs"), "iếc");
    assert_eq!(type_word(&mut e, "ieecj"), "iệc");
    assert_eq!(type_word(&mut e, "aaps"), "ấp");
    assert_eq!(type_word(&mut e, "aapj"), "ập");
}
