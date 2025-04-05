//!
//! ueno/libskk_compatibility tests/basic.c okuri_ari_transitionsより
//!

use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn basic() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n g a E",
        "▼考え",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n g a E r",
        "r",
        "考え",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H a Z",
        "▽は*z",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H a Z u",
        "▼恥ず",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T u k a T t",
        "▽つか*っt",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn check_nn_on_composition_mode_switching() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n J",
        "▽かん*j",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    // input is different from original to match the SKK-JISYO.S
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n Z i",
        "▼感じ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "F u N d a",
        "▼踏んだ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn check_small_tsu_on_okuri_ending() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "S a s S",
        "▽さっ*s",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn check_small_tsu_on_okuri_starting() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "S a S s",
        "▽さ*っs",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn katakana_okuri() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "q S i r o K u",
        "▼白ク",
        "",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "q S i r o K u Return",
        "",
        "白ク",
        InputMode::Katakana,
    );
}

#[test]
fn cancel_dict_edit() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T e t u d a I space C-g",
        "▼手伝い",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn dict_edit_initial_state() {
    init_test_logger();
    let mut context = default_test_context();
    // Space removed from original to match actual test it has to test.
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "N e o C h i N",
        "▼ねお*ち【▽n】",
        "",
        InputMode::Hiragana,
    );
}

// TODO: 接尾辞変換未実装なのでignore中
#[ignore]
#[test]
fn setsubiji() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A z u m a space",
        "▼東",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A z u m a space >",
        "▽>",
        "東",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A z u m a space > s h i space",
        "▽氏",
        "東",
        InputMode::Hiragana,
    );
}

// TODO: 接頭辞変換未実装
#[ignore]
#[test]
fn settouji() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T y o u >",
        "▽超",
        "",
        InputMode::Hiragana,
    );
}
