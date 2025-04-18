use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{skk_context_reset_rs, CskkContext};

fn azik_test_context() -> CskkContext {
    let mut context = default_test_context();
    context
        .set_rule_from_directory("azik", "../assets/rules")
        .unwrap();
    context
}

#[test]
fn basic() {
    init_test_logger();
    let mut context = azik_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "semicolon",
        "",
        "っ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

// SKKのコマンドのlとAZIKのxと被るので、xxを拗音に当てている。
// AZIKでは通常拗音単体ではなく拗音拡張を用いる。
#[test]
fn xx_youon() {
    init_test_logger();
    let mut context = azik_test_context();

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "x x a",
        "",
        "ぁ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn check_default_rule_removed() {
    init_test_logger();
    let mut context = azik_test_context();

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "t h e",
        "",
        "つうえ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "c a",
        "",
        "ちゃ",
        InputMode::Hiragana,
    );
}

#[test]
fn using_q() {
    init_test_logger();
    let mut context = azik_test_context();

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "q",
        "",
        "ん",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn check_two_letter_keysyms() {
    init_test_logger();
    let mut context = azik_test_context();

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "m u",
        "",
        "む",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "h t",
        "",
        "ひと",
        InputMode::Hiragana,
    );
}

// https://github.com/ueno/libskk/issues/82
// libskkのように'['ではなく、cskkでは'@'をデフォルトのモード切り替えに当てている。
#[test]
fn mode_switching_key() {
    init_test_logger();
    let mut context = azik_test_context();

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "bracketleft",
        "",
        "「",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "(control at)",
        "",
        "",
        InputMode::HankakuKatakana,
    );
}

#[test]
fn hankaku_mode_changing() {
    init_test_logger();
    let mut context = azik_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "n n C-at",
        "",
        "ん",
        InputMode::HankakuKatakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::HankakuKatakana,
        "n n C-at",
        "",
        "ﾝ",
        InputMode::Katakana,
    );
}

#[test]
fn hiragana_precomposition_input() {
    init_test_logger();
    let mut context = azik_test_context();
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::Hiragana,
        "a at",
        "",
        "ア",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::Hiragana,
        "a C-at",
        "",
        "ｱ",
        InputMode::Hiragana,
    );
}

#[test]
fn katakana_precomposition_input() {
    init_test_logger();
    let mut context = azik_test_context();
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::Katakana,
        "a at",
        "",
        "あ",
        InputMode::Katakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::Katakana,
        "a C-at",
        "",
        "ｱ",
        InputMode::Katakana,
    );
}

#[test]
fn hankaku_precomposition_input() {
    init_test_logger();
    let mut context = azik_test_context();
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::HankakuKatakana,
        "a at",
        "",
        "あ",
        InputMode::HankakuKatakana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::PreComposition,
        InputMode::HankakuKatakana,
        "a C-at",
        "",
        "ア",
        InputMode::HankakuKatakana,
    );
}

// Issue#227
// 「買った」の変換を開始できない問題
#[test]
fn original_small_tu() {
    init_test_logger();
    let mut context = azik_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a T U",
        "▼勝っ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a t U",
        "▽かっ",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "t U",
        "",
        "っ",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T U",
        "▽っ",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn original_small_tu_longer() {
    init_test_logger();
    let mut context = azik_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "x x t u",
        "",
        "っ",
        InputMode::Hiragana,
    );
}
