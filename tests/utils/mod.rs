use cskk::cskkstate::CskkStateInfo;
use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{
    skk_context_get_input_mode_rs, skk_context_set_composition_mode, skk_context_set_input_mode_rs,
    CskkContext,
};
use std::sync::Arc;
use tempfile::{NamedTempFile, TempPath};

// Test function to directly use the test cases from libskk.
// TODO: libskk互換のフォーマッタに依存しない[transition_test]に漸次おきかえる
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
    skk_context_set_input_mode_rs(context, initial_input_mode);
    context.process_key_events_string(key_inputs);
    let output = context.poll_output().unwrap_or_default();
    let preedit = context.get_preedit().unwrap();
    let input_mode = context.get_current_input_mode();
    assert_eq!(
        output, expected_output,
        "(output == expected) failed for '{key_inputs}' starting from '{initial_composition_mode:?},{initial_input_mode:?}'"
    );
    assert_eq!(
        preedit, expected_preedit,
        "(preedit == expected) failed for '{key_inputs}' starting from '{initial_composition_mode:?},{initial_input_mode:?}'"
    );
    assert_eq!(
        input_mode, expected_input_mode,
        "(input_mode == expected) failed for '{key_inputs}' starting from '{initial_composition_mode:?},{initial_input_mode:?}'"
    );
}

pub fn transition_test(
    context: &mut CskkContext,
    initial_composition_mode: CompositionMode,
    initial_input_mode: InputMode,
    key_inputs: &str,
    expected_composition_mode: CompositionMode,
    expected_input_mode: InputMode,
    expected_state: CskkStateInfo,
) {
    skk_context_set_composition_mode(context, initial_composition_mode);
    skk_context_set_input_mode_rs(context, initial_input_mode);
    context.process_key_events_string(key_inputs);
    let preedit_detail = context.get_preedit_detail();
    let actual_state = preedit_detail.first().unwrap();
    let actual_composition_mode = context.get_current_composition_mode();
    let actual_input_mode = context.get_current_input_mode();

    assert_eq!(
        actual_composition_mode, expected_composition_mode,
        "(composition_mode == expected) failed for '{key_inputs}' starting from '{initial_composition_mode:?},{initial_input_mode:?}'"
    );
    assert_eq!(
        actual_input_mode, expected_input_mode,
        "(input_mode == expected) failed for '{key_inputs}' starting from '{initial_composition_mode:?},{initial_input_mode:?}'"
    );
    assert_eq!(
        *actual_state, expected_state,
        "(state == expected) failed for '{key_inputs}' starting from '{initial_composition_mode:?},{initial_input_mode:?}'"
    );
}

/// Make a temporary empty dict file, returns filepath.
pub fn make_temp_file() -> anyhow::Result<TempPath> {
    let tempfile = NamedTempFile::new()?;

    Ok(tempfile.into_temp_path())
}

pub fn default_test_context() -> CskkContext {
    let dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/SKK-JISYO.S", "euc-jp", true)
            .unwrap();
    test_context_with_dictionaries(vec![Arc::new(dict)])
}

pub fn test_context_with_dictionaries(dictionaries: Vec<Arc<CskkDictionary>>) -> CskkContext {
    CskkContext::new_from_specified_paths(
        InputMode::Hiragana,
        CompositionMode::Direct,
        dictionaries,
        "assets/rule/kana_form.toml",
        "assets/rule/ascii_form.toml",
        "assets/rules",
    )
}

pub fn init_test_logger() {
    let _ = env_logger::builder()
        // Include all events in tests
        .filter_level(log::LevelFilter::max())
        // Ensure events are captured by `cargo test`
        .is_test(true)
        // Ignore errors initializing the logger if tests race to configure it
        .try_init();
}
