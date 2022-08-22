use crate::utils::{
    default_test_context, init_test_logger, test_context_with_dictionaries, transition_check,
};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn basic() {
    init_test_logger();
    let mut context = default_test_context();
    context.set_rule("azik").unwrap();

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
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "k u x w a",
        "",
        "くゎ",
        InputMode::Hiragana,
    );
}

#[test]
fn check_default_rule_removed() {
    init_test_logger();
    let mut context = default_test_context();
    context.set_rule("azik").unwrap();

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
    let mut context = default_test_context();
    context.set_rule("azik").unwrap();

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
    let mut context = default_test_context();
    context.set_rule("azik").unwrap();

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
