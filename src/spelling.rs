use crate::utils::add_mark_to_toneless_char;

static FIRST_CONSONANT_SEQS: &[&str] = &[
    "b d đ g gh m n nh p ph r s t tr v z",
    "c h k kh qu th",
    "ch gi l ng ngh x",
    "đ l",
    "h",
];

static VOWEL_SEQS: &[&str] = &[
    "ê i ua uê uy y",
    "a iê oa uyê yê",
    "â ă e o oo ô ơ oe u ư uâ uô ươ",
    "oă",
    "uơ",
    "ai ao au âu ay ây eo êu ia iêu iu oai oao oay oeo oi ôi ơi ưa uây ui ưi uôi ươi ươu ưu uya uyu yêu",
    "ă",
    "i",
];

static LAST_CONSONANT_SEQS: &[&str] = &["ch nh", "c ng", "m n p t", "k", "c"];

const CV_ALLOWED_MASKS: [u16; 5] = [
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 5),
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5),
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 5),
    1 << 6,
    1 << 7,
];

const VC_ALLOWED_MASKS: [u16; 8] = [
    (1 << 0) | (1 << 2),
    (1 << 0) | (1 << 1) | (1 << 2),
    (1 << 1) | (1 << 2),
    (1 << 1) | (1 << 2),
    0,
    0,
    1 << 3,
    1 << 4,
];

// ================= LOOKUP (LOW-ALLOC MASK) =================

fn lookup_mask(
    seq: &[&str],
    input: &str,
    input_is_full: bool,
    input_is_complete: bool,
) -> u16 {
    let input_len = input.chars().count();
    let mut ret = 0u16;

    for (index, row) in seq.iter().enumerate() {
        for token in row.split_whitespace() {
            let token_len = token.chars().count();
            if token_len < input_len {
                continue;
            }
            if input_is_full && token_len > input_len {
                continue;
            }

            let mut is_match = true;
            for (ic, tc) in input.chars().zip(token.chars()) {
                if ic != tc
                    && (input_is_complete
                        || add_mark_to_toneless_char(tc, 0) != ic)
                {
                    is_match = false;
                    break;
                }
            }

            if is_match {
                ret |= 1u16 << index;
                break;
            }
        }
    }

    ret
}

fn is_valid_cv(fc_mask: u16, vo_mask: u16) -> bool {
    let mut mask = fc_mask;
    while mask != 0 {
        let idx = mask.trailing_zeros() as usize;
        if idx < CV_ALLOWED_MASKS.len()
            && (CV_ALLOWED_MASKS[idx] & vo_mask) != 0
        {
            return true;
        }
        mask &= mask - 1;
    }
    false
}

fn is_valid_vc(vo_mask: u16, lc_mask: u16) -> bool {
    let mut mask = vo_mask;
    while mask != 0 {
        let idx = mask.trailing_zeros() as usize;
        if idx < VC_ALLOWED_MASKS.len()
            && (VC_ALLOWED_MASKS[idx] & lc_mask) != 0
        {
            return true;
        }
        mask &= mask - 1;
    }
    false
}

/// Validates a Vietnamese syllable based on its Consonant-Vowel-Consonant (CVC) structure.
///
/// # Arguments
///
/// * `fc` - First consonant(s).
/// * `vo` - Vowel(s).
/// * `lc` - Last consonant(s).
/// * `full` - Whether the input is considered complete (affects strictness of matching).
///
/// # Returns
///
/// `true` if the combination forms a valid (or potentially valid) Vietnamese syllable.
pub fn is_valid_cvc(fc: &str, vo: &str, lc: &str, full: bool) -> bool {
    let fc_mask = if !fc.is_empty() {
        let m =
            lookup_mask(FIRST_CONSONANT_SEQS, fc, full || !vo.is_empty(), true);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    let vo_mask = if !vo.is_empty() {
        let m = lookup_mask(VOWEL_SEQS, vo, full || !lc.is_empty(), full);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    let lc_mask = if !lc.is_empty() {
        let m = lookup_mask(LAST_CONSONANT_SEQS, lc, full, true);
        if m == 0 {
            return false;
        }
        m
    } else {
        0
    };

    if vo_mask == 0 {
        return fc_mask != 0;
    }

    if fc_mask != 0 {
        let valid_cv = is_valid_cv(fc_mask, vo_mask);
        if !valid_cv || lc_mask == 0 {
            return valid_cv;
        }
    }

    if lc_mask != 0 {
        return is_valid_vc(vo_mask, lc_mask);
    }

    true
}
