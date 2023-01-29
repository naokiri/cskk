//!
//! ueno/libskk_compatibility tests/basic.c abort_transitionsより
//!
use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn abort_transitions() {
    init_test_logger();
    let mut context = default_test_context();
    // Back to selection
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A k a space space space C-g",
        "▼垢",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "O K i C-g",
        "▽おき",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "O K C-g",
        "",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A o i O C-g",
        "▽あおいお",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn abort_transitions_from_register_direct() {
    init_test_logger();
    let mut context = default_test_context();
    // Back to preedit
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A p a space C-g",
        "▽あぱ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn abort_multiple_from_composition() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space C-g C-g",
        "",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space C-g C-g C-g",
        "",
        "",
        InputMode::Hiragana,
    );
}

/// (Direct ->) Composition -> Selection -> Register から二度戻る場合
#[test]
fn abort_multiple_register_back_to_composition() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H o k a space space space C-g C-g",
        "▽ほか",
        "",
        InputMode::Hiragana,
    );
}
/// Direct -> Composition -> Selection -> Register から三度戻る場合
#[test]
fn abort_multiple_register_back_to_direct() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H o k a space space space C-g C-g C-g",
        "",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn abort_candidate_selection() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space C-g",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn abort_non_register_direct_mode() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "C-g",
        "",
        "",
        InputMode::Hiragana,
    );
}
