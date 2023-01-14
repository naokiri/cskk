use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{
    skk_context_get_input_mode_rs, skk_context_get_preedit_rs, skk_context_poll_output_rs,
    skk_context_process_key_events_rs, skk_context_set_composition_mode,
    skk_context_set_input_mode_rs, CskkContext,
};
use std::sync::Arc;
use tempfile::{NamedTempFile, TempPath};

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
    skk_context_process_key_events_rs(context, key_inputs);
    let output = skk_context_poll_output_rs(context);
    let preedit = skk_context_get_preedit_rs(context);
    let input_mode = skk_context_get_input_mode_rs(context);
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
