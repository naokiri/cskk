mod utils;

use crate::utils::{
    init_test_logger, make_temp_file, test_context_with_dictionaries, transition_check,
};
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::sync::Arc;

#[test]
fn number_composition() {
    init_test_logger();
    let static_dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/number_jisyo.dat", "UTF-8", false)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(static_dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 3 k a i space",
        "▼3回",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 3 k a i space space",
        "▼３回",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 3 4 k a i space space space",
        "▼三四回",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 3 4 k a i space space space space",
        "▼三十四回",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 1 k a i space space space space space",
        "▼初回",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 1 0 0 0 0 k a i space space space space space space",
        "▼壱万回",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn multiple_number_composition() {
    init_test_logger();
    let static_dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/number_jisyo.dat", "UTF-8", false)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(static_dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 1 2 slash 4 space",
        "▼12月4日",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn register_number() {
    init_test_logger();
    let temppath = make_temp_file().unwrap();
    let empty_dict_path = temppath.to_str().unwrap();
    let dict = CskkDictionary::new_user_dict(empty_dict_path, "utf-8", false).unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 1 k a i space numbersign 1 k a i Return",
        "",
        "１かい",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 2 k a i space",
        "▼２かい",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn numeric_type_4() {
    init_test_logger();
    let dict =
        CskkDictionary::new_user_dict("tests/data/dictionaries/number_jisyo.dat", "utf-8", false)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Q 1 n i t i space",
        "▼初日",
        "",
        InputMode::Hiragana,
    );
}
