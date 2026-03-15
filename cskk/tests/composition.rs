mod utils;

use crate::utils::{default_test_context, init_test_logger, transition_check, transition_test};
use cskk::cskkstate::{CskkStateInfo, DirectData};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

///
/// ddskk -> きん*う
/// ueno/libskk -> ▽きぬ
///
/// どちらとも違うが上記precomposition_mode_from_middleと整合性を取り ▽き*ぬ
///
#[test]
fn okuri_mode_from_middle() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "k I n U",
        "▼き*ぬ【】",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_ari_henkan_precomposition() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A K",
        "▽あ*k",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_ari_henkan_to_composition_selection() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A K i",
        "▼飽き",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn previous_candidate() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space",
        "▼愛",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space space x",
        "▼愛",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space space x x",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn previous_candidate_with_okuri() {
    init_test_logger();
    let mut context = default_test_context();
    // This differs from libskk on purpose.
    // libskk keeps okuri, but inputting letter after previous candidate seems broken as of libskk v1.0.5
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H a Z u x",
        "▽はず",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn escape_from_candidate_selection() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space space Escape",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn escape_from_okuri_ari_candidate_selection() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "A I C-g space",
        "▼愛",
        "",
        InputMode::Katakana,
    );
}

#[test]
fn confirm_composition_on_non_kana() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "A i space 1",
        "",
        "愛1",
        InputMode::Katakana,
    );
}

#[test]
fn escape_from_okuri_ari_register() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "M a g u R o C-g space",
        "▼鮪",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn abort_precomposite_mode() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i C-g A i space",
        "▼愛",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A K C-g A K i space",
        "▼開き",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn abort_composition_by_previous_candidate() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space x",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    // Not あ*き。consolidate。
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A K i x",
        "▽あき",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn confirmed_in_register() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "W a w a w a space A i space U e space",
        "▼わわわ【愛▼上】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "W a w a w a space A i space U e Tab",
        "▼わわわ【愛■上】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "W a w a w a space A i space U",
        "▼わわわ【愛▽う】",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn abort() {
    init_test_logger();
    let mut context = default_test_context();
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A space Return A C-g C-g",
        CompositionMode::Direct,
        InputMode::Hiragana,
        CskkStateInfo::Direct(DirectData {
            confirmed: "阿".to_string(),
            unconverted: None,
        }),
    );
}

#[test]
fn basic_okuri_ari() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T u k u R i",
        "▼作り",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_more_than_2_hiragana() {
    init_test_logger();
    use crate::utils::test_context_with_dictionaries;
    use cskk::dictionary::CskkDictionary;
    use std::sync::Arc;

    let dict = CskkDictionary::new_static_dict(
        "tests/data/dictionaries/2letter_okuri.dat",
        "utf-8",
        false,
    )
    .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "I T t e",
        "▼言って",
        "",
        InputMode::Hiragana,
    );
}

// Issue #302
#[test]
fn ctrl_g_after_previous_candidate_resets_to_direct() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A space x x C-g",
        "",
        "",
        InputMode::Hiragana,
    );
}

// Issue #302
#[test]
fn ctrl_g_after_select_candidate_at_negative_resets_to_direct() {
    use crate::utils::test_context_with_dictionaries;
    use cskk::dictionary::CskkDictionary;
    use cskk::{
        skk_context_select_candidate_at_rs, skk_context_set_composition_mode,
        skk_context_set_input_mode_rs,
    };
    use std::sync::Arc;
    init_test_logger();
    let dict = CskkDictionary::new_static_dict(
        "tests/data/dictionaries/select_candidate_at.dat",
        "utf-8",
        false,
    )
    .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    skk_context_set_composition_mode(&mut context, CompositionMode::Direct);
    skk_context_set_input_mode_rs(&mut context, InputMode::Hiragana);
    context.process_key_events_string("A i space");
    skk_context_select_candidate_at_rs(&mut context, -1);
    context.process_key_events_string("C-g");
    let output = context.poll_output().unwrap_or_default();
    let preedit = context.get_preedit().unwrap();
    assert_eq!(preedit, "", "preedit should be empty after C-g");
    assert_eq!(output, "", "output should be empty after C-g");
    assert_eq!(
        context.get_current_composition_mode(),
        CompositionMode::Direct,
        "composition mode should be Direct after C-g"
    );
}

// Issue #257
#[test]
fn composition_select_with_command() {
    init_test_logger();
    let mut context = default_test_context();
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H e n k a n space l",
        CompositionMode::Direct,
        InputMode::Ascii,
        CskkStateInfo::Direct(DirectData {
            confirmed: "変換".to_string(),
            unconverted: None,
        }),
    );
}
