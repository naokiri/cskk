//!
//! ueno/libskk_compatibility tests/basic.c dict_edit_transitionsより
//!

use crate::utils::{default_test_context, transition_check};
use cskk::dictionary::static_dict::StaticFileDict;
use cskk::dictionary::user_dictionary::UserDictionary;
use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{skk_context_new_rs, skk_context_reset};

#[test]
fn register_mode_transitions_hiragana() {
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
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space a",
        "▼かぱ【あ】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a",
        "▼かぱ【▽か】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a p a space",
        "▼かぱ【▼かぱ【】】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
}

#[test]
fn register_mode_transition_abort() {
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
    skk_context_reset(&mut context);
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

// 実際の登録を実装するまでignore
#[test]
#[ignore]
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
    let static_dict =
        CskkDictionary::StaticFile(StaticFileDict::new("tests/data/SKK-JISYO.S", "euc-jp"));
    let user_dict = CskkDictionary::UserFile(UserDictionary::new("tests/data/empty.dat", "utf-8"));
    let mut context = skk_context_new_rs(vec![static_dict, user_dict]);

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a space",
        "▼かぱ【▼下】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a space H a space C-j",
        "▼かぱ【下破】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space K a space H a space C-j Return",
        "",
        "下破",
        InputMode::Hiragana,
    );
    skk_context_reset(&mut context);
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
