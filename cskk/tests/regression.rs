mod utils;

use crate::utils::{test_context_with_dictionaries, transition_test};
use cskk::cskkstate::{CskkStateInfo, DirectData};
use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::sync::Arc;

// Issue #252
#[test]
fn maruichi() {
    let static_dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/maruichi.dat", "utf-8", false)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(static_dict)]);
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "M a r u 1 space Return",
        CompositionMode::Direct,
        InputMode::Hiragana,
        CskkStateInfo::Direct(DirectData {
            confirmed: "①".to_string(),
            unconverted: None,
        }),
    )
}
