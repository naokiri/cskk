//!
//! ueno/libskk_compatibility tests/basic.c rom_kana_transitionsより
//!

use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn rom_kana_transitions_basic() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "k",
        "k",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "k a",
        "",
        "か",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m",
        "m",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m y",
        "my",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m y o",
        "",
        "みょ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
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
        InputMode::Katakana,
        "k",
        "k",
        "",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "k a",
        "",
        "カ",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "n .",
        "",
        "ン。",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn rom_kana_transitions_include_command_letter() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "z l",
        "",
        "→",
        InputMode::Hiragana,
    );
}

#[test]
fn rom_kana_transitions_abort() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m y C-g",
        "",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m y a C-g",
        "",
        "みゃ",
        InputMode::Hiragana,
    );
}

#[test]
fn rom_kana_transitions_kana_form_change_without_input_mode() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i q",
        "",
        "アイ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "A i q",
        "",
        "あい",
        InputMode::Katakana,
    );
}

/// Changed from libskk.
#[test]
fn rom_kana_conversion_vu_conversion() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "V u",
        "▽ゔ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "V u q",
        "",
        "ヴ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u",
        "▽ヴ",
        "",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u",
        "▽ヴ",
        "",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u q",
        "",
        "ゔ",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "v u",
        "",
        "ｳﾞ",
        InputMode::HankakuKatakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "V u q",
        "",
        "ゔ",
        InputMode::HankakuKatakana,
    );
}

#[test]
fn rom_kana_transitions_with_inputs() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q n q",
        "",
        "ン",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q Q",
        "▽",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "N o b a - s u C-q",
        "",
        "ﾉﾊﾞｰｽ",
        InputMode::Hiragana,
    );
}

#[test]
fn n_to_nn() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "n space",
        "",
        "ん ",
        InputMode::Hiragana,
    );
}

#[test]
fn w_as_kana() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "W o",
        "▽を",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn pass_through_tabs() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Tab K a",
        "▽か",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Ascii,
        "a Tab",
        "",
        "a",
        InputMode::Ascii,
    );
}
#[test]
fn n_followed_by_capital() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "q s a n S y a",
        "▽シャ",
        "サン",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn pass_through_ctrl_only() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H o h Control_L a a a a a",
        "▽ほはああああ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}
