use crate::utils::{
    default_test_context, init_test_logger, test_context_with_dictionaries, transition_check,
};
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};

#[test]
fn z_slash() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "z slash",
        "",
        "・",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn mu() {
    init_test_logger();
    let mut context = default_test_context();
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
}
