mod utils;

use crate::utils::{
    default_test_context, make_temp_file, test_context_with_dictionaries, transition_check,
};
use cskk::dictionary::CskkDictionary;
use cskk::skk_context_reset_rs;
use cskk::skk_modes::{CompositionMode, InputMode};
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;

#[test]
fn non_ascii_transition() {
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Ascii,
        "plus",
        "",
        "+",
        InputMode::Ascii,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "plus",
        "",
        "+",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K plus",
        "▽+",
        "",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    // 現在のueno/libskk と差異がある部分？
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "K a K plus",
        "▽か*+",
        "",
        InputMode::Hiragana,
    );
}

#[test]
fn semicolon_entry() {
    use crate::utils::init_test_logger;
    init_test_logger();
    let temppath = make_temp_file().unwrap();
    let empty_dict_path = temppath.to_str().unwrap();
    let user_dict = CskkDictionary::new_user_dict(empty_dict_path, "utf-8", false).unwrap();
    let mut context = test_context_with_dictionaries(vec![Arc::new(user_dict)]);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Z o u g o space l semicolon C-j a l semicolon Return",
        "",
        ";あ;",
        InputMode::Hiragana,
    );
    skk_context_reset_rs(&mut context);
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "Z o u g o space Return",
        "",
        ";あ;",
        InputMode::Hiragana,
    );
    context.save_dictionary();

    let dict_file = File::open(empty_dict_path).unwrap();
    let enc = Encoding::for_label_no_replacement("utf-8".as_bytes());
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(enc)
        .build(dict_file);
    let reader = BufReader::new(decoder);
    for line in reader.lines().flatten() {
        if !line.starts_with(";;") {
            assert_eq!(line.chars().filter(|x| x.eq(&';')).count(), 0);
        }
    }
}

#[test]
fn preconversion_clear_and_input() {
    use crate::utils::init_test_logger;
    init_test_logger();
    let mut context = default_test_context();
    transition_check(
        &mut context,
        CompositionMode::Direct,
        InputMode::Hiragana,
        "c minus",
        "",
        "ー",
        InputMode::Hiragana,
    );
}
