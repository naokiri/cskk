mod utils;

use crate::utils::{default_test_context, test_context_with_dictionaries, transition_check};
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::sync::Arc;

#[test]
fn okuri_nashi_henkan() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space",
        "▼愛",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_nashi_henkan_kakutei() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space Return",
        "",
        "愛",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_nashi_henkan_continuous() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space A i space C-j",
        "",
        "愛愛",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_ari_henkan_kakutei() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A K i space Return",
        "",
        "開き",
        InputMode::Hiragana,
    );
}

#[test]
fn empty_dict_context() {
    let mut context = test_context_with_dictionaries(vec![]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space",
        "▼あい【】",
        "",
        InputMode::Hiragana,
    )
}

#[test]
fn strict_okuri_entry() {
    use crate::utils::init_test_logger;
    init_test_logger();
    let dictpath = "tests/data/dictionaries/strict_okuri.dat";
    let user_dict = CskkDictionary::new_user_dict(dictpath, "utf-8", false).unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(user_dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H o S e",
        "▼干せ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "O k u T t e",
        "▼贈って",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn strict_okuri_precedence() {
    use crate::utils::init_test_logger;
    init_test_logger();
    let dictpath = "tests/data/dictionaries/strict_okuri_precedence.dat";
    let user_dict = CskkDictionary::new_user_dict(dictpath, "utf-8", false).unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(user_dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "O o K i",
        "▼大き",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "O o K u",
        "▼多く",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "O o K i space Return",
        "",
        "多き",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "O o K i ",
        "▼多き",
        "",
        InputMode::Hiragana,
    );
}
