extern crate cskk;

use cskk::{create_new_context, skk_context_set_composition_mode, skk_context_set_input_mode, skk_context_process_key_events_rs, skk_context_poll_output_rs, CskkContext, skk_context_get_preedit, skk_context_get_preedit_rs, skk_context_get_input_mode, skk_context_get_compositon_mode};
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