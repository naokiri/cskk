mod utils;

use crate::utils::{default_test_context, init_test_logger, transition_check};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

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

///
/// Shiftキーを押したままlキーを押した時、単なるLのつもりで {"Ｌ", modifier: SHIFT} となるので、
/// 大文字の場合シフト抜きの素のキーとしてコマンドを検索する。
/// 小文字の場合は通常のキーボードで存在しないが、一応そのまま別のものとして扱う。
///
#[test]
fn allow_shift_as_part_of_capital_only() {
    init_test_logger();
    let mut context = default_test_context();
    // 普通のキーボードでは存在しえないが、Shift-lとしてlとは別に扱うことにしている。
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "(shift l)",
        "",
        "l",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "(shift L)",
        "",
        "",
        InputMode::Zenkaku,
    );
}

// Issue #260
#[test]
fn reset_should_reset_composition_mode() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "J i space space space",
        "▼自",
        "",
        InputMode::Hiragana,
    );

    skk_context_reset_rs(&mut context);
    assert_eq!(
        context.get_current_composition_mode(),
        CompositionMode::Direct
    );
}
