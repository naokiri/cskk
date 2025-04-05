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
        "n C-q",
        "",
        "ん",
        InputMode::HankakuKatakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "n C-q",
        "",
        "ﾝ",
        InputMode::Katakana,
    );
}

#[test]
fn hiragana_precomposition_input() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::Hiragana,
        "a q",
        "",
        "ア",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::Hiragana,
        "a C-q",
        "",
        "ｱ",
        InputMode::Hiragana,
    );
}

#[test]
fn katakana_precomposition_input() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::Katakana,
        "a q",
        "",
        "あ",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::Katakana,
        "a C-q",
        "",
        "ｱ",
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

#[test]
fn ambiguous_kana() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "n",
        "n",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "n a",
        "",
        "な",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "n d a",
        "",
        "んだ",
        InputMode::Hiragana,
    );
}
