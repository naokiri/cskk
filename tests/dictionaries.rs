use crate::utils::{init_test_logger, test_context_with_dictionaries, transition_test};
use cskk::cskkstate::{CompositionSelectionData, CskkStateInfo};
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use std::fs::copy;
use std::sync::Arc;

mod utils;

///
/// 重複した候補も別アノテーションならば別候補として扱う。
/// 完全重複した場合は順序変更なく、最初に読み込まれた順番で扱う。
///
#[test]
pub fn dictionary_order_should_be_preserved() {
    init_test_logger();
    // Overwrite to the expected default order
    copy(
        "tests/data/dictionaries/order1.dat.orig",
        "tests/data/dictionaries/order1.dat",
    )
    .expect("TODO: Failed to copy and init the user dict.");
    let dict1 = CskkDictionary::new_user_dict("tests/data/dictionaries/order1.dat", "utf-8", false)
        .unwrap();
    let dict2 =
        CskkDictionary::new_static_dict("tests/data/dictionaries/order2.dat", "utf-8", false)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict1), Arc::new(dict2)]);
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "C h o u space",
        CompositionMode::CompositionSelection,
        InputMode::Hiragana,
        CskkStateInfo::CompositionSelection(CompositionSelectionData {
            confirmed: "".to_string(),
            composited: "腸".to_string(),
            okuri: None,
            annotation: None,
        }),
    );
    skk_context_reset_rs(&mut context);
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "C h o u space space",
        CompositionMode::CompositionSelection,
        InputMode::Hiragana,
        CskkStateInfo::CompositionSelection(CompositionSelectionData {
            confirmed: "".to_string(),
            composited: "超".to_string(),
            okuri: None,
            annotation: None,
        }),
    );
    skk_context_reset_rs(&mut context);
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "C h o u space space space",
        CompositionMode::CompositionSelection,
        InputMode::Hiragana,
        CskkStateInfo::CompositionSelection(CompositionSelectionData {
            confirmed: "".to_string(),
            composited: "帳".to_string(),
            okuri: None,
            annotation: None,
        }),
    );
    skk_context_reset_rs(&mut context);
    transition_test(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "C h o u space space space space",
        CompositionMode::CompositionSelection,
        InputMode::Hiragana,
        CskkStateInfo::CompositionSelection(CompositionSelectionData {
            confirmed: "".to_string(),
            composited: "長".to_string(),
            okuri: None,
            annotation: Some("人名".to_string()),
        }),
    );
}
