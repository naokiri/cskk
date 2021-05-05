extern crate cskk;

mod utils;

use crate::utils::{
    default_test_context, init_test_logger, test_context_with_dictionaries, transition_check,
};
use cskk::dictionary::static_dict::StaticFileDict;
use cskk::dictionary::user_dictionary::UserDictionary;
use cskk::dictionary::{CskkDictionary, CskkDictionaryType};
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{skk_context_reload_dictionary, skk_context_reset_rs, skk_context_save_dictionaries_rs};
use std::sync::Arc;

#[test]
fn basic_hiragana_input() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "a i u e o",
        "",
        "あいうえお",
        InputMode::Hiragana,
    );
}

#[test]
fn simple_composition() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
}

/// 参照時のueno/skklib と ddskkで動作が違う例。
/// ddskk式の方が現実的な打ち間違え時になってほしい状態なので採用
///
/// ueno/skklib
/// "k A" -> ▽あ
/// ddskk
/// "k A" -> ▽か
#[test]
fn composition_mode_from_middle() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "k I",
        "▽き",
        "",
        InputMode::Hiragana,
    );
}

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
fn skip_on_impossible_kana() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "b n y a",
        "",
        "にゃ",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_ari_henkan_precomposition() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A K",
        "▽あ*k",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn okuri_ari_henkan_to_composition_selection() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A K i",
        "▼飽き",
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
fn katakana_input() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "q x t u",
        "",
        "ッ",
        InputMode::Katakana,
    );
}

#[test]
fn save_dict() {
    let dict = CskkDictionary::new(CskkDictionaryType::UserFile(UserDictionary::new(
        "tests/data/userdict.dat",
        "utf-8",
    )));
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    skk_context_save_dictionaries_rs(&mut context);
    skk_context_reload_dictionary(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Z o u g o space Return",
        "",
        "象語",
        InputMode::Hiragana,
    )
}

#[test]
fn empty_dict() {
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
fn register_and_read() {
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
        "H o g e space A i space Return Return",
        "",
        "愛",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H o g e space",
        "▼愛",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn non_ascii_transition() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Ascii,
        "plus",
        "",
        "+",
        InputMode::Ascii,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "plus",
        "",
        "+",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K plus",
        "▽+",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    // 現在のueno/libskk と差異がある部分？
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a K plus",
        "▽か*+",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn capital_q_transition() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A a Q",
        "▽ああ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn g_without_ctrl() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "g",
        "g",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn backspace_direct_kanabuild() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "h y BackSpace",
        "h",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn backspace_precomposition() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i BackSpace",
        "▽あ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn previous_candidate() {
    init_test_logger();
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
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space space x",
        "▼愛",
        "",
        InputMode::Hiragana,
    );
}
