//!
//! ueno/libskk_compatibility tests/basic.c numeric_transitionsより
//!

use crate::utils::{init_test_logger, test_context_with_dictionaries, transition_check};
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::sync::Arc;

#[test]
fn numeric_transitions() {
    init_test_logger();
    let static_dict =
        CskkDictionary::new_user_dict("tests/data/dictionaries/number_jisyo.dat", "UTF-8").unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(static_dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 5 slash 1 space",
        "▼5月1日",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 5 h i k i space",
        "▼５匹",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 5 h i k i space space",
        "▼五匹",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 5 h i k i space space C-j",
        "",
        "五匹",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 1 h i k i space",
        "▼一匹",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 5 0 0 0 0 h i k i space",
        "▼五万匹",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 1 0 h i k i space",
        "▼十匹",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 1 1 1 1 1 h i k i space",
        "▼一万千百十一匹",
        "",
        InputMode::Hiragana,
    );
}
