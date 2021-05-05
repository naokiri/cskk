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

use crate::candidate_list::CandidateList;
use crate::command_handler::direct_mode_command_handler::DirectModeCommandHandler;
use crate::command_handler::kana_composition_handler::KanaCompositionHandler;
use crate::command_handler::kana_precomposition_handler::KanaPrecompositionHandler;
use crate::command_handler::CommandHandler;
use crate::dictionary::candidate::Candidate;
use crate::dictionary::file_dictionary::FileDictionary;
use crate::dictionary::static_dict::StaticFileDict;
use crate::dictionary::user_dictionary::UserDictionary;
use crate::dictionary::{
    confirm_candidate, get_all_candidates, purge_candidate, CskkDictionary, CskkDictionaryType,
    Dictionary,
};
use crate::kana_builder::KanaBuilder;
use crate::kana_form_changer::KanaFormChanger;
use crate::keyevent::KeyEventSeq;
use crate::keyevent::{CskkKeyEvent, SkkKeyModifier};
use crate::skk_modes::CompositionMode::PreCompositionOkurigana;
use crate::skk_modes::InputMode;
use crate::skk_modes::{has_rom2kana_conversion, CompositionMode};
use log::debug;
use log::warn;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::iter::FromIterator;
use std::sync::Arc;
use xkbcommon::xkb;

mod candidate_list;
#[cfg(feature = "capi")]
pub mod capi;
mod command_handler;
pub mod dictionary;
mod env;
pub mod error;
mod kana_builder;
mod kana_form_changer;
mod keyevent;
pub mod skk_modes;
#[cfg(test)]
mod testhelper;

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
    // 現在の変換候補リストを作りなおす
    UpdateCandidateList,
    // 変換候補ポインタを進める
    NextCandidatePointer,
    // 変換候補ポインタを戻す
    PreviousCandidatePointer,
    // 現在の変換候補で確定する
    ConfirmComposition,
    ConfirmAsHiragana,
    ConfirmAsKatakana,
    #[allow(clippy::upper_case_acronyms)]
    ConfirmAsJISX0201,
    // Direct時に確定する。辞書編集時は動作があるのでEnterをイベント消費するが、そうでない場合はcskkでイベントを消費しない。
    ConfirmDirect,
    // 現在の候補を辞書から消す
    Purge,
    // PreComposition時に一文字消去する。
    // ueno/libskk StartStateHandler のdelete時？
    DeletePrecomposition,
    // Direct時に一文字消去する。消去可能時のみキー入力処理を終わる。
    // ueno/libskk NoneStateHandler のdelete時？
    DeleteDirect,
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
    dictionaries: Vec<Arc<CskkDictionary>>,
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
    // 現在の変換候補リスト
    candidate_list: CandidateList,
    // 入力を漢字変換した現在の選択肢の送り仮名部分。 TODO: 保持せずにconverted_kana_to_okuriで良い？
    composited_okuri: String,
    // 確定済み入力列。pollされた時に渡してflushされるもの。
    confirmed: String,
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
            composited_okuri: "".to_string(),
            confirmed: "".to_string(),
            candidate_list: CandidateList::new(),
        }
    }

    fn preedit_string(&self) -> String {
        let converted = &self.confirmed;
        let unconverted = &self.pre_conversion;
        let kana_to_composite = &self.converted_kana_to_composite;
        let kana_to_okuri = &self.converted_kana_to_okuri;
        let current_candidate = self.candidate_list.get_current_candidate();
        let fallback_candidate = Candidate::default();
        let composited = &current_candidate.unwrap_or(&fallback_candidate).kouho_text;
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

pub fn skk_file_dict_new_rs(path_string: &str, encoding: &str) -> CskkDictionary {
    CskkDictionary::new(CskkDictionaryType::StaticFile(StaticFileDict::new(
        path_string,
        encoding,
    )))
}

pub fn skk_user_dict_new_rs(path_string: &str, encoding: &str) -> CskkDictionary {
    CskkDictionary::new(CskkDictionaryType::UserFile(UserDictionary::new(
        path_string,
        encoding,
    )))
}

pub fn skk_context_new_rs(dictionaries: Vec<Arc<CskkDictionary>>) -> CskkContext {
    CskkContext::new(InputMode::Hiragana, CompositionMode::Direct, dictionaries)
}

/// Test purpose
pub fn skk_context_process_key_events_rs(context: &mut CskkContext, keyevents: &str) -> bool {
    context.process_key_events_string(keyevents)
}

/// 現在のoutputをpollingする。
///
pub fn skk_context_poll_output_rs(context: &mut CskkContext) -> String {
    if let Some(str) = context.poll_output() {
        return str;
    }
    "".to_string()
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

pub fn skk_context_get_input_mode_rs(context: &CskkContext) -> InputMode {
    context.current_state_ref().input_mode
}

pub fn skk_context_reset_rs(context: &mut CskkContext) {
    context.poll_output();
    context.reset_state_stack();
    context.set_composition_mode(CompositionMode::Direct);
}

/// テスト用途
/// 他のステートとの整合性は無視される。
pub fn skk_context_set_composition_mode(
    context: &mut CskkContext,
    composition_mode: CompositionMode,
) {
    context.set_composition_mode(composition_mode)
}

/// 他のステートとの整合性は無視される。
pub fn skk_context_set_input_mode_rs(context: &mut CskkContext, input_mode: InputMode) {
    context.set_input_mode(input_mode)
}

pub fn skk_context_save_dictionaries_rs(context: &mut CskkContext) {
    context.save_dictionary();
}

///
/// reload current dictionaries
/// For integration test purpose.
///
pub fn skk_context_reload_dictionary(context: &mut CskkContext) {
    context.reload_dictionary();
}

pub fn skk_context_set_dictionaries_rs(
    context: &mut CskkContext,
    dictionaries: Vec<Arc<CskkDictionary>>,
) {
    context.set_dictionaries(dictionaries);
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
    /// TODO: 常に返るので、Optionである必要がなかった。caller側できちんとOption扱いするか、返り値の型を変えるか。
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
    /// UTF-8 character range of text to emphasize in preedit.
    ///
    /// Currently we don't have expand/shrink-preedit feature, thus we have no text we want to emphasize.
    ///
    pub fn get_preedit_underline(&self) -> (isize, isize) {
        (0, 0)
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

    fn delete_precomposition(&mut self) {
        let mut current_state = self.current_state();
        if !current_state.pre_conversion.is_empty() {
            current_state.pre_conversion.pop();
            current_state.raw_to_composite.pop();
            if current_state.pre_conversion.is_empty()
                && current_state.composition_mode == CompositionMode::PreCompositionOkurigana
            {
                current_state.composition_mode = CompositionMode::PreComposition;
            }
        } else if !current_state.converted_kana_to_okuri.is_empty() {
            current_state.converted_kana_to_okuri.pop();
            current_state.raw_to_composite.pop();
            if current_state.converted_kana_to_okuri.is_empty()
                && current_state.composition_mode == CompositionMode::PreCompositionOkurigana
            {
                current_state.composition_mode = CompositionMode::PreComposition;
            }
        } else if !current_state.converted_kana_to_composite.is_empty() {
            current_state.converted_kana_to_composite.pop();
            current_state.raw_to_composite.pop();
        } else {
            current_state.composition_mode = CompositionMode::Direct;
        }
    }

    fn delete_direct(&mut self) -> bool {
        let current_state = self.current_state();
        if !current_state.pre_conversion.is_empty() {
            current_state.pre_conversion.pop();
            return true;
        } else if !current_state.confirmed.is_empty() {
            current_state.confirmed.pop();
            return true;
        }
        false
    }

    fn reset_unconverted(&mut self) {
        let current_state = self.current_state();
        current_state.pre_conversion.clear();
    }

    fn set_carry_over(&mut self, unconv: &[char]) {
        let current_state = self.current_state();
        current_state.pre_conversion = unconv.to_owned();
    }

    #[allow(unused_must_use)]
    fn next_candidate(&mut self) {
        let current_state = self.current_state();
        current_state.candidate_list.forward_candidate();
    }

    #[allow(unused_must_use)]
    fn previous_candidate(&mut self) {
        let current_state = self.current_state();
        current_state.candidate_list.backward_candidate();
    }

    fn set_new_candidate(&mut self, new_candidate_kanji: &str) {
        let current_state = self.current_state();
        let okuri = current_state.converted_kana_to_okuri.to_owned();
        current_state.candidate_list.set_new_candidate(new_candidate_kanji, !okuri.is_empty());
        current_state.composited_okuri = okuri;
    }

    fn update_candidate_list(&mut self) {
        let raw_to_composite = self.current_state_ref().raw_to_composite.clone();
        let okuri = self.current_state_ref().converted_kana_to_okuri.to_owned();
        let candidates = get_all_candidates(&self.dictionaries, &raw_to_composite);
        self.current_state()
            .candidate_list
            .set(raw_to_composite, candidates);
        self.current_state().composited_okuri = okuri;
    }

    #[allow(unused_must_use)]
    fn purge_current_composition_candidate(&mut self) {
        let current_candidate = self
            .current_state_ref()
            .candidate_list
            .get_current_candidate()
            .unwrap()
            .clone();
        for cskkdict in self.dictionaries.iter_mut() {
            purge_candidate(cskkdict, &current_candidate);
        }

        self.reset_current_state();
    }

    #[allow(unused_must_use)]
    fn confirm_current_composition_candidate(&mut self) {
        let current_candidate = self
            .current_state_ref()
            .candidate_list
            .get_current_candidate()
            .unwrap()
            .clone();
        for cskkdict in self.dictionaries.iter_mut() {
            confirm_candidate(cskkdict, &current_candidate);
        }

        let composited_okuri = &self.current_state_ref().converted_kana_to_okuri;
        let composited_kanji_and_okuri =
            current_candidate.kouho_text.to_string() + composited_okuri;
        let current_state = self.current_state();
        current_state
            .confirmed
            .push_str(&composited_kanji_and_okuri);
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
        current_state.composited_okuri.clear();
        current_state.candidate_list.reset();
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

    fn exit_register_mode(&mut self, confirmed: &str) {
        if self.state_stack.len() > 1 {
            self.state_stack.pop();
            if confirmed.is_empty() {
                self.current_state().composition_mode =
                    self.current_state_ref().previous_composition_mode;
            } else {
                let current_state = self.current_state();
                current_state.composition_mode = CompositionMode::Direct;
                self.set_new_candidate(confirmed);
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
    pub fn process_key_event(&mut self, key_event: &CskkKeyEvent) -> bool {
        let modifier = key_event.get_modifier();
        if modifier.contains(SkkKeyModifier::RELEASE) {
            // TODO: from ueno/libskk's comment, returning false for all release might need to be reconsidered on dictionary editing.
            return false;
        }
        self.process_key_event_inner(key_event, false)
    }

    pub fn save_dictionary(&mut self) {
        for cskkdict in &self.dictionaries {
            if let Ok(lock) = cskkdict.lock() {
                let result = match &*lock {
                    CskkDictionaryType::StaticFile(dictionary) => dictionary.save_dictionary(),
                    CskkDictionaryType::UserFile(dictionary) => dictionary.save_dictionary(),
                    CskkDictionaryType::EmptyDict(dictionary) => dictionary.save_dictionary(),
                };
                match result {
                    Ok(_) => {}
                    Err(error) => {
                        warn!("{}", &error.to_string());
                    }
                }
            }
        }
    }

    pub fn reload_dictionary(&mut self) {
        for cskkdict in &self.dictionaries {
            if let Ok(mut lock) = cskkdict.lock() {
                let result = match *lock {
                    CskkDictionaryType::StaticFile(ref mut dictionary) => dictionary.reload(),
                    CskkDictionaryType::UserFile(ref mut dictionary) => dictionary.reload(),
                    CskkDictionaryType::EmptyDict(_) => Ok(()),
                };
                match result {
                    Ok(_) => {}
                    Err(error) => {
                        warn!("{}", &error.to_string());
                    }
                }
            }
        }
    }

    pub fn set_dictionaries(&mut self, dicts: Vec<Arc<CskkDictionary>>) {
        self.kana_composition_handler
            .set_dictionaries(dicts.clone());
        self.dictionaries = dicts;
    }

    // FIXME: まだ良いルールが把握できていない中でインクリメンタルに機能を追加しているのでぐちゃぐちゃ。一通り機能ができてバグ修正できたらリファクタリング
    ///
    /// 上から順に
    /// rom2kana可能? -yes-> かな入力として処理 (大文字のみcompositionmode変更コマンドとしても処理)
    /// 現在のCompositionMode内で解釈されるコマンド？ -yes-> compositionmode用コマンドとして処理
    /// rom2kana継続可能 or ascii？ -yes-> 継続入力として処理
    /// rom2kana継続不可能 -all-> Flush後に入力として処理
    ///
    fn process_key_event_inner(&mut self, key_event: &CskkKeyEvent, is_delegated: bool) -> bool {
        debug!("Keyevent: {:?}", key_event);
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
            debug!("{:?}", &instruction);
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
                Instruction::UpdateCandidateList => {
                    self.update_candidate_list();
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
                    return if self.state_stack.len() > 1 {
                        self.exit_register_mode(&self.current_state_ref().confirmed.to_owned());
                        true
                    } else {
                        false
                    }
                }
                Instruction::Purge => {
                    self.purge_current_composition_candidate();
                }
                Instruction::NextCandidatePointer => {
                    self.next_candidate();
                }
                Instruction::PreviousCandidatePointer => {
                    self.previous_candidate();
                }
                Instruction::DeletePrecomposition => {
                    self.delete_precomposition();
                }
                Instruction::DeleteDirect => {
                    return self.delete_direct();
                }
                #[allow(unreachable_patterns)]
                _ => {
                    debug!("unimplemented instruction: {}", &instruction);
                }
            }
        }
        if must_delegate && is_delegated {
            // Delegated more than twice in commands. Something is wrong.
            // Return in odd state but better than infinte loop.
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
                                && (key_event.get_modifier() - SkkKeyModifier::SHIFT).is_empty()
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
                                    return true;
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
                                    return true;
                                }
                            }
                        }
                        InputMode::Hiragana | InputMode::Katakana | InputMode::HankakuKatakana => {
                            if key_event.is_ascii_inputtable()
                                && (key_event.get_modifier() - SkkKeyModifier::SHIFT).is_empty()
                            {
                                if self
                                    .kana_converter
                                    .can_continue(key_event, &initial_unprocessed_vector)
                                {
                                    // かな変換できる可能性が残るのでFlushはされない
                                    if let Some(key_char) = key_event.get_symbol_char() {
                                        match &self.current_state_ref().composition_mode {
                                            CompositionMode::Direct
                                            | CompositionMode::PreComposition => self
                                                .append_unconverted(key_char.to_ascii_lowercase()),
                                            CompositionMode::PreCompositionOkurigana => {
                                                self.append_to_composite_iff_no_preconversion(
                                                    key_char.to_ascii_lowercase(),
                                                );
                                                self.append_unconverted(
                                                    key_char.to_ascii_lowercase(),
                                                );
                                            }
                                            _ => {
                                                debug!("Unreachable.");
                                                return false;
                                            }
                                        }
                                        return true;
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
                                            // TODO: 上のflush前と同じ？ 本当に同じならば適当なメソッドで共通化する。
                                            match &self.current_state_ref().composition_mode {
                                                CompositionMode::Direct => {
                                                    self.append_converted(&converted);
                                                }
                                                CompositionMode::PreComposition
                                                | CompositionMode::PreCompositionOkurigana => {
                                                    // 入力単独によらない特殊な遷移で、",."は送り仮名のように扱われて▽モードから▼モードへ移行する？
                                                    // TODO: ,.以外を知らなかったので直書きしてしまったが、Auto-start-henkanは別でまとめて行うべきだった。
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
                                            // TODO: 上のflush前と同じ？ 本当に同じならば適当なメソッドで共通化する。
                                            match &self.current_state_ref().composition_mode {
                                                CompositionMode::Direct
                                                | CompositionMode::PreComposition => self
                                                    .append_unconverted(
                                                        key_char.to_ascii_lowercase(),
                                                    ),
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
                                            return true;
                                        } else if key_event.get_symbol() == xkb::keysyms::KEY_q
                                            || key_event.get_symbol() == xkb::keysyms::KEY_Q
                                        {
                                            // kana builder で該当がないが記号ではない文字。
                                            // コマンドとして処理するとddskkの実装から離れることでlibskkとも動作を合わせるのが難しく、ここで直接指定。
                                            // TODO: kanaとして続けられないかもしれないが、記号ではない文字であるかどうかの判定をどこかに統合。[a-zA-Z]判定だけで良い？
                                            // do nothing.
                                            return true;
                                        } else {
                                            // kana builderで該当がない記号等
                                            match &self.current_state_ref().composition_mode {
                                                CompositionMode::Direct => {
                                                    self.append_confirmed_raw_char(key_char);
                                                }
                                                CompositionMode::PreComposition => {
                                                    self.append_converted_to_composite(
                                                        &key_char.to_string(),
                                                    );
                                                }
                                                CompositionMode::PreCompositionOkurigana => {
                                                    self.append_converted_to_okuri(
                                                        &key_char.to_string(),
                                                    );
                                                }

                                                _ => {
                                                    debug!("Unreachable");
                                                    return false;
                                                }
                                            }
                                        }
                                        return true;
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
        false
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
    pub fn will_process(&self, key_event: &CskkKeyEvent) -> bool {
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
        self.process_key_events(&CskkKeyEvent::deserialize_seq(key_event_string).unwrap())
    }

    /// Mainly for test purpose, but exposed to test as library.
    /// FIXME: Remove this clippy rule allow when parameterize on array length is stable in Rust. maybe 1.51?
    #[allow(clippy::ptr_arg)]
    fn process_key_events(&mut self, key_event_seq: &KeyEventSeq) -> bool {
        for key_event in key_event_seq {
            let processed = self.process_key_event(key_event);
            if !processed {
                debug!("Key event not processed: {:?}", key_event);
            }
            debug!("{:?}", &self.state_stack);
            //dbg!(self.current_state_ref());
        }
        true
    }

    pub fn new(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        dictionaries: Vec<Arc<CskkDictionary>>,
    ) -> Self {
        let kana_direct_handler = DirectModeCommandHandler::new();
        let kana_precomposition_handler = KanaPrecompositionHandler::new();
        let kana_composition_handler = KanaCompositionHandler::new(dictionaries.clone());
        let kana_converter = Box::new(KanaBuilder::default_converter());

        let initial_stack = vec![CskkState::new(input_mode, composition_mode)];
        Self {
            state_stack: initial_stack,
            direct_handler: kana_direct_handler,
            kana_precomposition_handler,
            kana_composition_handler,
            kana_converter,
            kana_form_changer: KanaFormChanger::default_kanaform_changer(),
            dictionaries,
        }
    }

    /// For e2e test purpose. Use new() instead.
    pub fn new_from_shared_files(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        dictionaries: Vec<Arc<CskkDictionary>>,
        kana_converter_filepath: &str,
        kana_form_changer_filepath: &str,
    ) -> Self {
        let kana_direct_handler = DirectModeCommandHandler::new();
        let kana_precomposition_handler = KanaPrecompositionHandler::new();
        let kana_composition_handler = KanaCompositionHandler::new(dictionaries.clone());
        let kana_converter = Box::new(KanaBuilder::converter_from_file(kana_converter_filepath));

        let initial_stack = vec![CskkState::new(input_mode, composition_mode)];
        Self {
            state_stack: initial_stack,
            direct_handler: kana_direct_handler,
            kana_precomposition_handler,
            kana_composition_handler,
            kana_converter,
            kana_form_changer: KanaFormChanger::from_file(kana_form_changer_filepath),
            dictionaries,
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
            converted_kana_to_composite: "".to_string(),
            converted_kana_to_okuri: "".to_string(),
            composited_okuri: "".to_string(),
            confirmed: "".to_string(),
            candidate_list: CandidateList::new(),
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::testhelper::init_test_logger;

    fn new_test_context(input_mode: InputMode, composition_mode: CompositionMode) -> CskkContext {
        let dict = Arc::new(skk_file_dict_new_rs("tests/data/SKK-JISYO.S", "euc-jp"));
        let dictionaries = vec![dict];
        let kana_direct_handler = DirectModeCommandHandler::new();
        let kana_precomposition_handler = KanaPrecompositionHandler::new();
        let kana_composition_handler = KanaCompositionHandler::new(dictionaries.clone());
        let kana_converter = Box::new(KanaBuilder::test_converter());

        let initial_stack = vec![CskkState::new(input_mode, composition_mode)];
        CskkContext {
            state_stack: initial_stack,
            direct_handler: kana_direct_handler,
            kana_precomposition_handler,
            kana_composition_handler,
            kana_converter,
            kana_form_changer: KanaFormChanger::test_kana_form_changer(),
            dictionaries,
        }
    }

    #[test]
    fn will_process() {
        let cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);
        let a = CskkKeyEvent::from_str("a").unwrap();
        assert!(cskkcontext.will_process(&a));
        let copy = CskkKeyEvent::from_str("C-c").unwrap();
        assert!(!cskkcontext.will_process(&copy));
    }

    #[test]
    fn process_key_event() {
        let mut cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);

        let a = CskkKeyEvent::from_str("a").unwrap();
        let result = cskkcontext.process_key_event(&a);
        assert!(result);
    }

    #[test]
    fn retrieve_output() {
        let mut cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);
        let a = CskkKeyEvent::from_str("a").unwrap();
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
        let a = CskkKeyEvent::from_str("a").unwrap();
        cskkcontext.process_key_event(&a);
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("a", actual);
        let after = cskkcontext.poll_output();
        assert_eq!(None, after);
    }

    #[test]
    fn get_preedit() {
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        let capital_a = CskkKeyEvent::from_str("A").unwrap();
        cskkcontext.process_key_event(&capital_a);
        let actual = cskkcontext.get_preedit().unwrap_or_else(|| {
            panic!("No preedit. context: {:?}", cskkcontext.current_state_ref())
        });
        assert_eq!("▽あ", actual);
    }

    #[test]
    fn get_preedit_register_mode() {
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext.append_converted_to_composite("ほげ");
        cskkcontext.enter_register_mode(CompositionMode::Direct);
        cskkcontext.append_converted("あか");
        cskkcontext.append_unconverted('s');
        let actual = cskkcontext.get_preedit().unwrap_or_else(|| {
            panic!("No preedit. context: {:?}", cskkcontext.current_state_ref())
        });
        assert_eq!("▼ほげ【あかs】", actual);
    }

    #[test]
    fn process_backspace() {
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext.process_key_event(&CskkKeyEvent::from_str("h").unwrap());
        let actual = cskkcontext.process_key_event(&CskkKeyEvent::from_str("BackSpace").unwrap());
        assert!(actual);
    }

    #[test]
    fn process_period() {
        init_test_logger();
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext.process_key_event(&CskkKeyEvent::from_str("h").unwrap());
        let actual = cskkcontext.process_key_event(&CskkKeyEvent::from_str("period").unwrap());
        assert!(actual);
    }

    #[test]
    fn dont_process_return_in_direct() {
        init_test_logger();
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        let actual = cskkcontext.process_key_event(&CskkKeyEvent::from_str("Return").unwrap());
        assert!(!actual);
    }
}
