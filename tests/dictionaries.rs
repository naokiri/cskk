use std::sync::Arc;
use cskk::cskkstate::{CompositionSelectionData, CskkStateInfo, DirectData};
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use crate::utils::{default_test_context, init_test_logger, test_context_with_dictionaries, transition_test};

mod utils;

#[test]
pub fn completion_mode_from_direct() {
    init_test_logger();
    let dict1 =
        CskkDictionary::new_static_dict("tests/data/dictionaries/order1.dat", "utf-8", false)
            .unwrap();
    let dict2 =
        CskkDictionary::new_static_dict("tests/data/dictionaries/order2.dat", "utf-8", false)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict1), Arc::new(dict2)]);
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T a t i space",
        CompositionMode::CompositionSelection,
        InputMode::Hiragana,
        CskkStateInfo::CompositionSelection(CompositionSelectionData{
            confirmed: "".to_string(),
            composited:"太刀".to_string(),
            okuri: None,
            annotation: None
        }),
    );
    skk_context_reset_rs(&mut context);
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T a t i space space",
        CompositionMode::CompositionSelection,
        InputMode::Hiragana,
        CskkStateInfo::CompositionSelection(CompositionSelectionData{
            confirmed: "".to_string(),
            composited:"大刀".to_string(),
            okuri: None,
            annotation: None
        }),
    );
    skk_context_reset_rs(&mut context);
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "T a t i space space space",
        CompositionMode::CompositionSelection,
        InputMode::Hiragana,
        CskkStateInfo::CompositionSelection(CompositionSelectionData{
            confirmed: "".to_string(),
            composited:"達".to_string(),
            okuri: None,
            annotation: None
        }),
    );
}