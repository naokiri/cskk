//!
//! ueno/libskk_compatibility tests/basic.c dict_edit_transitionsより
//!

use crate::utils::{
    default_test_context, init_test_logger, test_context_with_dictionaries, transition_check,
};
use cskk::dictionary::static_dict::StaticFileDict;
use cskk::dictionary::user_dictionary::UserDictionary;
use cskk::dictionary::{CskkDictionary, CskkDictionaryType};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::sync::Arc;

#[test]
fn register_mode_transitions_hiragana() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space",
        "▼かぱ【】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space a",
        "▼かぱ【あ】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a",
        "▼かぱ【▽か】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a p a space",
        "▼かぱ【▼かぱ【】】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn register_mode_transition_abort() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a p a space C-g",
        "▼かぱ【▽かぱ】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a p a space C-g C-g",
        "▼かぱ【】",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn register_mode_dont_register_empty() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space Return",
        "▽かぱ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn register_mode_transitions_composition() {
    let static_dict = CskkDictionary::new(CskkDictionaryType::StaticFile(StaticFileDict::new(
        "tests/data/SKK-JISYO.S",
        "euc-jp",
    )));
    let user_dict = CskkDictionary::new(CskkDictionaryType::UserFile(UserDictionary::new(
        "tests/data/empty.dat",
        "utf-8",
    )));
    let mut context =
        test_context_with_dictionaries(vec![Arc::new(static_dict), Arc::new(user_dict)]);

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a space",
        "▼かぱ【▼下】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a space H a space C-j",
        "▼かぱ【下破】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a space H a space C-j Return",
        "",
        "下破",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space",
        "▼下破",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn register_mode_purge_candidate() {
    let static_dict = CskkDictionary::new(CskkDictionaryType::StaticFile(StaticFileDict::new(
        "tests/data/SKK-JISYO.S",
        "euc-jp",
    )));
    let user_dict = CskkDictionary::new(CskkDictionaryType::UserFile(UserDictionary::new(
        "tests/data/empty.dat",
        "utf-8",
    )));
    let mut context =
        test_context_with_dictionaries(vec![Arc::new(static_dict), Arc::new(user_dict)]);

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a space H a space C-j Return",
        "",
        "下破",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space",
        "▼下破",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space X",
        "",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space",
        "▼かぱ【】",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn register_mode_okuri_ari() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n g a E space",
        "▼かんが*え【】",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn register_mode_katakana() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a t a k a n a space space K a t a k a n a q",
        "▼かたかな【カタカナ】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a t a k a n a space space K a t a k a n a q l n a",
        "▼かたかな【カタカナna】",
        "",
        InputMode::Ascii,
    );
    skk_context_reset_rs(&mut context);
    // C-m should be Return
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a t a k a n a space space K a t a k a n a q C-m",
        "",
        "カタカナ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a t a k a n a space space K a t a k a n a q l n a",
        "▼かたかな【カタカナna】",
        "",
        InputMode::Ascii,
    );
}

#[test]
fn register_mode_xtsu() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "t a k K u n space",
        "▼っくん【】",
        "た",
        InputMode::Hiragana,
    );
}

#[test]
fn register_mode_input_modes_change() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a n j i k a t a k a n a k a n j i space K a n j i space K a t a k a n a q K a n j i space C-j",
        "▼かんじかたかなかんじ【漢字カタカナ漢字】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T e s u t o t e s u t o t e s u t o t e s u t o space t e s u t o T e s u t o q q T e s u t o q T e s u t o C-q  C-q T e s u t o q",
        "▼てすとてすとてすとてすと【てすとテストてすとﾃｽﾄてすと】",
        "",
        InputMode::HankakuKatakana,
    );
}
