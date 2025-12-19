mod utils;

use crate::utils::{make_temp_file, test_context_with_dictionaries, transition_check};
use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{skk_context_reload_dictionary, skk_context_reset_rs, skk_context_save_dictionaries_rs};
use std::sync::Arc;

#[test]
fn save_dict() {
    let dict =
        CskkDictionary::new_user_dict("tests/data/dictionaries/userdict.dat", "utf-8", false)
            .unwrap();
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
fn register_and_read() {
    let temppath = make_temp_file().unwrap();
    let empty_dict_path = temppath.to_str().unwrap();
    let static_dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/SKK-JISYO.S", "euc-jp", false)
            .unwrap();
    let user_dict = CskkDictionary::new_user_dict(empty_dict_path, "utf-8", false).unwrap();
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
fn register_more_for_existing_candidate() {
    use crate::utils::init_test_logger;
    init_test_logger();
    let dict = CskkDictionary::new_user_dict(
        "tests/data/dictionaries/register_test_dict.dat",
        "utf-8",
        false,
    )
    .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A space",
        "▼候補",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A space space",
        "▼あ【】",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A space space i Return",
        "",
        "い",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A space",
        "▼い",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A space space",
        "▼候補",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn concat_dict_entry_read() {
    use crate::utils::init_test_logger;
    init_test_logger();
    let dict =
        CskkDictionary::new_static_dict("tests/data/dictionaries/concat_dict.dat", "utf-8", false)
            .unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "D o s u space",
        "▼DOS/V",
        "",
        InputMode::Hiragana,
    );
}
