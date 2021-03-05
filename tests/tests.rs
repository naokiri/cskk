extern crate cskk;

use cskk::{create_new_context, skk_context_set_composition_mode, skk_context_set_input_mode, skk_context_process_key_events_rs, skk_context_poll_output_rs};
use cskk::skk_modes::{CompositionMode, InputMode};


#[test]
fn basic_hiragana_input() {
    let mut context = create_new_context();
    skk_context_set_composition_mode(context.as_mut(), CompositionMode::Direct);
    skk_context_set_input_mode(context.as_mut(), InputMode::Hiragana);
    skk_context_process_key_events_rs(context.as_mut(), &"a");
    let preedit = skk_context_poll_output_rs(context.as_mut());
    assert_eq!("あ", preedit, "Direct Hiragana mode function is correct");
}