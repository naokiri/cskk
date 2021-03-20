//!
//! ueno/libskk_compatibility tests/basic.c rom_kana_transitionsより
//!

use crate::utils::{default_test_context, transition_check};
use cskk::skk_context_reset;
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
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "k a",
        "",
        "か",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m",
        "m",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m y",
        "my",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m y o",
        "",
        "みょ",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "q",
        "",
        "",
        InputMode::Katakana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "k",
        "k",
        "",
        InputMode::Katakana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "k a",
        "",
        "カ",
        InputMode::Katakana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "n .",
        "",
        "ン。",
        InputMode::Katakana,
    );
    skk_context_reset(&mut context);
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
    skk_context_reset(&mut context);
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
    skk_context_reset(&mut context);
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

#[test]
fn rom_kana_conversion_longer_conversion() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "V u",
        "▽う゛",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "V u q",
        "",
        "ヴ",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u",
        "▽ヴ",
        "",
        InputMode::Katakana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u",
        "▽ヴ",
        "",
        InputMode::Katakana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u q",
        "",
        "う゛",
        InputMode::Katakana,
    );
}

#[test]
fn rom_kana_transitions_with_inputs() {
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
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q Q",
        "▽",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "N o b a - s u C-q",
        "",
        "ﾉﾊﾞｰｽ",
        InputMode::Hiragana,
    );

    // TODO: Add other additional test cases from issues
}
