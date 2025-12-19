mod utils;

use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

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

/// 参照時のueno/libskk と ddskkで動作が違う例。
/// ddskk式の方が現実的な打ち間違え時になってほしい状態なので採用
///
/// ueno/libskk
/// "k A" -> ▽あ
/// ddskk
/// "k A" -> ▽か
#[test]
fn precomposition_mode_from_middle() {
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

///
/// 大文字をかな変換の要素に入れられる。
///
#[test]
fn using_capital_letter() {
    init_test_logger();
    use cskk::dictionary::CskkDictionary;
    use cskk::CskkContext;
    use std::sync::Arc;

    let dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/SKK-JISYO.S", "euc-jp", false)
            .unwrap();
    let mut context = CskkContext::new_from_specified_paths(
        InputMode::Ascii,
        CompositionMode::Direct,
        vec![Arc::new(dict)],
        "../assets/rule/kana_form.toml",
        "../assets/rule/ascii_form.toml",
        "tests/data/rules",
    );

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "t s U",
        "",
        "ちいさいっ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

///
/// 付ける の T u K e のようにせわしなくShift押したり離したりするのは失敗しやすいため、
/// T u K E のように k e -> け という1変換の間にはShift押しっぱなしでもモード変更等として捉えない。
///
#[test]
fn continuous_capital_kana_converting() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T u K E",
        "▼付け",
        "",
        InputMode::Hiragana,
    );
}
