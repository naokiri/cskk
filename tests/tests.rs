extern crate cskk;

use cskk::{create_new_context,
           skk_context_set_composition_mode,
           skk_context_set_input_mode,
           skk_context_process_key_events_rs,
           skk_context_poll_output_rs,
           CskkContext,
           skk_context_get_preedit_rs,
           skk_context_get_input_mode};
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
    assert_eq!(output, expected_output);
    assert_eq!(preedit, expected_preedit);
    assert_eq!(input_mode, expected_input_mode);
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