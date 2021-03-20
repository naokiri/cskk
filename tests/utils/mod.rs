use cskk::dictionary::static_dict::StaticFileDict;
use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{
    skk_context_get_input_mode, skk_context_get_preedit_rs, skk_context_new_rs,
    skk_context_poll_output_rs, skk_context_process_key_events_rs,
    skk_context_set_composition_mode, skk_context_set_input_mode, CskkContext,
};

pub fn transition_check(
    context: &mut CskkContext,
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
    assert_eq!(
        output, expected_output,
        "(output == expected) failed for '{}'",
        key_inputs
    );
    assert_eq!(
        preedit, expected_preedit,
        "(preedit == expected) failed for '{}'",
        key_inputs
    );
    assert_eq!(
        input_mode, expected_input_mode,
        "(input_mode == expected) failed for '{}'",
        key_inputs
    );
}

pub fn default_test_context() -> CskkContext {
    let dict = CskkDictionary::StaticFile(StaticFileDict::new("tests/data/SKK-JISYO.S", "euc-jp"));
    skk_context_new_rs(vec![dict])
}
