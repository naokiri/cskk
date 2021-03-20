//!
//! ueno/libskk_compatibility tests/basic.c okuri_ari_transitionsより
//!

use crate::utils::{default_test_context, transition_check};
use cskk::skk_context_reset;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn basic() {
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
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n g a E r",
        "r",
        "考え",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H a Z",
        "▽は*z",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H a Z u",
        "▼恥ず",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
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
    skk_context_reset(&mut context);
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
    skk_context_reset(&mut context);
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
fn check_small_tsu_on_okuri() {
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
    skk_context_reset(&mut context);
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
fn katakana_okuri() {
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
}

// TODO: 辞書登録実装 関連テストをignoreしないようにする
#[ignore]
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
        InputMode::Katakana,
    );
}

#[ignore]
#[test]
fn dict_edit_initial_state() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "N e o C h i space N",
        "▼ねお*ち【 ▽n】",
        "",
        InputMode::Katakana,
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
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A z u m a space >",
        "▽>",
        "東",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
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
