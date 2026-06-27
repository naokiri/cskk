mod utils;

use crate::utils::init_test_logger;
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{skk_context_set_composition_mode, skk_context_set_input_mode_rs, CskkContext};
use std::sync::Arc;

fn test_context_with_data_rules() -> CskkContext {
    let dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/SKK-JISYO.S", "euc-jp", true)
            .unwrap();
    CskkContext::new_from_specified_paths(
        InputMode::Hiragana,
        CompositionMode::Direct,
        vec![Arc::new(dict)],
        "../assets/rule/kana_form.toml",
        "../assets/rule/ascii_form.toml",
        "tests/data/rules",
    )
}

fn check(
    context: &mut CskkContext,
    key_inputs: &str,
    expected_preedit: &str,
    expected_output: &str,
) {
    skk_context_set_composition_mode(context, CompositionMode::Direct);
    skk_context_set_input_mode_rs(context, InputMode::Hiragana);
    context.process_key_events_string(key_inputs);
    let output = context.poll_output().unwrap_or_default();
    let preedit = context.get_preedit().unwrap();
    assert_eq!(
        preedit, expected_preedit,
        "preedit mismatch for '{key_inputs}'"
    );
    assert_eq!(
        output, expected_output,
        "output mismatch for '{key_inputs}'"
    );
}

/// (shift quotedbl) → triggers PreComposition and inserts っ as the kana.
/// This demonstrates the use case of inputting kana with a non-letter key.
#[test]
fn shift_quotedbl_enters_precomposition() {
    init_test_logger();
    let mut ctx = test_context_with_data_rules();
    check(&mut ctx, "(shift quotedbl)", "▽っ", "");
    skk_context_reset_rs(&mut ctx);
}

/// Bare quotedbl (keysym quotedbl, no explicit shift) also triggers PreComposition
/// because the trigger check is keysym-based: quotedbl == quotedbl regardless of how
/// the OS produced it (dedicated key or Shift+').
#[test]
fn bare_quotedbl_also_triggers_precomposition() {
    init_test_logger();
    let mut ctx = test_context_with_data_rules();
    check(&mut ctx, "quotedbl", "▽っ", "");
    skk_context_reset_rs(&mut ctx);
}

/// (shift at) is NOT in the trigger list → outputs @ raw without entering PreComposition.
#[test]
fn non_trigger_shift_symbol_outputs_raw() {
    init_test_logger();
    let mut ctx = test_context_with_data_rules();
    check(&mut ctx, "(shift at)", "", "@");
    skk_context_reset_rs(&mut ctx);
}

/// Regression: capital A still enters PreComposition with default trigger set.
#[test]
fn capital_a_still_triggers_precomposition() {
    init_test_logger();
    let mut ctx = test_context_with_data_rules();
    check(&mut ctx, "A", "▽あ", "");
    skk_context_reset_rs(&mut ctx);
}

/// Regression: tsU (uppercase mid-sequence rule) still converts without mode change.
#[test]
#[allow(non_snake_case)]
fn tsU_sequence_converts_without_mode_change() {
    init_test_logger();
    let mut ctx = test_context_with_data_rules();
    check(&mut ctx, "t s U", "", "ちいさいっ");
    skk_context_reset_rs(&mut ctx);
}

/// A followed by (shift quotedbl) — quotedbl acts as an okurigana trigger, advancing past
/// PreComposition. Depending on the dictionary the engine may enter CompositionSelection
/// (dict hit) or Register mode (no entry); either proves the okurigana trigger fired.
#[test]
fn precomposition_then_quotedbl_trigger_starts_okurigana() {
    init_test_logger();
    let mut ctx = test_context_with_data_rules();
    skk_context_set_composition_mode(&mut ctx, CompositionMode::Direct);
    skk_context_set_input_mode_rs(&mut ctx, InputMode::Hiragana);
    ctx.process_key_events_string("A (shift quotedbl)");
    let preedit = ctx.get_preedit().unwrap();
    // Okurigana trigger fired: preedit is either 【…】 (Register) or ▼… (CompositionSelection).
    // In both cases we have left plain PreComposition (▽あ).
    assert!(
        preedit.contains('【') || !preedit.starts_with('▽'),
        "expected okurigana processing after 'A (shift quotedbl)', got: {}",
        preedit
    );
    skk_context_reset_rs(&mut ctx);
}
