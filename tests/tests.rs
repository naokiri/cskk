extern crate cskk;

use cskk::{create_new_context, skk_context_set_composition_mode, skk_context_set_input_mode, skk_context_process_key_events_rs, skk_context_poll_output_rs, CskkContext, skk_context_get_preedit_rs, skk_context_get_input_mode, skk_context_reset};
use cskk::skk_modes::{CompositionMode, InputMode};


fn transition_check(context: &mut CskkContext,
                    initial_composition_mode: CompositionMode,
                    initial_input_mode: InputMode,
                    key_inputs: &str,
                    expected_preedit: &str,
                    expected_output: &str,
                    expected_input_mode: InputMode,
) {
    skk_context_set_composition_mode(context, initial_composition_mode);
    skk_context_set_input_mode(context, initial_input_mode);
    skk_context_process_key_events_rs(context, key_inputs);
    let output = skk_context_poll_output_rs(context);
    let preedit = skk_context_get_preedit_rs(context);
    let input_mode = skk_context_get_input_mode(context);
    assert_eq!(output, expected_output, "(output == expected) failed");
    assert_eq!(preedit, expected_preedit, "(preedit == expected) failed");
    assert_eq!(input_mode, expected_input_mode, "(input_mode == expected) failed");
}

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

#[test]
fn input_mode_transitions_hiragana() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "q",
                     "",
                     "",
                     InputMode::Katakana);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "C-q",
                     "",
                     "",
                     InputMode::HankakuKatakana);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "l",
                     "",
                     "",
                     InputMode::Ascii);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "L",
                     "",
                     "",
                     InputMode::Zenkaku);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "w w q",
                     "",
                     "っ",
                     InputMode::Katakana);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "l Q",
                     "",
                     "Q",
                     InputMode::Ascii);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Hiragana,
                     "n q",
                     "",
                     "ん",
                     InputMode::Katakana);
}

#[test]
fn input_mode_transition_katakana() {
    let mut context = create_new_context();

    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "q",
                     "",
                     "",
                     InputMode::Hiragana);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "C-q",
                     "",
                     "",
                     InputMode::HankakuKatakana);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "l",
                     "",
                     "",
                     InputMode::Ascii);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "L",
                     "",
                     "",
                     InputMode::Zenkaku);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Katakana,
                     "n q",
                     "",
                     "ン",
                     InputMode::Hiragana);
}

#[test]
fn input_mode_transition_hankakukatakana() {
    let mut context = create_new_context();

    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::HankakuKatakana,
                     "q",
                     "",
                     "",
                     InputMode::Hiragana);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::HankakuKatakana,
                     "C-q",
                     "",
                     "",
                     InputMode::Hiragana);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::HankakuKatakana,
                     "l",
                     "",
                     "",
                     InputMode::Ascii);
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::HankakuKatakana,
                     "L",
                     "",
                     "",
                     InputMode::Zenkaku);
}

#[test]
fn input_mode_transition_ascii() {
    let mut context = create_new_context();
    transition_check(context.as_mut(),
                     CompositionMode::Direct,
                     InputMode::Ascii,
                     "C-j",
                     "",
                     "",
                     InputMode::Hiragana);
}

// ueno/libskk tests/basic.c rom_kana_transitionsより
mod rom_kata_transitions {
    use super::*;

    #[test]
    fn rom_kana_transitions_basic() {
        let mut context = create_new_context();
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "k",
                         "k",
                         "",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "k a",
                         "",
                         "か",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "m",
                         "m",
                         "",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "m y",
                         "my",
                         "",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "m y o",
                         "",
                         "みょ",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "q",
                         "",
                         "",
                         InputMode::Katakana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Katakana,
                         "k",
                         "k",
                         "",
                         InputMode::Katakana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Katakana,
                         "k a",
                         "",
                         "カ",
                         InputMode::Katakana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Katakana,
                         "n .",
                         "",
                         "ン。",
                         InputMode::Katakana);
        skk_context_reset(context.as_mut());
    }

    #[test]
    fn rom_kana_transitions_include_command_letter() {
        let mut context = create_new_context();
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "z l",
                         "",
                         "→",
                         InputMode::Hiragana);
    }

    #[test]
    fn rom_kana_transitions_abort() {
        let mut context = create_new_context();
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "m y C-g",
                         "",
                         "",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "m y a C-g",
                         "",
                         "みゃ",
                         InputMode::Hiragana);
    }

    #[test]
    fn rom_kana_transitions_kana_form_change_without_input_mode() {
        let mut context = create_new_context();
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "A i q",
                         "",
                         "アイ",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Katakana,
                         "A i q",
                         "",
                         "あい",
                         InputMode::Katakana);
    }

    #[test]
    fn rom_kana_conversion_longer_conversion() {
        let mut context = create_new_context();
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "V u",
                         "▽う゛",
                         "",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "V u q",
                         "",
                         "ヴ",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Katakana,
                         "V u",
                         "▽ヴ",
                         "",
                         InputMode::Katakana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Katakana,
                         "V u",
                         "▽ヴ",
                         "",
                         InputMode::Katakana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Katakana,
                         "V u q",
                         "",
                         "う゛",
                         InputMode::Katakana);
    }

    #[test]
    fn rom_kana_transitions_with_inputs() {
        let mut context = create_new_context();
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "Q n q",
                         "",
                         "ン",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "Q Q",
                         "▽",
                         "",
                         InputMode::Hiragana);
        skk_context_reset(context.as_mut());
        transition_check(context.as_mut(),
                         CompositionMode::Direct,
                         InputMode::Hiragana,
                         "N o b a - s u C-q",
                         "",
                         "ﾉﾊﾞｰｽ",
                         InputMode::Hiragana);
    }
}