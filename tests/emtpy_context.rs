mod utils;

use crate::utils::init_test_logger;
use crate::utils::transition_check;
use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::CskkContext;
use std::sync::Arc;

#[test]
fn simple_check() {
    init_test_logger();
    let dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/SKK-JISYO.S", "utf-8").unwrap();
    let mut context = CskkContext::new_empty(
        InputMode::Hiragana,
        CompositionMode::Direct,
        vec![Arc::new(dict)],
    );
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "a i",
        "",
        "ai",
        InputMode::Hiragana,
    );
}
