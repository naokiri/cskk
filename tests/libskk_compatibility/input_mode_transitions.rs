//!
//! ueno/libskk_compatibility tests/basic.c input_mode_transitionsより
//!

use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn input_mode_transitions_hiragana() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "q",
        "",
        "",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
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
        InputMode::Hiragana,
        "l",
        "",
        "",
        InputMode::Ascii,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "L",
        "",
        "",
        InputMode::Zenkaku,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "w w q",
        "",
        "っ",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "l Q",
        "",
        "Q",
        InputMode::Ascii,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "n q",
        "",
        "ん",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "n l",
        "",
        "ん",
        InputMode::Ascii,
    );
}

#[test]
fn input_mode_transition_katakana() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "q",
        "",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "C-q",
        "",
        "",
        InputMode::HankakuKatakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "l",
        "",
        "",
        InputMode::Ascii,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "L",
        "",
        "",
        InputMode::Zenkaku,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "n q",
        "",
        "ン",
        InputMode::Hiragana,
    );
}

#[test]
fn input_mode_transition_hankakukatakana() {
    let mut context = default_test_context();

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "q",
        "",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "l",
        "",
        "",
        InputMode::Ascii,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "L",
        "",
        "",
        InputMode::Zenkaku,
    );
}

#[ignore]
#[test]
fn input_mode_transition_hankakukatakana_orig() {
    init_test_logger();
    let mut context = default_test_context();
    // これはfailする。
    // あえてlibskkから変更している箇所で、q,C-qでのモード変更やPreCompositionでの入力確定がひらがな、カタカナ、半角カナで対称になるようにした。
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "C-q",
        "",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn input_mode_transition_ascii() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Ascii,
        "C-j",
        "",
        "",
        InputMode::Hiragana,
    );
}
