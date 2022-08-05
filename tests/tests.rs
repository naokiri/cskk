extern crate cskk;

mod utils;

use crate::utils::{
    default_test_context, init_test_logger, test_context_with_dictionaries, transition_check,
};
use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{
    skk_context_reload_dictionary, skk_context_reset_rs, skk_context_save_dictionaries_rs,
    skk_context_set_auto_start_henkan_keywords_rs,
};
use std::sync::Arc;

// TODO: Split into several files.

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
    let dict = CskkDictionary::new_user_dict("tests/data/userdict.dat", "utf-8").unwrap();
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
    let static_dict = CskkDictionary::new_static_dict("tests/data/SKK-JISYO.S", "euc-jp").unwrap();
    let user_dict = CskkDictionary::new_user_dict("tests/data/empty.dat", "utf-8").unwrap();
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
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space space x x",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
}


#[test]
fn previous_candidate_with_okuri() {
    init_test_logger();
    let mut context = default_test_context();
    // This differs from libskk on purpose.
    // libskk keeps okuri, but inputting letter after previous candidate seems broken as of libskk v1.0.5
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "H a Z u x",
        "▽はず",
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
    let static_dict = CskkDictionary::new_static_dict("tests/data/SKK-JISYO.S", "euc-jp").unwrap();
    let user_dict = CskkDictionary::new_user_dict("tests/data/empty.dat", "utf-8").unwrap();
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
fn escape_from_candidate_selection() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i space space Escape",
        "▽あい",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn escape_from_okuri_ari_candidate_selection() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "A I C-g space",
        "▼愛",
        "",
        InputMode::Katakana,
    );
}

#[test]
fn confirm_composition_on_non_kana() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "A i space 1",
        "",
        "愛1",
        InputMode::Katakana,
    );
}

#[test]
fn auto_start_henkan() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i period",
        "▼愛。",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i comma",
        "▼愛、",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i bracketright",
        "▼愛」",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn set_auto_start_henkan() {
    init_test_logger();
    let mut context = default_test_context();

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i w o",
        "▽あいを",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);

    skk_context_set_auto_start_henkan_keywords_rs(&mut context, vec!["を".to_string()]);

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i w o",
        "▼愛を",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn escape_from_okuri_ari_register() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "M a g u R o C-g space",
        "▼鮪",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn wide_latin() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "L a b (shift exclam)",
        "",
        "ａｂ！",
        InputMode::Zenkaku,
    );
}

#[test]
fn number_composition() {
    init_test_logger();
    let static_dict =
        CskkDictionary::new_static_dict("tests/data/number_jisyo.dat", "UTF-8").unwrap();
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

///
/// ueno/libskk と動作が違う点だが、v1.0.0でどうするか未定
///
/// 今はCStringの文字コードをUnicode3.2以降のUTF-8まま返す形なのでRust内では問題ない。
/// 受け側で想定するLinuxのいくつかのIMEで使用が難しければ変更するかも。
///
#[test]
fn rom_kana_conversion_vu() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "V u",
        "▽ゔ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "V u q",
        "",
        "ヴ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u",
        "▽ヴ",
        "",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u",
        "▽ヴ",
        "",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Katakana,
        "V u q",
        "",
        "ゔ",
        InputMode::Katakana,
    );
}

#[test]
fn multiple_number_composition() {
    init_test_logger();
    let static_dict =
        CskkDictionary::new_static_dict("tests/data/number_jisyo.dat", "UTF-8").unwrap();
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
    let dict = CskkDictionary::new_user_dict("tests/data/empty.dat", "utf-8").unwrap();
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
        "Q 1 k a i space",
        "▼１かい",
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
