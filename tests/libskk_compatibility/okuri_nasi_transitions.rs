//!
//! ueno/libskk_compatibility tests/basic.c okuri_nasi_transitionsより
//!

use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn okuri_nasi_transitions_basic() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A",
        "▽あ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
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
        "A i space space",
        "▼哀",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space space Return",
        "",
        "哀",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_nasi_transitions_all_capital() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "N A",
        "▽な",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "N A N",
        "▽な*n",
        "",
        InputMode::Hiragana,
    );

    // TODO: Registration mode
    if false {
        skk_context_reset_rs(&mut context);
        transition_check(
            &mut context,
            CompositionMode::Direct,
            InputMode::Hiragana,
            "N A N A",
            "▼な*んあ【】",
            "",
            InputMode::Hiragana,
        );
        skk_context_reset_rs(&mut context);
        transition_check(
            &mut context,
            CompositionMode::Direct,
            InputMode::Hiragana,
            "N A N a",
            "▼な*な【】",
            "",
            InputMode::Hiragana,
        );
    }
}

#[test]
fn okuri_nasi_transitions_kanjis() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "I z e n space",
        "▼以前",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n j i space C-j",
        "",
        "漢字",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n j i space C-g",
        "▽かんじ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_nasi_transitions_other_inputmodes() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "K a n j i space",
        "▼漢字",
        "",
        InputMode::HankakuKatakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n j i space q",
        "",
        "漢字",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "K a n j i space q",
        "",
        "漢字",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_nasi_transitions_ignore_non_command_ctrl() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A o space Control_L",
        "▼青",
        "",
        InputMode::Hiragana,
    );
}
