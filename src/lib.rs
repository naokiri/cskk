#[macro_use]
extern crate bitflags;
#[cfg(test)]
extern crate env_logger;
extern crate sequence_trie;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate xkbcommon;

use std::fmt::Display;
use std::fmt::Formatter;
use std::iter::FromIterator;
use std::{fmt, slice};

use log::debug;

use crate::command_handler::direct_mode_command_handler::DirectModeCommandHandler;
use crate::command_handler::kana_composition_handler::KanaCompositionHandler;
use crate::command_handler::kana_precomposition_handler::KanaPrecompositionHandler;
use crate::command_handler::CommandHandler;
use crate::dictionary::file_dictionary::FileDictionary;
use crate::dictionary::static_dict::StaticFileDict;
use crate::dictionary::{CskkDictionary, Dictionary};
use crate::kana_builder::KanaBuilder;
use crate::kana_form_changer::KanaFormChanger;
use crate::keyevent::KeyEventSeq;
use crate::keyevent::{KeyEvent, SkkKeyModifier};
use crate::skk_modes::CompositionMode::PreCompositionOkurigana;
use crate::skk_modes::InputMode;
use crate::skk_modes::{has_rom2kana_conversion, CompositionMode, PeriodStyle};
use log::warn;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use xkbcommon::xkb;

mod command_handler;
pub mod dictionary;
pub mod error;
mod kana_builder;
mod kana_form_changer;
mod keyevent;
pub mod skk_modes;

#[derive(Debug, PartialEq)]
pub(crate) enum Instruction {
    // Abort current composition selection, registration
    Abort,
    ChangeInputMode(InputMode),
    /// Try to convert preconversion if in input mode which has conversion. Mostly (or only?) just for single 'n' conversion.
    #[allow(clippy::upper_case_acronyms)]
    OutputNNIfAny(InputMode),
    FlushPreviousCarryOver,
    // Aborts if empty after flush. flush条件用に必要？
    // AbortIfEmptyKanaToConvert,
    ChangeCompositionMode {
        composition_mode: CompositionMode,
        // FIXME: PrecompositionからspaceでCompositionを働かせるためにDelegateを作ったが、Delegate無限ループに陥いらないようにする仕組みがない。
        // delegate: delegate processing current key. Run the key event handling again on the next mode with current key.
        delegate: bool,
    },
    // モード変更などで入力を処理し、入力モードの入力としての処理をしない命令
    FinishConsumingKeyEvent,
    // keyeventを処理しなかったとして処理を終了する。ueno/libskkでの"*-unhandled"系命令用
    #[allow(dead_code)]
    FinishNotConsumingKeyEvent,
    // 今の変換候補を変更する。
    SetComposition {
        kanji: String,
    },
    // 変換候補ポインタを進める
    NextCandidatePointer,
    // 現在の変換候補で確定する
    ConfirmComposition,
    ConfirmAsHiragana,
    ConfirmAsKatakana,
    #[allow(clippy::upper_case_acronyms)]
    ConfirmAsJISX0201,
    ConfirmDirect,
    // 現在の候補を辞書から消す
    Purge,
}

/// Rough design prototype yet
///
/// TODO: Rustのstructまわりの一部分mutに変更があったら非mutでstateアクセスしているところを直す
///
pub struct CskkContext {
    state_stack: Vec<CskkState>,
    direct_handler: DirectModeCommandHandler,
    kana_precomposition_handler: KanaPrecompositionHandler,
    kana_composition_handler: KanaCompositionHandler,
    kana_converter: Box<KanaBuilder>,
    kana_form_changer: KanaFormChanger,
}

/// Rough prototype yet.
///
#[derive(Debug)]
struct CskkState {
    input_mode: InputMode,
    composition_mode: CompositionMode,
    // 直前のCompositionMode。Abort時に元に戻すmode。
    previous_composition_mode: CompositionMode,
    // 入力文字で、かな確定済みでないものすべて
    pre_conversion: Vec<char>,
    // 変換辞書のキーとなる部分。送りなし変換の場合はconverted_kana_to_composite と同じ。送りあり変換時には加えてconverted_kana_to_okuriの一文字目の子音や'>'付き。Abbrebiation変換の場合kana-convertされる前の入力など
    raw_to_composite: String,
    // 未確定入力をInputモードにあわせてかな変換したもののうち、漢字の読み部分。convertがあるInputMode時のみ使用
    converted_kana_to_composite: String,
    // 未確定入力をInputモードにあわせてかな変換したもののうち、おくり仮名部分。convertがあるInputMode時のみ使用
    converted_kana_to_okuri: String,
    // 入力を漢字変換した現在の選択肢。送り仮名含まない
    composited_kanji: String,
    // 入力を漢字変換した現在の選択肢の送り仮名部分。
    composited_okuri: String,
    // 確定済み入力列。pollされた時に渡してflushされるもの。
    confirmed: String,
    // 変換中の選択肢
    //composition_candidates: &Vec<Arc<Candidate>>,
    // 変換中の選択肢のうち、どれをさしているか
    selection_pointer: usize,
}

impl CskkState {
    pub fn new(input_mode: InputMode, composition_mode: CompositionMode) -> Self {
        CskkState {
            input_mode,
            composition_mode,
            previous_composition_mode: composition_mode,
            pre_conversion: vec![],
            raw_to_composite: "".to_string(),
            converted_kana_to_composite: "".to_string(),
            converted_kana_to_okuri: "".to_string(),
            composited_kanji: "".to_string(),
            composited_okuri: "".to_string(),
            confirmed: "".to_string(),
            selection_pointer: 0,
        }
    }

    fn preedit_string(&self) -> String {
        let converted = &self.confirmed;
        let unconverted = &self.pre_conversion;
        let kana_to_composite = &self.converted_kana_to_composite;
        let kana_to_okuri = &self.converted_kana_to_okuri;
        let composited = &self.composited_kanji;
        let composited_okuri = &self.composited_okuri;

        match self.composition_mode {
            CompositionMode::Direct => {
                converted.to_owned() + &String::from_iter(unconverted.iter())
            }
            CompositionMode::PreComposition => {
                "▽".to_owned()
                    + converted
                    + kana_to_composite
                    + &String::from_iter(unconverted.iter())
            }
            CompositionMode::PreCompositionOkurigana => {
                "▽".to_owned()
                    + converted
                    + kana_to_composite
                    + "*"
                    + kana_to_okuri
                    + &String::from_iter(unconverted.iter())
            }
            CompositionMode::CompositionSelection => "▼".to_owned() + composited + composited_okuri,
            CompositionMode::Register => {
                if kana_to_okuri.is_empty() {
                    "▼".to_string() + kana_to_composite
                } else {
                    "▼".to_string() + kana_to_composite + "*" + kana_to_okuri
                }
            }
            CompositionMode::Abbreviation => {
                "Abbreviaton mode. detail not implemented.".to_string()
            }
        }
    }
}

///
/// Creates a skk file dict based on the path_string. Returns the pointer of it.
///
/// # Safety
/// c_path_string and c_encoidng must be a valid c string that terminates with \0.
///
/// Dictionary must be handled by a cskk context on creating a new context or registering later.
/// If not, memory leaks.
///
#[no_mangle]
pub unsafe extern "C" fn skk_file_dict_new(
    c_path_string: *const c_char,
    c_encoding: *const c_char,
) -> *mut CskkDictionary {
    let path = CStr::from_ptr(c_path_string);
    let encoding = CStr::from_ptr(c_encoding);

    Box::into_raw(Box::new(skk_file_dict_new_rs(
        path.to_str().unwrap(),
        encoding.to_str().unwrap(),
    )))
}

pub fn skk_file_dict_new_rs(path_string: &str, encoding: &str) -> CskkDictionary {
    CskkDictionary::StaticFile(StaticFileDict::new(path_string, encoding))
}

/// Returns newly allocated CSKKContext.
///
/// # Safety
/// Caller have to retain the pointer
/// Caller must free the memory using skk_context_destroy
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_new(
    dictionary_array: *const *mut CskkDictionary,
    dictionary_count: usize,
) -> *mut CskkContext {
    let tmp_array = slice::from_raw_parts(dictionary_array, dictionary_count);
    let mut dict_array = vec![];
    for dictref in tmp_array {
        let cskkdict = *Box::from_raw(*dictref);
        dict_array.push(cskkdict);
    }
    Box::into_raw(Box::new(skk_context_new_rs(dict_array)))
}

pub fn skk_context_new_rs(dictionaries: Vec<CskkDictionary>) -> CskkContext {
    CskkContext::new(InputMode::Hiragana, CompositionMode::Direct, dictionaries)
}

/// Reset the context
#[no_mangle]
pub extern "C" fn skk_context_reset(context: &mut CskkContext) {
    // TODO: Flush all the state stack after implementing the register mode
    // TODO: あとまわし。他のテストがこけはじめたらちゃんと実装する。
    context.poll_output();
    context.reset_state_stack();
}

/// Test purpose
/// # Safety
///
/// This function must be called by a valid C string terminated by a NULL.
#[no_mangle]
pub unsafe extern "C" fn skk_context_process_key_events(
    context: &mut CskkContext,
    keyevents_cstring: *mut c_char,
) -> bool {
    let keyevents = CStr::from_ptr(keyevents_cstring);
    context.process_key_events_string(keyevents.to_str().unwrap())
}

/// Test purpose
pub fn skk_context_process_key_events_rs(context: &mut CskkContext, keyevents: &str) -> bool {
    context.process_key_events_string(keyevents)
}

/// テスト用途。composition modeを設定する。
/// 他のステートとの整合性は無視される。
#[no_mangle]
pub extern "C" fn skk_context_set_composition_mode(
    context: &mut CskkContext,
    composition_mode: CompositionMode,
) {
    context.set_composition_mode(composition_mode)
}

/// テスト用途。input_modeを設定する。
/// 他のステートとの整合性は無視される。
#[no_mangle]
pub extern "C" fn skk_context_set_input_mode(context: &mut CskkContext, input_mode: InputMode) {
    context.set_input_mode(input_mode)
}

/// 現在のoutputをpollingする。
///
/// # Safety
/// 返り値のポインタの文字列を直接編集して文字列長を変えてはいけない。
/// 返り値はcallerがskk_free_stringしないと実体がメモリリークする。
///
#[no_mangle]
pub extern "C" fn skk_context_poll_output(context: &mut CskkContext) -> *mut c_char {
    CString::new(skk_context_poll_output_rs(context))
        .unwrap()
        .into_raw()
}

/// 現在のoutputをpollingする。
///
pub fn skk_context_poll_output_rs(context: &mut CskkContext) -> String {
    if let Some(str) = context.poll_output() {
        return str;
    }
    "".to_string()
}

///
/// Period style を設定する
///
#[no_mangle]
pub extern "C" fn skk_context_set_period_style(
    context: &mut CskkContext,
    period_style: PeriodStyle,
) {
    context.kana_converter.set_period_style(period_style)
}

/// テスト用途？。preedit文字列と同じ内容の文字列を取得する。
///
/// # Safety
/// 返り値のポインタの文字列を直接編集して文字列長を変えてはいけない。
/// 返り値はcallerがskk_free_stringしないとメモリリークする。
/// ueno/libskkと違う点なので注意が必要
///
#[no_mangle]
pub extern "C" fn skk_context_get_preedit(context: &CskkContext) -> *mut c_char {
    let preedit = context.get_preedit().unwrap();
    CString::new(preedit).unwrap().into_raw()
}

/// テスト用途？。preedit文字列と同じ内容の文字列を取得する。
///
pub fn skk_context_get_preedit_rs(context: &CskkContext) -> String {
    context.get_preedit().unwrap()
}

/// テスト用途？
pub fn skk_context_get_compositon_mode(context: &CskkContext) -> CompositionMode {
    context.current_state_ref().composition_mode
}

/// テスト用途？
pub fn skk_context_get_input_mode(context: &CskkContext) -> InputMode {
    context.current_state_ref().input_mode
}

///
/// cskk libraryが渡したC言語文字列をfreeする。
///
/// # Safety
///
/// CSKKライブラリで返したC言語文字列のポインタ以外を引数に渡してはいけない。
/// 他で管理されるべきメモリを過剰に解放してしまう。
///
#[no_mangle]
pub unsafe extern "C" fn skk_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // Get back ownership in Rust side, then do nothing.
    CString::from_raw(ptr);
}

///
/// save current dictionaries
///
#[no_mangle]
pub extern "C" fn skk_context_save_dictionaries(context: &mut CskkContext) {
    context.save_dictionary();
}

///
/// reload current dictionaries
/// For integration test purpose.
///
pub fn skk_context_reload_dictionary(context: &mut CskkContext) {
    context.reload_dictionary();
}

///
/// CskkContextを解放する。
///
/// # Safety
///
/// context_ptr は必ずCskkContextのポインタでなければならない。
///
#[no_mangle]
pub unsafe extern "C" fn skk_context_free(context_ptr: *mut CskkContext) {
    if context_ptr.is_null() {
        return;
    }
    Box::from_raw(context_ptr);
}

impl CskkContext {
    ///
    /// Retrieve and remove the current output string
    ///
    pub fn poll_output(&mut self) -> Option<String> {
        self.retrieve_output(true)
    }

    ///
    /// pollされていない入力を状態に応じて修飾して返す。
    /// preeditという名称はueno/libskkより。
    ///
    pub fn get_preedit(&self) -> Option<String> {
        let mut result = String::new();
        let mut stack_count = 0;
        for state in &self.state_stack {
            result.push_str(&state.preedit_string());
            if state.composition_mode == CompositionMode::Register {
                stack_count += 1;
                result.push('【');
            }
        }
        for _ in 0..stack_count {
            result.push('】');
        }
        Some(result)
    }

    ///
    /// 確定済文字を返す。
    /// IM側からのpolling用途でなければ、状態を変えない。
    /// IMからのpollingで出力用途ならば、flushする。
    ///
    fn retrieve_output(&mut self, is_polling: bool) -> Option<String> {
        let topmost_state = self
            .state_stack
            .get_mut(0)
            .expect("State would never be empty");

        if topmost_state.confirmed.is_empty() {
            None
        } else {
            let out = topmost_state.confirmed.clone();
            if is_polling {
                topmost_state.confirmed.clear();
            }
            Some(out)
        }
    }

    fn append_confirmed_raw_char(&mut self, result: char) {
        let current_state = self.current_state();
        current_state.confirmed.push(result);
    }

    fn append_converted(&mut self, result: &str) {
        let current_state = self.current_state();
        let current_input_mode = current_state.input_mode;
        self.append_converted_in_input_mode(result, &current_input_mode)
    }

    fn append_converted_in_input_mode(&mut self, result: &str, input_mode: &InputMode) {
        let kana_form_changer = &self.kana_form_changer;
        let adjusted = kana_form_changer.adjust_kana_string(input_mode, &result);
        let current_state = self.current_state();
        current_state.confirmed.push_str(&adjusted);
    }

    fn append_unconverted(&mut self, unconv: char) {
        let current_state = self.current_state();
        current_state.pre_conversion.push(unconv);
    }

    fn append_converted_to_composite(&mut self, result: &str) {
        let kana_form_changer = &self.kana_form_changer;
        let current_input_mode = &self.current_state_ref().input_mode;
        let adjusted = kana_form_changer.adjust_kana_string(current_input_mode, &result);
        let current_state = self.current_state();
        current_state
            .converted_kana_to_composite
            .push_str(&adjusted);
        current_state.raw_to_composite.push_str(result);
    }

    fn append_converted_to_okuri(&mut self, result: &str) {
        let kana_form_changer = &self.kana_form_changer;
        let current_input_mode = &self.current_state_ref().input_mode;
        let adjusted = kana_form_changer.adjust_kana_string(current_input_mode, &result);
        let current_state = self.current_state();

        current_state.converted_kana_to_okuri.push_str(&adjusted);
    }

    fn append_to_composite_iff_no_preconversion(&mut self, to_composite_last: char) {
        let current_state = self.current_state();
        if current_state.pre_conversion.is_empty() {
            current_state
                .raw_to_composite
                .push_str(&to_composite_last.to_string())
        }
    }

    /// Append a char to raw_to_composite without checking.
    /// Usually use append_to_composite_iff_no_preconversion.
    #[allow(dead_code)]
    fn append_raw_to_composite(&mut self, to_composite_last: char) {
        let current_state = self.current_state();
        current_state
            .raw_to_composite
            .push_str(&to_composite_last.to_string())
    }

    // to_compositeをconvertedの内容でリセットする。
    #[allow(dead_code)]
    fn set_to_composite_to_converted_kana(&mut self) {
        let mut current_state = self.current_state();
        current_state.raw_to_composite = current_state.converted_kana_to_composite.clone()
    }

    fn reset_unconverted(&mut self) {
        let current_state = self.current_state();
        current_state.pre_conversion.clear();
    }

    fn set_carry_over(&mut self, unconv: &[char]) {
        let current_state = self.current_state();
        current_state.pre_conversion = unconv.to_owned();
    }

    fn increment_selection_pointer(&mut self) {
        let current_state = self.current_state();
        current_state.selection_pointer += 1;
    }

    fn set_composition_candidate(&mut self, kanji: &str) {
        let current_state = self.current_state();
        let okuri = current_state.converted_kana_to_okuri.to_owned();
        current_state.composited_kanji = kanji.to_owned();
        current_state.composited_okuri = okuri;
    }

    fn purge_current_composition_candidate(&mut self) {
        let composited_kanji = self.current_state_ref().composited_kanji.to_owned();
        let raw_to_composite = self.current_state_ref().raw_to_composite.to_owned();
        self.kana_composition_handler.purge_candidate(
            &raw_to_composite,
            !self.current_state_ref().converted_kana_to_okuri.is_empty(),
            &composited_kanji,
        );

        self.reset_current_state();
    }

    fn confirm_current_composition_candidate(&mut self) {
        let composited_kanji = self.current_state_ref().composited_kanji.to_owned();
        let raw_to_composite = self.current_state_ref().raw_to_composite.to_owned();
        self.kana_composition_handler.confirm_candidate(
            &raw_to_composite,
            !self.current_state_ref().converted_kana_to_okuri.is_empty(),
            &composited_kanji,
        );

        let composited_okuri = &self.current_state_ref().composited_okuri;
        let composited_kanji_okuri = composited_kanji + composited_okuri;
        let current_state = self.current_state();
        current_state.confirmed.push_str(&composited_kanji_okuri);
        current_state.composited_kanji.clear();
        current_state.composited_okuri.clear();
        current_state.raw_to_composite.clear();
        current_state.converted_kana_to_composite.clear();
        current_state.converted_kana_to_okuri.clear();
    }

    fn confirm_current_kana_to_composite(&mut self, temporary_input_mode: &InputMode) {
        let kana_form_changer = &self.kana_form_changer;
        let kana = kana_form_changer.adjust_kana_string(
            temporary_input_mode,
            &self.current_state_ref().raw_to_composite,
        );

        let current_state = self.current_state();
        current_state.pre_conversion.clear();
        current_state.confirmed.push_str(&kana);
        current_state.raw_to_composite.clear();
        current_state.converted_kana_to_composite.clear();
        current_state.converted_kana_to_okuri.clear();
    }

    fn reset_carry_over(&mut self) -> bool {
        let current_state = self.current_state();
        let do_reset = !current_state.pre_conversion.is_empty();
        current_state.pre_conversion.clear();
        do_reset
    }

    fn reset_converted_kanas(&mut self) {
        let current_state = self.current_state();
        current_state.raw_to_composite.clear();
        current_state.converted_kana_to_composite.clear();
        current_state.converted_kana_to_okuri.clear();
    }

    fn reset_composited(&mut self) {
        let current_state = self.current_state();
        current_state.composited_kanji.clear();
        current_state.composited_okuri.clear();
    }

    fn consolidate_converted_to_to_composite(&mut self) {
        let current_state = self.current_state();
        let okuri = current_state.converted_kana_to_okuri.clone();
        current_state.converted_kana_to_composite.push_str(&okuri);
        current_state.converted_kana_to_okuri.clear();
    }

    /// Set the current composition mode.
    fn set_composition_mode(&mut self, composition_mode: CompositionMode) {
        let mut current_state = self.current_state();
        current_state.previous_composition_mode = current_state.composition_mode;
        current_state.composition_mode = composition_mode;
        current_state.selection_pointer = 0;
    }

    fn enter_register_mode(&mut self, previous_composition_mode: CompositionMode) {
        self.current_state().previous_composition_mode = previous_composition_mode;
        self.current_state().composition_mode = CompositionMode::Register;
        self.state_stack
            .push(CskkState::new(InputMode::Hiragana, CompositionMode::Direct))
    }

    fn reset_state_stack(&mut self) {
        while self.state_stack.len() > 1 {
            self.state_stack.pop();
        }
        self.reset_current_state();
    }

    fn reset_current_state(&mut self) {
        self.reset_carry_over();
        self.reset_converted_kanas();
        self.reset_unconverted();
        self.reset_composited();
    }

    fn abort_register_mode(&mut self) {
        if self.state_stack.len() > 1 {
            self.state_stack.pop();
            if self.current_state_ref().previous_composition_mode
                == CompositionMode::PreCompositionOkurigana
            {
                // おくり文字を繋げてPreCompositionにする
                self.current_state().composition_mode = CompositionMode::PreComposition;
            } else {
                self.current_state().composition_mode =
                    self.current_state_ref().previous_composition_mode;
            }
            let mut kana_to_composite =
                self.current_state_ref().converted_kana_to_composite.clone();
            kana_to_composite.push_str(&self.current_state_ref().converted_kana_to_okuri);
            self.current_state().converted_kana_to_composite = kana_to_composite;
            self.current_state().converted_kana_to_okuri.clear();
        }
    }

    fn exit_register_mode(&mut self, kanji: &str) {
        if self.state_stack.len() > 1 {
            self.state_stack.pop();
            if kanji.is_empty() {
                self.current_state().composition_mode =
                    self.current_state_ref().previous_composition_mode;
            } else {
                let current_state = self.current_state();
                current_state.composition_mode = CompositionMode::Direct;
                self.set_composition_candidate(kanji);
                self.confirm_current_composition_candidate();
            }
        }
    }

    fn set_input_mode(&mut self, input_mode: InputMode) {
        let mut current_state = self.current_state();
        current_state.input_mode = input_mode
    }

    ///
    /// return true if output ん
    ///
    fn output_nn_if_any(
        &mut self,
        input_mode: &InputMode,
        composition_mode: &CompositionMode,
    ) -> bool {
        // I cannot think of other case than single 'n' being orphaned. This implementation would be enough for it.
        let current_state = self.current_state_ref();
        let unprocessed = current_state.pre_conversion.clone();
        if unprocessed.len() == 1 && unprocessed[0] == 'n' {
            return match composition_mode {
                CompositionMode::Direct => {
                    self.append_converted_in_input_mode("ん", input_mode);
                    true
                }
                CompositionMode::PreComposition => {
                    self.append_converted_to_composite("ん");
                    true
                }
                CompositionMode::PreCompositionOkurigana => {
                    self.append_converted_to_okuri("ん");
                    //self.append_raw_to_composite('n');
                    self.append_to_composite_iff_no_preconversion('n');
                    let current_state = self.current_state();
                    current_state.pre_conversion = vec![];
                    true
                }
                _ => false,
            };
        }
        false
    }

    ///
    /// process that key event and change the internal states.
    /// if key_event is not processable by current CSKK state, then return false
    ///
    pub fn process_key_event(&mut self, key_event: &KeyEvent) -> bool {
        self.process_key_event_inner(key_event, false)
    }

    pub fn save_dictionary(&mut self) {
        for dictionary in self.kana_composition_handler.get_dictionaries() {
            let result = match dictionary {
                CskkDictionary::StaticFile(dictionary) => dictionary.save_dictionary(),
                CskkDictionary::UserFile(dictionary) => dictionary.save_dictionary(),
            };
            match result {
                Ok(_) => {}
                Err(error) => {
                    warn!("{}", &error.to_string());
                }
            }
        }
    }

    pub fn reload_dictionary(&mut self) {
        for cskkdict in self.kana_composition_handler.get_dictionaries() {
            let result = match cskkdict {
                CskkDictionary::StaticFile(ref mut dictionary) => dictionary.reload(),
                CskkDictionary::UserFile(ref mut dictionary) => dictionary.reload(),
            };
            match result {
                Ok(_) => {}
                Err(error) => {
                    warn!("{}", &error.to_string());
                }
            }
        }
    }

    // FIXME: まだ良いルールが把握できていない中でインクリメンタルに機能を追加しているのでぐちゃぐちゃ。一通り機能ができてバグ修正できたらリファクタリング
    ///
    /// 上から順に
    /// rom2kana可能? -yes-> かな入力として処理 (大文字のみcompositionmode変更コマンドとしても処理)
    /// 現在のCompositionMode内で解釈されるコマンド？ -yes-> compositionmode用コマンドとして処理
    /// rom2kana継続可能 or ascii？ -yes-> 継続入力として処理
    /// rom2kana継続不可能 -all-> Flush後に入力として処理
    ///
    fn process_key_event_inner(&mut self, key_event: &KeyEvent, is_delegated: bool) -> bool {
        dbg!(key_event);
        let kana_converter = &self.kana_converter;
        let current_state = self.current_state_ref();
        let initial_composition_mode = current_state.composition_mode;
        let initial_unprocessed_vector = &current_state.pre_conversion.clone();
        let combined_keys = KanaBuilder::combined_key(key_event, initial_unprocessed_vector);
        let modifier = key_event.get_modifier();

        // Shift以外のmodifierがある場合はかな変換とみなさない。
        if (modifier - SkkKeyModifier::SHIFT).is_empty()
            && has_rom2kana_conversion(&current_state.input_mode, &current_state.composition_mode)
        {
            if let Some((converted, carry_over)) = kana_converter.convert(&combined_keys) {
                // clone to live long after possibly changing composition mode.
                let converted = converted.clone();
                let carry_over = carry_over.clone();

                let symbol = key_event.get_symbol();
                // must clone to allow changing state
                let initial_composition_mode = &current_state.composition_mode.clone();
                let is_capital = xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z;
                if is_capital && *initial_composition_mode == CompositionMode::Direct {
                    self.set_composition_mode(CompositionMode::PreComposition);
                } else if is_capital
                    && *initial_composition_mode == CompositionMode::PreComposition
                    && !current_state.raw_to_composite.is_empty()
                {
                    self.set_composition_mode(CompositionMode::PreCompositionOkurigana);
                }

                let current_state = self.current_state_ref();
                let current_composition_mode = &current_state.composition_mode.clone();
                return match current_composition_mode {
                    CompositionMode::Direct => {
                        self.append_converted(&converted);
                        self.set_carry_over(&carry_over);
                        true
                    }
                    CompositionMode::PreComposition => {
                        self.append_converted_to_composite(&converted);
                        self.set_carry_over(&carry_over);
                        true
                    }
                    CompositionMode::PreCompositionOkurigana => {
                        if let Some(key_char) = key_event.get_symbol_char() {
                            self.append_to_composite_iff_no_preconversion(
                                key_char.to_ascii_lowercase(),
                            );
                        } else {
                            debug!("Unreachable. Key event without symbol char is kana converted. Okuri will be ignored on composition.");
                        }
                        if *initial_composition_mode == CompositionMode::PreComposition
                            && !initial_unprocessed_vector.is_empty()
                        {
                            // 以前入力されていた部分はPreComposition側として処理する。
                            // 例: "t T" の't'部分が 'っ' とかな変換される場合
                            self.append_converted_to_composite(&converted)
                        } else {
                            self.append_converted_to_okuri(&converted)
                        }
                        self.set_carry_over(&carry_over);
                        // 入力単独によらない特殊な遷移で、かな変換の結果によって▽モードから▼モードへ移行する。
                        if carry_over.is_empty() {
                            self.set_composition_mode(CompositionMode::CompositionSelection);
                            return self.process_key_event_inner(key_event, true);
                        }
                        true
                    }
                    _ => {
                        debug!("Unreachable by has rom-kana conversion check.");
                        false
                    }
                };
            }
        }

        let handler = self.get_handler(&current_state.input_mode, &current_state.composition_mode);
        // if !handler.can_process(key_event) {
        //     return false;
        // }
        let instructions = handler.get_instruction(key_event, &current_state, is_delegated);
        drop(handler);

        let mut must_delegate = false;
        for instruction in instructions {
            dbg!(&instruction);
            match instruction {
                Instruction::ChangeCompositionMode {
                    composition_mode,
                    delegate,
                } => {
                    if composition_mode == CompositionMode::Register {
                        let previous_mode;
                        if is_delegated {
                            previous_mode = self.current_state_ref().previous_composition_mode;
                        } else {
                            previous_mode = self.current_state_ref().composition_mode;
                        }
                        self.enter_register_mode(previous_mode);
                        must_delegate = delegate;
                    } else {
                        self.set_composition_mode(composition_mode);
                        must_delegate = delegate;
                    }
                }
                Instruction::ChangeInputMode(input_mode) => {
                    self.set_input_mode(input_mode);
                }
                Instruction::OutputNNIfAny(input_mode) => {
                    self.output_nn_if_any(&input_mode, &initial_composition_mode);
                }
                Instruction::FlushPreviousCarryOver => {
                    self.reset_unconverted();
                }
                // Instruction::AbortIfEmptyKanaToConvert => {
                //     if self.current_state().borrow().converted_kana_to_composite.is_empty() {
                //         return true;
                //     }
                // }
                Instruction::SetComposition { kanji } => {
                    self.set_composition_candidate(&kanji);
                }
                Instruction::Abort => {
                    // CompositionSelectionのAbortを想定している。他のAbortでも共通？ 各々instruction変える？
                    self.reset_composited();
                    self.consolidate_converted_to_to_composite();
                    self.abort_register_mode();
                }
                Instruction::ConfirmComposition => {
                    self.confirm_current_composition_candidate();
                }
                Instruction::FinishConsumingKeyEvent => {
                    return true;
                }
                Instruction::FinishNotConsumingKeyEvent => {
                    return false;
                }
                Instruction::ConfirmAsKatakana => {
                    self.confirm_current_kana_to_composite(&InputMode::Katakana);
                    self.set_composition_mode(CompositionMode::Direct);
                }
                Instruction::ConfirmAsHiragana => {
                    self.confirm_current_kana_to_composite(&InputMode::Hiragana);
                    self.set_composition_mode(CompositionMode::Direct);
                }
                Instruction::ConfirmAsJISX0201 => {
                    self.confirm_current_kana_to_composite(&InputMode::HankakuKatakana);
                    self.set_composition_mode(CompositionMode::Direct);
                }
                Instruction::ConfirmDirect => {
                    if self.state_stack.len() > 1 {
                        self.exit_register_mode(&self.current_state_ref().confirmed.to_owned());
                        return true;
                    }
                }
                Instruction::Purge => {
                    self.purge_current_composition_candidate();
                }
                Instruction::NextCandidatePointer => {
                    self.increment_selection_pointer();
                }
                #[allow(unreachable_patterns)]
                _ => {
                    debug!("unimplemented instruction: {}", instruction);
                }
            }
        }
        if must_delegate && is_delegated {
            // Delegated more than twice in commands. Something is wrong.
            // Return in odd state but better than infinte loop.
            // dbg!(instructions);
            return false;
        }
        if must_delegate {
            return self.process_key_event_inner(key_event, true);
        }

        // ここまで来たらかな変換もなく、ステート変更等の命令としての処理が済み、素の入力として処理する状態
        if (modifier - SkkKeyModifier::SHIFT).is_empty() {
            match &self.current_state_ref().composition_mode {
                CompositionMode::CompositionSelection => {
                    debug!("Reached to process as input in composition selection mode. Something is wrong. Ignoring the input.");
                    // Do nothing.
                }
                CompositionMode::Direct
                | CompositionMode::PreComposition
                | CompositionMode::PreCompositionOkurigana => {
                    match &self.current_state_ref().input_mode {
                        InputMode::Ascii => {
                            if key_event.is_ascii_inputtable()
                                && key_event.get_modifier().is_empty()
                            {
                                if let Some(key_char) = key_event.get_symbol_char() {
                                    match &self.current_state_ref().composition_mode {
                                        CompositionMode::Direct => {
                                            self.append_confirmed_raw_char(key_char);
                                        }
                                        _ => {
                                            debug!(
                                                "Unreachable. Ascii should be always in Direct mode."
                                            );
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                        InputMode::Zenkaku => {
                            // TODO: Change input to wide latin
                            if key_event.is_ascii_inputtable()
                                && key_event.get_modifier().is_empty()
                            {
                                if let Some(key_char) = key_event.get_symbol_char() {
                                    match &self.current_state_ref().composition_mode {
                                        CompositionMode::Direct => {
                                            self.append_confirmed_raw_char(key_char);
                                        }
                                        _ => {
                                            debug!(
                                                "Unreachable. Ascii should be always in Direct mode."
                                            );
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                        InputMode::Hiragana | InputMode::Katakana | InputMode::HankakuKatakana => {
                            if self
                                .kana_converter
                                .can_continue(key_event, &initial_unprocessed_vector)
                            {
                                // かな変換できる可能性が残るのでFlushはされない
                                if let Some(key_char) = key_event.get_symbol_char() {
                                    match &self.current_state_ref().composition_mode {
                                        CompositionMode::Direct
                                        | CompositionMode::PreComposition => {
                                            self.append_unconverted(key_char.to_ascii_lowercase())
                                        }
                                        CompositionMode::PreCompositionOkurigana => {
                                            self.append_to_composite_iff_no_preconversion(
                                                key_char.to_ascii_lowercase(),
                                            );
                                            self.append_unconverted(key_char.to_ascii_lowercase());
                                        }
                                        _ => {
                                            debug!("Unreachable.");
                                            return false;
                                        }
                                    }
                                }
                            } else {
                                // "k g" 等かな変換が続けられない場合、resetしてから入力として処理する。
                                // "n d" 等の場合、直前の'n'を'ん'とする。
                                let current_input_mode =
                                    &self.current_state_ref().input_mode.clone();
                                self.output_nn_if_any(
                                    current_input_mode,
                                    &initial_composition_mode,
                                );
                                self.reset_unconverted();
                                let unprocessed_vector =
                                    self.current_state_ref().pre_conversion.clone();
                                if let Some(key_char) = key_event.get_symbol_char() {
                                    // カンマピリオドは特殊な設定と処理がある。
                                    if let Some(converted) =
                                        self.kana_converter.convert_periods(&key_char)
                                    {
                                        match &self.current_state_ref().composition_mode {
                                            CompositionMode::Direct => {
                                                self.append_converted(&converted);
                                            }
                                            CompositionMode::PreComposition
                                            | CompositionMode::PreCompositionOkurigana => {
                                                // 入力単独によらない特殊な遷移で、",."は送り仮名のように扱われて▽モードから▼モードへ移行する。
                                                self.reset_unconverted();
                                                self.append_converted_to_okuri(&converted);
                                                self.set_composition_mode(
                                                    CompositionMode::CompositionSelection,
                                                );
                                                return self
                                                    .process_key_event_inner(key_event, true);
                                            }
                                            _ => {
                                                debug!("Unreachable");
                                                return false;
                                            }
                                        }
                                    } else if self
                                        .kana_converter
                                        .can_continue(key_event, &unprocessed_vector)
                                    {
                                        match &self.current_state_ref().composition_mode {
                                            CompositionMode::Direct
                                            | CompositionMode::PreComposition => self
                                                .append_unconverted(key_char.to_ascii_lowercase()),
                                            CompositionMode::PreCompositionOkurigana => {
                                                if initial_composition_mode
                                                    != PreCompositionOkurigana
                                                {
                                                    self.append_to_composite_iff_no_preconversion(
                                                        key_char.to_ascii_lowercase(),
                                                    );
                                                }
                                                self.append_unconverted(
                                                    key_char.to_ascii_lowercase(),
                                                );
                                            }
                                            _ => {
                                                debug!("Unreachable");
                                                return false;
                                            }
                                        }
                                    } else {
                                        // kana builderで該当がない記号や、表示されないキー
                                        // TODO: 表示がある文字であればAsciiモード扱いで入力する。 '%','!'などが該当
                                        // とりあえず無視する。
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
                // TODO
                CompositionMode::Abbreviation => {}
                CompositionMode::Register => {}
            }
        }
        // TODO: 入力として内部では処理しつつunhandledで返す命令は必要か調べる。ueno/libskkのviとの連携関連issueとか読むとわかるか？
        true
    }

    fn current_state_ref(&self) -> &CskkState {
        self.state_stack.last().expect("State stack is empty!")
    }

    fn current_state(&mut self) -> &mut CskkState {
        self.state_stack.last_mut().expect("State stack is empty!")
    }

    ///
    /// Returns if that key event can be processed by current CSKK
    /// Only checking, doesn't change internal states
    /// TODO: maybe not a proper impl for IM? can be replaced with just checking meta of keyevent?
    ///
    #[allow(dead_code)]
    pub fn will_process(&self, key_event: &KeyEvent) -> bool {
        let current_state = self.current_state_ref();
        let handler = self.get_handler(&current_state.input_mode, &current_state.composition_mode);
        handler.can_process(key_event)
    }

    fn get_handler<'a>(
        &'a self,
        _input_mode: &InputMode,
        composition_mode: &CompositionMode,
    ) -> Box<dyn CommandHandler + 'a> {
        // FIXME: this _ => default handler looks error prone
        match composition_mode {
            CompositionMode::Direct => Box::new(&self.direct_handler),
            CompositionMode::PreComposition => Box::new(&self.kana_precomposition_handler),
            CompositionMode::PreCompositionOkurigana => Box::new(&self.kana_precomposition_handler),
            CompositionMode::CompositionSelection => Box::new(&self.kana_composition_handler),
            _ => Box::new(&self.direct_handler),
        }
    }

    /// Mainly for test purpose, but exposed to test as library.
    fn process_key_events_string(&mut self, key_event_string: &str) -> bool {
        self.process_key_events(&KeyEvent::deserialize_seq(key_event_string).unwrap())
    }

    /// Mainly for test purpose, but exposed to test as library.
    /// FIXME: Remove this clippy rule allow when parameterize on array length is stable in Rust. maybe 1.51?
    #[allow(clippy::ptr_arg)]
    fn process_key_events(&mut self, key_event_seq: &KeyEventSeq) -> bool {
        dbg!(key_event_seq);
        for key_event in key_event_seq {
            let processed = self.process_key_event(key_event);
            if !processed {
                dbg!("Key event not processed", key_event);
            }
            dbg!(&self.state_stack);
            //dbg!(self.current_state_ref());
        }
        true
    }

    pub fn new(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        dictionaries: Vec<CskkDictionary>,
    ) -> Self {
        let kana_direct_handler = DirectModeCommandHandler::new();
        let kana_precomposition_handler = KanaPrecompositionHandler::new();
        let kana_composition_handler = KanaCompositionHandler::new(dictionaries);
        let kana_converter = Box::new(KanaBuilder::default_converter());

        let initial_stack = vec![CskkState::new(input_mode, composition_mode)];
        Self {
            state_stack: initial_stack,
            direct_handler: kana_direct_handler,
            kana_precomposition_handler,
            kana_composition_handler,
            kana_converter,
            kana_form_changer: KanaFormChanger::default_kanaform_changer(),
        }
    }
}

impl Display for Instruction {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            //            Instruction::Abort => {
            //                writeln!(f, "Abort")
            //            }
            Instruction::ChangeCompositionMode {
                composition_mode,
                delegate,
            } => writeln!(
                f,
                "ChangeComopositionMode: {:?} (delegate: {})",
                composition_mode, delegate
            ),
            _ => writeln!(f, "Display-unsupported instruction. This is a TODO."),
        }
    }
}

impl Display for CskkState {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            r#"{{
            {:?}
            {:?}
            confirmed: {}"#,
            self.input_mode, self.composition_mode, self.confirmed
        );
        write!(f, "            unconverted:");
        for c in self.pre_conversion.to_vec() {
            write!(f, "{}", c);
        }
        writeln!(f);
        writeln!(f, "}}");
        Ok(())
    }
}

#[cfg(test)]
impl CskkState {
    pub fn new_test_state(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        pre_conversion: Vec<char>,
    ) -> Self {
        CskkState {
            input_mode,
            composition_mode,
            previous_composition_mode: composition_mode,
            pre_conversion,
            raw_to_composite: "".to_string(),
            converted_kana_to_composite: "".to_string(), // TODO
            converted_kana_to_okuri: "".to_string(),     // TODO
            composited_kanji: "".to_string(),            // TODO
            composited_okuri: "".to_string(),
            confirmed: "".to_string(), // TODO
            selection_pointer: 0,
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use std::sync::Once;

    use super::*;

    pub static INIT_SYNC: Once = Once::new();

    // TODO: setup proper debug log and move to lib test
    #[allow(dead_code)]
    fn init() {
        INIT_SYNC.call_once(|| {
            let _ = env_logger::init();
        });
    }

    fn new_test_context(input_mode: InputMode, composition_mode: CompositionMode) -> CskkContext {
        let dict = skk_file_dict_new_rs("tests/data/SKK-JISYO.S", "euc-jp");
        let mut context = skk_context_new_rs(vec![dict]);
        context.set_input_mode(input_mode);
        context.set_composition_mode(composition_mode);
        context
    }

    #[test]
    fn will_process() {
        let cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);
        let a = KeyEvent::from_str("a").unwrap();
        assert!(cskkcontext.will_process(&a));
        let copy = KeyEvent::from_str("C-c").unwrap();
        assert!(!cskkcontext.will_process(&copy));
    }

    #[test]
    fn process_key_event() {
        let mut cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);

        let a = KeyEvent::from_str("a").unwrap();
        let result = cskkcontext.process_key_event(&a);
        assert!(result);
    }

    #[test]
    fn retrieve_output() {
        let mut cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);
        let a = KeyEvent::from_str("a").unwrap();
        cskkcontext.process_key_event(&a);
        let actual = cskkcontext.retrieve_output(false).unwrap();
        assert_eq!("a", actual);
        let actual = cskkcontext.retrieve_output(true).unwrap();
        assert_eq!("a", actual);
        let after = cskkcontext.retrieve_output(true);
        assert_eq!(None, after);
    }

    #[test]
    fn poll_output() {
        let mut cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);
        let a = KeyEvent::from_str("a").unwrap();
        cskkcontext.process_key_event(&a);
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("a", actual);
        let after = cskkcontext.poll_output();
        assert_eq!(None, after);
    }

    #[test]
    fn get_preedit() {
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        let capital_a = KeyEvent::from_str("A").unwrap();
        cskkcontext.process_key_event(&capital_a);
        let actual = cskkcontext.get_preedit().expect(&format!(
            "No preedit. context: {:?}",
            cskkcontext.current_state_ref()
        ));
        assert_eq!("▽あ", actual);
    }

    #[test]
    fn get_preedit_register_mode() {
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext.append_converted_to_composite("ほげ");
        cskkcontext.enter_register_mode(CompositionMode::Direct);
        cskkcontext.append_converted("あか");
        cskkcontext.append_unconverted('s');
        let actual = cskkcontext.get_preedit().expect(&format!(
            "No preedit. context: {:?}",
            cskkcontext.current_state_ref()
        ));
        assert_eq!("▼ほげ【あかs】", actual);
    }
}
