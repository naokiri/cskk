mod utils;

use crate::utils::{
    default_test_context, init_test_logger, make_temp_file, test_context_with_dictionaries,
    transition_check,
};
use cskk::dictionary::CskkDictionary;
use cskk::keyevent::CskkKeyEvent;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::str::FromStr;
use std::sync::Arc;

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
fn kakutei_with_ctrl_j() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i C-j",
        "",
        "あい",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    // ueno/libskkと動作が違う部分
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i n C-j",
        "",
        "あいん",
        InputMode::Hiragana,
    );
}

#[test]
fn reset_precomposition_on_ctrl_g() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i C-g A i",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn flush_kana_on_abort_no_candidate_registration() {
    init_test_logger();
    let temppath = make_temp_file().unwrap();
    let empty_dict_path = temppath.to_str().unwrap();
    let static_dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/SKK-JISYO.S", "euc-jp", false)
            .unwrap();
    let user_dict = CskkDictionary::new_user_dict(empty_dict_path, "utf-8", false).unwrap();
    let mut context =
        test_context_with_dictionaries(vec![Arc::new(static_dict), Arc::new(user_dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space C-g space",
        "▼かぱ【】",
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
        "K a p a space C-g space X",
        "",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a p a space C-g space",
        "▼かぱ【】",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn delete_pre_conversion() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K o k o r C-h r o space",
        "▼心",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn delete_on_beginning_of_register_mode() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "P e R e C-h S u",
        "▼ぺ*れ【▽す】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn backspace_to_abort() {
    init_test_logger();
    let mut context = default_test_context();
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
}

///
/// 現在surrounding_textに対応しないので、矢印キーは処理せずスルーする。
///
#[test]
fn arrow() {
    init_test_logger();
    let mut context = default_test_context();
    let processed = context.process_key_event(&CskkKeyEvent::from_str("Right").unwrap());
    assert!(!processed);
    let processed = context.process_key_event(&CskkKeyEvent::from_str("Up").unwrap());
    assert!(!processed);
    let processed = context.process_key_event(&CskkKeyEvent::from_str("Left").unwrap());
    assert!(!processed);
    let processed = context.process_key_event(&CskkKeyEvent::from_str("Down").unwrap());
    assert!(!processed);
}
