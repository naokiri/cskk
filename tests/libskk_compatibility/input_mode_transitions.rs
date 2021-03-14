//!
//! ueno/libskk_compatibility tests/basic.c input_mode_transitionsより
//!

use cskk::{skk_context_reset};
use cskk::skk_modes::{CompositionMode, InputMode};
use crate::utils::{transition_check,new_test_context};

#[test]
fn input_mode_transitions_hiragana() {
    let mut context = new_test_context();
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "q",
                     "",
                     "",
                     InputMode::Katakana);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "C-q",
                     "",
                     "",
                     InputMode::HankakuKatakana);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "l",
                     "",
                     "",
                     InputMode::Ascii);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "L",
                     "",
                     "",
                     InputMode::Zenkaku);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "w w q",
                     "",
                     "っ",
                     InputMode::Katakana);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "l Q",
                     "",
                     "Q",
                     InputMode::Ascii);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "n q",
                     "",
                     "ん",
                     InputMode::Katakana);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "n l",
                     "",
                     "ん",
                     InputMode::Ascii);
}

#[test]
fn input_mode_transition_katakana() {
    let mut context = new_test_context();

    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "q",
                     "",
                     "",
                     InputMode::Hiragana);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "C-q",
                     "",
                     "",
                     InputMode::HankakuKatakana);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "l",
                     "",
                     "",
                     InputMode::Ascii);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "L",
                     "",
                     "",
                     InputMode::Zenkaku);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "n q",
                     "",
                     "ン",
                     InputMode::Hiragana);
}

#[test]
fn input_mode_transition_hankakukatakana() {
    let mut context = new_test_context();

    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::HankakuKatakana,
                     "q",
                     "",
                     "",
                     InputMode::Hiragana);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::HankakuKatakana,
                     "C-q",
                     "",
                     "",
                     InputMode::Hiragana);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::HankakuKatakana,
                     "l",
                     "",
                     "",
                     InputMode::Ascii);
    skk_context_reset(&mut context);
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::HankakuKatakana,
                     "L",
                     "",
                     "",
                     InputMode::Zenkaku);
}

#[test]
fn input_mode_transition_ascii() {
    let mut context = new_test_context();
    transition_check(&mut context,
                     CompositionMode::Direct,
                     InputMode::Ascii,
                     "C-j",
                     "",
                     "",
                     InputMode::Hiragana);
}