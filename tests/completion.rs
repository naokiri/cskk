use crate::utils::{
    default_test_context, init_test_logger, test_context_with_dictionaries, transition_check,
};
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::sync::Arc;

mod utils;

#[test]
pub fn completion_mode_from_direct() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H o k k a i Tab",
        "■北海道",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H o k k a i Tab Return",
        "",
        "北海道",
        InputMode::Hiragana,
    );
}

#[test]
pub fn completion_mode_continue_typing() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H o k k a i Tab h e",
        "",
        "北海道へ",
        InputMode::Hiragana,
    );
}

#[test]
pub fn completion_mode_multiple_candidates() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "M a k u Tab Tab",
        "■幕",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "M a k u Tab Tab Tab",
        "■枕",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "M a k u Tab Tab Tab Tab period",
        "■枕",
        "",
        InputMode::Hiragana,
    );
}

#[test]
pub fn previous_completion() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "S h u u Tab Tab",
        "■修",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "S h u u Tab Tab Tab",
        "■週",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "S h u u Tab Tab Tab (shift Tab) comma",
        "■集",
        "",
        InputMode::Hiragana,
    );
}

#[test]
pub fn abort() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "S e i k a Tab Tab Escape k u",
        "▽せいかく",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "S e i k a Tab Tab C-g k u",
        "▽せいかく",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
pub fn backspace() {
    // 通常の漢字変換と同一の処理の場合、現在候補で確定してから一文字消すので統一している
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "S e i k a Tab Tab BackSpace k u",
        "",
        "正く",
        InputMode::Hiragana,
    );
}

#[test]
pub fn completion_mode_from_abbrev() {
    init_test_logger();
    let dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/abbreviation.dat", "utf-8", true)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash c h a Tab",
        "■カラザ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
pub fn completion_mode_when_no_candidate() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i u e Tab",
        "▽あいうえ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
pub fn completion_mode_disabled_dictionary() {
    init_test_logger();
    let dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/abbreviation.dat", "utf-8", false)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash c h a Tab",
        "▽cha",
        "",
        InputMode::Hiragana,
    );
}

#[test]
pub fn completion_mode_abort_to_abbreviation() {
    init_test_logger();
    let dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/abbreviation.dat", "utf-8", true)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash f a Tab C-g m Tab",
        "■家族",
        "",
        InputMode::Hiragana,
    );
}
