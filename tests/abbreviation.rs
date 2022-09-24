extern crate cskk;

use crate::utils::{
    default_test_context, init_test_logger, test_context_with_dictionaries, transition_check,
};
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::sync::Arc;

mod utils;

#[test]
fn basic_abbreviation_mode() {
    init_test_logger();
    let dict = CskkDictionary::new_static_dict("tests/data/dictionaries/abbreviation.dat", "utf-8")
        .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash c h a l a z a",
        "▽chalaza",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash c h a l a z a space",
        "▼カラザ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash c h a l a z a space Return",
        "",
        "カラザ",
        InputMode::Hiragana,
    );
}

#[test]
fn confirm_on_abbreviation_mode() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash c h C-j",
        "",
        "ch",
        InputMode::Hiragana,
    );
}

#[test]
fn confirm_as_zenkaku_on_abbreviation_mode() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash C h C-q",
        "",
        "Ｃｈ",
        InputMode::Hiragana,
    );
}

#[test]
fn delete_on_abbreviation_mode() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "slash c h BackSpace",
        "▽c",
        "",
        InputMode::Hiragana,
    );
}
