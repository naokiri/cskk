extern crate cskk;

mod utils;

use cskk::{create_new_context};
use cskk::skk_modes::{CompositionMode, InputMode};
use crate::utils::transition_check;

#[test]
fn basic_hiragana_input() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "a i u e o",
                     "",
                     "あいうえお",
                     InputMode::Hiragana);
}

#[test]
fn simple_composition() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "A i",
                     "▽あい",
                     "",
                     InputMode::Hiragana);
}

/// 参照時のueno/skklib と ddskkで動作が違う例。
/// ddskk式の方が現実的な打ち間違え時になってほしい状態なので採用
///
/// ueno/skklib
/// "k A" -> ▽あ
/// ddskk
/// "k A" -> ▽か
#[test]
fn composition_mode_from_middle() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "k I",
                     "▽き",
                     "",
                     InputMode::Hiragana);
}


#[test]
fn okuri_nashi_henkan() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "A i space",
                     "▼愛",
                     "",
                     InputMode::Hiragana);
}

#[test]
fn skip_on_impossible_kana() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "b n y a",
                     "",
                     "にゃ",
                     InputMode::Hiragana);
}

#[test]
fn okuri_ari_henkan_precomposition() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "A K",
                     "▽あ*k",
                     "",
                     InputMode::Hiragana);
}

#[test]
fn okuri_ari_henkan_to_composition_selection() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "A K i",
                     "▼開き",
                     "",
                     InputMode::Hiragana);
}

#[test]
fn okuri_nashi_henkan_kakutei() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "A i space Return",
                     "",
                     "愛",
                     InputMode::Hiragana);
}

#[test]
fn okuri_ari_henkan_kakutei() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "A K i space Return",
                     "",
                     "飽き",
                     InputMode::Hiragana);
}

#[test]
fn katakana_input() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "q x t u",
                     "",
                     "ッ",
                     InputMode::Katakana);
}