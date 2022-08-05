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