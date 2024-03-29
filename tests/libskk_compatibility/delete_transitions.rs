//!
//! ueno/libskk_compatibility tests/basic.c delete_transitionsより
//!
use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn delete_transitions() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A BackSpace",
        "▽",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A BackSpace BackSpace",
        "",
        "",
        InputMode::Hiragana,
    );

    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A C-h",
        "▽",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A C-h C-h",
        "",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "E B BackSpace",
        "▽え",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "E B BackSpace r a B",
        "▽えら*b",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn delete_from_composition_selection() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i s a t s u space BackSpace",
        "",
        "挨",
        InputMode::Hiragana,
    );
}
