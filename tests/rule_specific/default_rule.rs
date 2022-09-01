use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn z_slash() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "z slash",
        "",
        "・",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn z_minus() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "z minus",
        "",
        "〜", // using U+301C wave dash
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn mu() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m u",
        "",
        "む",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn hankaku_mode_changing() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "C-q",
        "",
        "",
        InputMode::HankakuKatakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "C-q",
        "",
        "",
        InputMode::Katakana,
    );
}

#[test]
fn hankaku_precomposition_input() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::HankakuKatakana,
        "a q",
        "",
        "あ",
        InputMode::HankakuKatakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::HankakuKatakana,
        "a C-q",
        "",
        "ア",
        InputMode::HankakuKatakana,
    );
}
