mod utils;

use crate::utils::{
    default_test_context, init_test_logger, make_temp_file, test_context_with_dictionaries,
    transition_check,
};
use cskk::dictionary::CskkDictionary;
use cskk::skk_modes::{CompositionMode, InputMode};
use cskk::{skk_context_reset_rs, skk_context_set_auto_start_henkan_keywords_rs};
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

#[test]
fn auto_start_henkan() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i period",
        "▼愛。",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i comma",
        "▼愛、",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i bracketright",
        "▼愛」",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn set_auto_start_henkan() {
    init_test_logger();
    let mut context = default_test_context();

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i w o",
        "▽あいを",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);

    skk_context_set_auto_start_henkan_keywords_rs(&mut context, vec!["を".to_string()]);

    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A i w o",
        "▼愛を",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn auto_start_henkan_cleanup() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A K i A i space",
        "▼愛",
        "飽き",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
}

#[test]
fn auto_start_henkan_without_kana() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "C period",
        "▽。",
        "",
        InputMode::Hiragana,
    );
}

// Issue #222
#[test]
fn auto_start_henkan_no_candidate() {
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A a period",
        "▼ああ。【】",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn auto_start_henkan_register_not_includes_period() {
    init_test_logger();
    let temppath = make_temp_file().unwrap();
    let new_dict_path = temppath.to_str().unwrap();
    let user_dict = CskkDictionary::new_user_dict(new_dict_path, "utf-8", false).unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(user_dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "A a period a Return",
        "",
        "あ。",
        InputMode::Hiragana,
    );
    context.save_dictionary();

    let dict_file = File::open(new_dict_path).unwrap();
    let enc = Encoding::for_label_no_replacement("utf-8".as_bytes());
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(enc)
        .build(dict_file);
    let reader = BufReader::new(decoder);
    for line in reader.lines().flatten() {
        if !line.starts_with(";;") {
            assert_eq!(line, "ああ /あ/");
        }
    }
}
