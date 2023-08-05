#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate sequence_trie;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate nom;
extern crate xkbcommon;

use crate::command_handler::ConfigurableCommandHandler;
use crate::command_handler::Instruction;
use crate::config::CskkConfig;
use crate::cskkstate::{CskkState, CskkStateInfo};
use crate::dictionary::{
    confirm_candidate, get_all_candidates, numeric_entry_count, numeric_string_count,
    purge_candidate, replace_numeric_string, to_composite_to_numeric_dict_key, CskkDictionary,
    CskkDictionaryType, Dictionary,
};
use crate::dictionary::{get_all_complete, Candidate};
use crate::error::CskkError;
use crate::kana_builder::KanaBuilder;
use crate::keyevent::KeyEventSeq;
use crate::keyevent::{CskkKeyEvent, SkkKeyModifier};
use crate::rule::{CskkRule, CskkRuleMetadata, CskkRuleMetadataEntry};
use crate::skk_modes::{has_rom2kana_conversion, CompositionMode};
use crate::skk_modes::{CommaStyle, InputMode, PeriodStyle};
use form_changer::{AsciiFormChanger, KanaFormChanger};
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::sync::Arc;
use xkbcommon::xkb::Keysym;

mod candidate_list;
#[cfg(feature = "capi")]
pub mod capi;
mod command_handler;
mod config;
pub mod cskkstate;
pub mod dictionary;
mod env;
pub mod error;
mod form_changer;
mod kana_builder;
pub mod keyevent;
mod rule;
pub mod skk_modes;
#[cfg(test)]
mod testhelper;

// 実用上5でも問題なかった。安全率2で。
const REGISTER_MODE_STACK_MAX: usize = 10;

///
/// CSKK のメインの構造体。このcontextをに対しキー入力を行って変換された状態を得る。
/// 元々C向けAPIを作ることを想定していた多くの関数が*_rsとして公開されている。
///
/// FIXME: Rustのインタフェースとしてあまり直感的ではないので*_rsのものはCSKKContextの関数にする。
///
/// TODO: Rustのstructまわりの一部分mutに変更があったら非mutでstateアクセスしているところを直す
///
pub struct CskkContext {
    state_stack: Vec<CskkState>,
    command_handler: ConfigurableCommandHandler,
    kana_converter: Box<KanaBuilder>,
    kana_form_changer: KanaFormChanger,
    ascii_form_changer: AsciiFormChanger,
    dictionaries: Vec<Arc<CskkDictionary>>,
    config: CskkConfig,
    //rule: CskkRuleMetadataEntry,
}

/// Test purpose only.
#[deprecated(
    since = "2.0.0",
    note = "Not proper for Rust interface. See rustdoc for details."
)]
pub fn skk_context_process_key_events_rs(context: &mut CskkContext, keyevents: &str) -> bool {
    context.process_key_events_string(keyevents)
}

///
/// Testing purpose? Use `CskkContext.poll_output()` instead. this interface might be deleted at any update.
/// 現在のoutputをpollingする。
///
#[deprecated(
    since = "2.0.0",
    note = "Not proper for Rust interface. See rustdoc for details."
)]
pub fn skk_context_poll_output_rs(context: &mut CskkContext) -> String {
    if let Some(str) = context.poll_output() {
        return str;
    }
    "".to_string()
}

/// テスト用途？。preedit文字列と同じ内容の文字列を取得する。
/// This interface might be deleted at any update. Use `CskkContext.get_preedit()` instead.
///
#[deprecated(
    since = "2.0.0",
    note = "Not proper for Rust interface. See rustdoc for details."
)]
pub fn skk_context_get_preedit_rs(context: &CskkContext) -> String {
    context.get_preedit().unwrap()
}

///
/// 元capiテスト用途だが、libで公開してしまったうえに代理メソッドがないので当面残す。
/// 将来的にはcontextのmethodに置き換える。
///
#[deprecated(since = "3.1.0", note = "Use CskkContext.get_current_input_mode()")]
pub fn skk_context_get_input_mode_rs(context: &CskkContext) -> InputMode {
    context.current_state_ref().input_mode
}

///
/// 元capiテスト用途だが、libで公開してしまったうえに代理メソッドがないので当面残す。
/// 将来的にはcontextのmethodに置き換える。
///
#[deprecated(
    since = "3.0.1",
    note = "Use CskkContext.get_current_composition_mode()"
)]
pub fn skk_context_get_composition_mode_rs(context: &CskkContext) -> CompositionMode {
    context.current_state_ref().composition_mode
}

///
/// 元capiテスト用途だが、libで公開してしまったうえに代理メソッドがないので当面残す。
/// 将来的にはcontextのmethodに置き換える。
///
pub fn skk_context_reset_rs(context: &mut CskkContext) {
    context.reset_state_stack();
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

///
/// 元capiテスト用途。ABI変更を明示しないバージョンアップデートにより公開を止めうる。
/// Rust libとしてはsave_dictionaryを使用する。
///
#[deprecated(
    since = "2.0.0",
    note = "Not proper for Rust interface. See rustdoc for details."
)]
pub fn skk_context_save_dictionaries_rs(context: &mut CskkContext) {
    context.save_dictionary();
}

///
/// reload current dictionaries
/// For integration test purpose.ABI変更を明示しないバージョンアップデートにより公開を止めうる。
/// Rust libとしてはreload_dictionaryを代わりに用いる
///
#[deprecated(
    since = "2.0.0",
    note = "Not proper for Rust interface. See rustdoc for details."
)]
pub fn skk_context_reload_dictionary(context: &mut CskkContext) {
    context.reload_dictionary();
}

///
/// 元capiテスト用途。ABI変更を明示しないバージョンアップデートにより公開を止めうる。
/// Rust libとしてはset_dictionariesを使用する。
///
#[deprecated(
    since = "2.0.0",
    note = "Not proper for Rust interface. See rustdoc for details."
)]
pub fn skk_context_set_dictionaries_rs(
    context: &mut CskkContext,
    dictionaries: Vec<Arc<CskkDictionary>>,
) {
    context.set_dictionaries(dictionaries);
}

///
/// 元capiテスト用途。ABI変更を明示しないバージョンアップデートにより公開を止めうる。
/// 内部状態なので、Rust libが使用することを想定しない。
///
#[deprecated(
    since = "2.0.0",
    note = "Not proper for Rust interface. See rustdoc for details."
)]
pub fn skk_context_get_current_to_composite_rs(context: &CskkContext) -> String {
    context
        .current_state_ref()
        .get_composite_key()
        .get_to_composite()
        .to_string()
}

///
/// 元capiテスト用途だが、libで公開してしまったうえに代理メソッドがないので当面残す。
/// 将来的にはcontextのmethodに置き換える。
///
// TODO: Rust interface としてはcontextのメソッドにする。
// #[deprecated(
//     since = "2.0.0",
//     note = "Not proper for Rust interface. See rustdoc for details."
// )]
pub fn skk_context_get_current_candidate_count_rs(context: &CskkContext) -> usize {
    context.current_state_ref().get_candidate_list().len()
}

///
/// 現在の候補リストを返す。
///
// TODO: Rust interface としてはcontextのメソッドにする。
// #[deprecated(
//     since = "2.0.0",
//     note = "Not proper for Rust interface. See rustdoc for details."
// )]
pub fn skk_context_get_current_candidates_rs(context: &CskkContext) -> &Vec<Candidate> {
    context
        .current_state_ref()
        .get_candidate_list()
        .get_all_candidates()
}

///
/// Get candidate cursor position.
///
// TODO: Rust interface としてはcontextのメソッドにする。
// #[deprecated(
//     since = "2.0.0",
//     note = "Not proper for Rust interface. See rustdoc for details."
// )]
pub fn skk_context_get_current_candidate_cursor_position_rs(
    context: &CskkContext,
) -> Result<usize, CskkError> {
    if context.current_state_ref().get_candidate_list().is_empty() {
        Err(CskkError::Error(
            "Likely not in candidate selection".to_string(),
        ))
    } else {
        Ok(context
            .current_state_ref()
            .get_candidate_list()
            .get_selection_pointer())
    }
}

///
/// Set candidate cursor position.
///
// TODO: Rust interface としてはcontextのメソッドにする。
// #[deprecated(
//     since = "2.0.0",
//     note = "Not proper for Rust interface. See rustdoc for details."
// )]
pub fn skk_context_select_candidate_at_rs(context: &mut CskkContext, i: i32) -> bool {
    let len = context
        .current_state_ref()
        .get_candidate_list()
        .get_all_candidates()
        .len();
    if len == 0 {
        return false;
    }

    if i < 0 {
        context
            .current_state()
            .consolidate_converted_to_to_composite();
        context.current_state().clear_candidate_list();
        context.set_composition_mode(CompositionMode::PreComposition);
    } else if i >= len as i32 {
        context.current_state().set_candidate_pointer_index(len - 1);
        context.enter_register_mode(CompositionMode::CompositionSelection);
    } else {
        context
            .current_state()
            .set_candidate_pointer_index(i as usize);
    }
    true
}

///
/// Confirm candidate at the cursor position.
///
// TODO: Rust interface としてはcontextのメソッドにする。
// #[deprecated(
//     since = "2.0.0",
//     note = "Not proper for Rust interface. See rustdoc for details."
// )]
pub fn skk_context_confirm_candidate_at_rs(context: &mut CskkContext, i: usize) -> bool {
    if context.current_state().set_candidate_pointer_index(i) {
        context.confirm_current_composition_candidate();
        return true;
    }
    false
}

pub fn skk_context_set_auto_start_henkan_keywords_rs(
    context: &mut CskkContext,
    keywords: Vec<String>,
) {
    context.config.set_auto_start_henkan_keywords(keywords);
}

pub fn skk_context_set_period_style_rs(context: &mut CskkContext, period_style: PeriodStyle) {
    context.config.set_period_style(period_style);
}

pub fn skk_context_set_comma_style_rs(context: &mut CskkContext, comma_style: CommaStyle) {
    context.config.set_comma_style(comma_style);
}

///
/// 使えるruleのリストを返す。
///
pub fn get_available_rules() -> Result<BTreeMap<String, CskkRuleMetadataEntry>, CskkError> {
    let rulematadata = CskkRuleMetadata::load_metadata()?;
    Ok(rulematadata.get_rule_list())
}

impl CskkContext {
    ///
    /// 現在のInputModeを返す
    ///
    pub fn get_current_input_mode(&self) -> InputMode {
        self.current_state_ref().input_mode
    }

    ///
    /// 現在のCompositionModeを返す
    ///
    pub fn get_current_composition_mode(&self) -> CompositionMode {
        self.current_state_ref().composition_mode
    }

    ///
    /// Retrieve and remove the current output string
    ///
    pub fn poll_output(&mut self) -> Option<String> {
        self.retrieve_output(true)
    }

    ///
    /// pollされていない入力を状態に応じて修飾して返す。
    ///
    /// libskk互換の素朴な修飾ではないものを作る場合は[get_preedit_detail]でIME側で修飾する。
    ///
    /// TODO: 常に返るので、Optionである必要がなかった。caller側できちんとOption扱いするか、返り値の型を変えるか。
    ///
    pub fn get_preedit(&self) -> Option<String> {
        let mut result = String::new();
        let mut stack_count = 0;
        for state in &self.state_stack {
            result.push_str(&state.preedit_string(&self.kana_form_changer, state.input_mode));
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
    /// 現在の状態スタック全ての表示要素リストを返す。
    /// 最初のものが一番外側の状態で、Registerモードの時には後にその内側の状態が続く。
    /// [get_preedit]で素朴に修飾されたデータの元。
    ///
    pub fn get_preedit_detail(&self) -> Vec<CskkStateInfo> {
        let mut result = Vec::new();
        for state in &self.state_stack {
            result.push(state.preedit_detail(&self.kana_form_changer, state.input_mode))
        }

        result
    }

    ///
    /// UTF-8 character range of text to emphasize in preedit.
    ///
    /// Currently we don't have expand/shrink-preedit feature, thus we have no text we want to emphasize.
    ///
    /// # Deprecated
    ///
    /// Fancy formatting will be delegated to IMEs in favor of [get_preedit_detail]
    #[deprecated(since = "1.0.0")]
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
        let confirmed = topmost_state.get_confirmed_string().to_owned();
        if confirmed.is_empty() {
            None
        } else {
            if is_polling {
                topmost_state.flush_confirmed_string();
            }
            Some(confirmed)
        }
    }

    fn convert_kana_in_input_mode(&self, input: &str, input_mode: InputMode) -> String {
        let kana_form_changer = &self.kana_form_changer;
        kana_form_changer.adjust_kana_string(input_mode, input)
    }

    pub fn get_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn append_converted(&mut self, result: &str) {
        let current_state = self.current_state();
        let current_input_mode = current_state.input_mode;
        let kana = self.convert_kana_in_input_mode(result, current_input_mode);
        self.current_state().push_string(&kana);
    }

    fn set_unconverted(&mut self, unconv: Vec<Keysym>) {
        let current_state = self.current_state();
        current_state.pre_conversion = unconv;
    }

    #[cfg(test)]
    fn append_unconverted(&mut self, unconv: Keysym) {
        let current_state = self.current_state();
        current_state.pre_conversion.push(unconv);
    }

    fn set_carry_over(&mut self, unconv: &[Keysym]) {
        let current_state = self.current_state();
        current_state.pre_conversion = unconv.to_owned();
        if current_state.pre_conversion.is_empty() {
            self.current_state().set_capital_transition(false);
        }
    }

    ///
    /// 現在のraw_to_compositeから変換候補をリストにして、変換候補を指すポインタを0に戻す。
    ///
    fn update_candidate_list(&mut self) {
        let composite_key = self.current_state_ref().get_composite_key();
        let candidates = get_all_candidates(&self.dictionaries, &composite_key);
        self.current_state().set_new_candidate_list(candidates);
    }

    ///
    /// 現在のraw_to_compositeから補完候補をリストにして、候補ポインタを0にする。
    ///
    fn update_completion_list(&mut self) {
        let composite_key = self.current_state_ref().get_composite_key();
        let candidates = get_all_complete(&self.dictionaries, &composite_key);
        self.current_state().set_new_candidate_list(candidates);
    }

    #[allow(unused_must_use)]
    fn purge_current_composition_candidate(&mut self) {
        if let Ok(current_candidate) = self
            .current_state_ref()
            .get_candidate_list()
            .get_current_candidate()
        {
            let current_candidate = current_candidate.to_owned();
            let composite_key = self
                .current_state_ref()
                .get_candidate_list()
                .get_current_to_composite()
                .to_owned();
            for cskkdict in self.dictionaries.iter_mut() {
                purge_candidate(cskkdict, &composite_key, &current_candidate);
            }
        } else {
            log::warn!(
                "Tried to purge candidate when current candidate is not available. Skipping."
            )
        }

        self.current_state().clear_all();
    }

    // TODO: make this internal to cskkstate state内でfieldの齟齬が起きないようにcskkstate内の関数にする
    #[allow(unused_must_use)]
    fn confirm_current_composition_candidate(&mut self) {
        if let Ok(current_candidate) = self
            .current_state_ref()
            .get_candidate_list()
            .get_current_candidate()
        {
            let current_candidate = current_candidate.to_owned();

            for cskkdict in self.dictionaries.iter_mut() {
                confirm_candidate(cskkdict, &current_candidate);
            }

            let composited_okuri = self.kana_form_changer.adjust_kana_string(
                self.current_state_ref().input_mode,
                self.current_state_ref().get_okuri_string(),
            );
            let composited_kanji_and_okuri = current_candidate.output + &composited_okuri;

            let current_state = self.current_state();
            current_state.push_string(&composited_kanji_and_okuri);
            current_state.clear_unconfirmed();
            current_state.reset_to_default_composition_mode();
        } else {
            log::warn!(
                "Tried to confirm candidate when current candidate is not available. Skipping."
            )
        }
    }

    fn confirm_current_kana_to_composite(&mut self, temporary_input_mode: InputMode) {
        let kana_like = self
            .current_state_ref()
            .get_to_composite_string()
            .to_owned();
        let adjusted = if temporary_input_mode == InputMode::Zenkaku {
            self.ascii_form_changer.adjust_ascii_string(&kana_like)
        } else {
            self.convert_kana_in_input_mode(&kana_like, temporary_input_mode)
        };
        self.current_state()
            .push_string_for_composition_mode(&adjusted, CompositionMode::Direct);
        self.current_state().clear_unconfirmed();
    }

    /// Set the current composition mode.
    /// If changing to Register mode, recoomend using enter_register_mode to set the stack properly.
    fn set_composition_mode(&mut self, composition_mode: CompositionMode) {
        self.current_state()
            .change_composition_mode(composition_mode);
    }

    /// Registerモードに入る。
    /// その前のモードはprevious_composition_modeであったとみなす。
    /// Registerモードに入れなかった場合は何もしない。
    fn enter_register_mode(&mut self, previous_composition_mode: CompositionMode) {
        if self.state_stack.len() < REGISTER_MODE_STACK_MAX {
            self.current_state().change_composition_mode_from_old_mode(
                CompositionMode::Register,
                previous_composition_mode,
            );
            self.state_stack
                .push(CskkState::new(InputMode::Hiragana, CompositionMode::Direct))
        }
    }

    fn reset_state_stack(&mut self) {
        while self.state_stack.len() > 1 {
            self.state_stack.pop();
        }
        self.current_state().clear_all();
    }

    /// Register modeの時のみRegister用のスタックをpopして以前の状態に復帰させる。
    /// Register modeでない時には何もしない。
    // TODO: configurable command handlerに以降してdelegateをなくすと、previous_composition_modeまわりはクリーンアップして単にスタック戻すだけで良くなる。
    fn abort_register_mode(&mut self) -> bool {
        if self.state_stack.len() > 1 {
            self.state_stack.pop();

            true
        } else {
            false
        }
    }

    /// Registerモードを抜け、引数confirmedを変換候補として登録する。
    fn exit_register_mode(&mut self, confirmed: &str) {
        let confirmed = confirmed.to_owned();
        if self.state_stack.len() > 1 {
            self.state_stack.pop();
            if confirmed.is_empty() {
                self.current_state().abort_to_previous_mode();
            } else {
                // FIXME: refactoring. Candidate::new here looks too much...?
                let current_state = self.current_state();
                current_state.composition_mode = CompositionMode::Direct;

                let numeric_count = numeric_entry_count(&confirmed);

                if numeric_count != 0
                    && numeric_count
                        == numeric_string_count(
                            current_state
                                .get_candidate_list()
                                .get_current_to_composite()
                                .get_to_composite(),
                        )
                {
                    // 変換する文字列の数字が確定文字列の数字代理と同数含まれる場合(numeric entry)を辞書登録する。
                    // to_composite:"1かい" confirmed:"#3回" 等。
                    let (dict_key, numbers) = to_composite_to_numeric_dict_key(
                        current_state
                            .get_candidate_list()
                            .get_current_to_composite(),
                    );
                    let outputs = replace_numeric_string(&confirmed, &numbers, &self.dictionaries);
                    let mut candidates = vec![];
                    for output in outputs {
                        candidates.push(Candidate::new(
                            dict_key.get_to_composite().to_string(),
                            dict_key.get_okuri().to_owned(),
                            !self.current_state_ref().get_okuri_string().is_empty(),
                            confirmed.to_owned(),
                            None,
                            output,
                        ));
                    }
                    self.current_state()
                        .add_new_candidates_for_existing_string_to_composite(candidates);
                } else {
                    // numeric entryではない普通の変換候補としてconfirmedを追加する。
                    let registring_compsite_key = current_state
                        .get_candidate_list()
                        .get_current_to_composite();
                    let candidates = vec![Candidate::new(
                        registring_compsite_key.get_to_composite().to_string(),
                        registring_compsite_key.get_okuri().to_owned(),
                        !self.current_state_ref().get_okuri_string().is_empty(),
                        confirmed.to_owned(),
                        None,
                        confirmed,
                    )];
                    self.current_state()
                        .add_new_candidates_for_existing_string_to_composite(candidates);
                }

                self.confirm_current_composition_candidate();
            }
        }
    }

    fn set_input_mode(&mut self, input_mode: InputMode) {
        let current_state = self.current_state();
        current_state.input_mode = input_mode
    }

    ///
    /// 他の変換規則の部分列となっている規則でも変換し、入力する。
    /// return true if output such converted input.
    ///
    /// ローマ字ベースの入力規則の例ではnが残っている時に別の状態に切り替わる時に以前の状態の'ん'を入力することが多い。それを例にすると'ん'を入力する際のmodeを引数に取る。
    /// On Direct, output style is defined by given inputmode。 (ん,ン,...)
    ///
    fn output_converted_kana_if_any(
        &mut self,
        input_mode: InputMode,
        composition_mode: CompositionMode,
    ) -> bool {
        let current_state = self.current_state_ref();
        let unprocessed = current_state.pre_conversion.clone();
        let greedy_kana_convert_result = self.kana_converter.convert_greedy(&unprocessed);

        if let Some((converted_kana, carry_over)) = greedy_kana_convert_result {
            let converted_kana = converted_kana.to_owned();
            let carry_over = carry_over.to_owned();
            return match composition_mode {
                CompositionMode::Direct => {
                    let kana_form_changer = &self.kana_form_changer;
                    let adjusted =
                        kana_form_changer.adjust_kana_string(input_mode, &converted_kana);
                    self.current_state()
                        .push_string_for_composition_mode(&adjusted, composition_mode);
                    self.current_state().clear_preconverted_kanainputs();
                    self.set_carry_over(&carry_over);
                    true
                }
                CompositionMode::PreComposition => {
                    self.current_state().push_string_for_composition_mode(
                        &converted_kana,
                        CompositionMode::PreComposition,
                    );
                    self.current_state().clear_preconverted_kanainputs();
                    self.set_carry_over(&carry_over);
                    true
                }
                CompositionMode::PreCompositionOkurigana => {
                    //self.append_converted_to_okuri("ん");
                    self.current_state()
                        .push_string_for_composition_mode(&converted_kana, composition_mode);
                    self.current_state().clear_preconverted_kanainputs();
                    self.set_carry_over(&carry_over);
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
        self.process_key_event_inner_v2(key_event)
    }

    pub fn save_dictionary(&mut self) {
        for cskkdict in &self.dictionaries {
            // Using mutex in match on purpose, never acquiring lock again.
            #[allow(clippy::significant_drop_in_scrutinee)]
            let result = match *cskkdict.mutex.lock().unwrap() {
                CskkDictionaryType::StaticFile(ref mut dictionary) => dictionary.save_dictionary(),
                CskkDictionaryType::UserFile(ref mut dictionary) => dictionary.save_dictionary(),
                CskkDictionaryType::EmptyDict(ref mut dictionary) => dictionary.save_dictionary(),
            };
            match result {
                Ok(_) => {}
                Err(error) => {
                    log::warn!("{}", &error.to_string());
                }
            }
        }
    }

    pub fn reload_dictionary(&mut self) {
        for cskkdict in &self.dictionaries {
            // Using mutex in match on purpose, never acquiring lock again.
            #[allow(clippy::significant_drop_in_scrutinee)]
            let result = match *cskkdict.mutex.lock().unwrap() {
                CskkDictionaryType::StaticFile(ref mut dictionary) => dictionary.reload(),
                CskkDictionaryType::UserFile(ref mut dictionary) => dictionary.reload(),
                CskkDictionaryType::EmptyDict(_) => Ok(()),
            };
            match result {
                Ok(_) => {}
                Err(error) => {
                    log::warn!("{}", &error.to_string());
                }
            }
        }
    }

    pub fn set_dictionaries(&mut self, dicts: Vec<Arc<CskkDictionary>>) {
        self.dictionaries = dicts;
    }

    /// 大文字であり、かつコマンドではないキー入力をした時のモード変更を行う。
    ///
    /// done_transition_on_kana_build: 現在のkanabuildで既にモード変更を行っているかどうか。
    ///
    fn transition_composition_mode_by_capital_letter(
        &mut self,
        key_event: &CskkKeyEvent,
        initial_kanainput_composition_mode: CompositionMode,
        done_transition_on_kana_build: bool,
    ) -> bool {
        let is_capital = key_event.is_upper();
        let is_empty = self
            .current_state_ref()
            .get_to_composite_string()
            .is_empty();

        return if is_capital
            && !done_transition_on_kana_build
            && initial_kanainput_composition_mode == CompositionMode::Direct
        {
            self.set_composition_mode(CompositionMode::PreComposition);
            self.current_state().set_capital_transition(true);
            true
        } else if is_capital
            && !is_empty
            && !done_transition_on_kana_build
            && initial_kanainput_composition_mode == CompositionMode::PreComposition
        {
            self.set_composition_mode(CompositionMode::PreCompositionOkurigana);
            self.current_state().set_capital_transition(true);
            true
        } else {
            false
        };
    }

    ///
    /// かな変換できた時に、それを入力する。
    ///
    fn input_converted_kana(
        &mut self,
        converted: &str,
        carry_over: Vec<Keysym>,
        did_change_mode_by_capital: bool,
        initial_unprocessed_vector: Vec<Keysym>,
    ) {
        match self.current_state_ref().composition_mode {
            CompositionMode::Direct => {
                self.append_converted(converted);
                self.set_carry_over(&carry_over);
            }
            CompositionMode::PreComposition => {
                self.current_state().push_string(converted);
                self.set_carry_over(&carry_over);
                self.auto_start_henkan();
            }
            CompositionMode::PreCompositionOkurigana => {
                if did_change_mode_by_capital
                    && !initial_unprocessed_vector.is_empty()
                    && !carry_over.is_empty()
                {
                    // 以前入力されていた部分はPreComposition側として処理する。
                    // 例: "t T" の't'部分が 'っ' とかな変換される場合
                    // '察する'等 carry_over以降が送り仮名となる
                    self.current_state().push_string_for_composition_mode(
                        converted,
                        CompositionMode::PreComposition,
                    );
                } else {
                    self.current_state().push_string(converted);
                }
                self.set_carry_over(&carry_over);

                // 入力単独によらない特殊な遷移で、かな変換の結果によって▽モードから▼モードへ移行する。
                if carry_over.is_empty()
                    && !self
                        .current_state_ref()
                        .get_to_composite_string()
                        .is_empty()
                {
                    // この部分TryNextCandidateと似ているが、事前条件が違うので共通にできなかった。
                    self.update_candidate_list();
                    if self.current_state_ref().get_candidate_list().is_empty() {
                        let prev_composition_mode = self.current_state_ref().composition_mode;
                        self.enter_register_mode(prev_composition_mode);
                    } else {
                        self.set_composition_mode(CompositionMode::CompositionSelection);
                    }
                }
            }
            _ => {
                log::debug!("Tried to input converted kana in no kana henkan mode. Ignored.");
            }
        }
    }

    /// 上から順に
    /// 1.そのまま、あるいは小文字化するとrom_kana変換決定可能ならcommandとして解釈をスキップする
    /// 2.commandとして解釈されるならcommandとして実行、
    /// 3.各モードで入力として処理
    ///
    /// 1. は"z l"から"→"の変換のように、コマンドでもひらがな変換に使うことがあるための処置。即rom_kana以外ではコマンド優先。
    fn process_key_event_inner_v2(&mut self, key_event: &CskkKeyEvent) -> bool {
        log::debug!("Keyevent: {:?}", key_event);
        let initial_composition_mode = self.current_state_ref().composition_mode;
        let initial_input_mode = self.current_state_ref().input_mode;

        let initial_unprocessed_vector = self.current_state_ref().pre_conversion.clone();
        let lower_combined_keyinputs =
            KanaBuilder::combine_lower(key_event, &initial_unprocessed_vector);
        let raw_combined_keyinputs =
            KanaBuilder::combine_raw(key_event, &initial_unprocessed_vector);
        let unchecked_lower_kana_convert_result = self
            .kana_converter
            .convert_non_partial(&lower_combined_keyinputs);
        let unchecked_raw_kana_convert_result = self
            .kana_converter
            .convert_non_partial(&raw_combined_keyinputs);

        let skip_command = key_event.is_modifierless_input()
            && (has_rom2kana_conversion(&initial_input_mode, &initial_composition_mode)
                && (unchecked_lower_kana_convert_result.is_some()
                    || unchecked_raw_kana_convert_result.is_some()));

        if !skip_command {
            let maybe_instructions = self.get_handler_v2().get_instruction(
                key_event,
                &initial_input_mode,
                &initial_composition_mode,
            );
            if let Some(instructions) = maybe_instructions {
                return self.process_instructions(
                    &instructions,
                    initial_composition_mode,
                    initial_input_mode,
                );
            }
        }

        // CompositionSelectionの選択キーをlibcskkではなく外部のUI所持側が持つ想定なので、ここで処理しない。

        if !key_event.is_modifierless_input() {
            return false;
        }

        // ここ以降がコマンドではない通常のキー入力扱い

        // CompositionSelectionModeやCompletionModeで、入力っぽいと現在の選択肢で確定をしてDirectモードとして処理させる
        let will_be_processed = key_event.is_modifierless_input()
            && (has_rom2kana_conversion(
                &self.current_state_ref().input_mode,
                &self.current_state_ref().composition_mode,
            ) && (self
                .kana_converter
                .can_continue(key_event, &initial_unprocessed_vector)
                || self.kana_converter.can_continue(key_event, &[])))
            || (!has_rom2kana_conversion(
                &self.current_state_ref().input_mode,
                &self.current_state_ref().composition_mode,
            ) && key_event.is_ascii_inputtable());
        if matches!(
            initial_composition_mode,
            CompositionMode::CompositionSelection | CompositionMode::Completion
        ) && will_be_processed
        {
            self.confirm_current_composition_candidate();
        }

        if has_rom2kana_conversion(
            &self.current_state_ref().input_mode,
            &self.current_state_ref().composition_mode,
        ) && key_event.is_modifierless_input()
        {
            let combined_raw = KanaBuilder::combine_raw(key_event, &initial_unprocessed_vector);
            let combined_lower = KanaBuilder::combine_lower(key_event, &initial_unprocessed_vector);
            let alone_raw = KanaBuilder::combine_raw(key_event, &[]);
            let alone_lower = KanaBuilder::combine_lower(key_event, &[]);

            // 上の間の処理で所有権を失っているので再度convert_non_partialを呼ぶ。
            // FIXME: RustのバージョンがあがってSome内部のrefを上手くto_ownedみたいな扱いが簡単に書けるようになったら
            // immutable borrowから切り離したunchecked_lower_kana_convert_resultを再利用する。
            let converted_lower = self.kana_converter.convert_non_partial(&combined_lower);
            let initial_kanainput_composition_mode = self.current_state_ref().composition_mode;

            if let Some((converted, carry_over)) =
                self.kana_converter.convert_non_partial(&combined_raw)
            {
                // When input made a kana conversion in raw input.
                // Even if matched in upper case, this won't try to change the composition mode.
                let converted = converted.clone();
                let carry_over = carry_over.clone();

                self.input_converted_kana(
                    &converted,
                    carry_over,
                    false,
                    initial_unprocessed_vector,
                );
                return true;
            } else if key_event.is_upper() {
                if let Some((converted, carry_over)) = converted_lower {
                    // When input can make kana conversion in lowercase input.
                    // Try changing the composition mode and then insert kana.

                    let converted = converted.clone();
                    let carry_over = carry_over.clone();

                    let did_change_mode = self.transition_composition_mode_by_capital_letter(
                        key_event,
                        initial_kanainput_composition_mode,
                        self.current_state_ref().get_capital_transition(),
                    );
                    // When input made a kana conversion when tried with lower letter.
                    self.input_converted_kana(
                        &converted,
                        carry_over,
                        did_change_mode,
                        initial_unprocessed_vector,
                    );
                    return true;
                }
            }

            // character input didn't make kana conversion in this else flow.
            let current_input_mode = self.current_state_ref().input_mode;

            if let Some(key_char) = key_event.get_symbol_char() {
                // カンマピリオドは特殊な設定と処理がある。 TODO: これも一般化したい。
                if let Some(converted) = self.kana_converter.convert_periods(
                    &key_char,
                    self.config.period_style,
                    self.config.comma_style,
                ) {
                    // カンマピリオド確定なのでcompositionmode変更は割愛。
                    // まず他の入力があれば終わらせる。
                    self.output_converted_kana_if_any(
                        current_input_mode,
                        initial_kanainput_composition_mode,
                    );
                    self.current_state().clear_preconverted_kanainputs();

                    // input_as_direct_char
                    match &self.current_state_ref().composition_mode {
                        CompositionMode::Direct => {
                            self.append_converted(&converted);
                        }
                        CompositionMode::PreComposition => {
                            self.current_state().push_string(&converted);
                            self.current_state().clear_preconverted_kanainputs();
                        }
                        CompositionMode::PreCompositionOkurigana => {
                            self.current_state().push_string(&converted);
                            self.current_state().clear_preconverted_kanainputs();
                        }
                        _ => {
                            log::debug!("Unreachable");
                            return false;
                        }
                    }
                } else if self
                    .kana_converter
                    .can_continue(key_event, &initial_unprocessed_vector)
                {
                    // そのままでかな入力の続きとなれる入力なので、そのように処理
                    self.input_as_continuous_kana(key_event);
                } else if key_event.is_upper()
                    && self
                        .kana_converter
                        .can_continue_lower(key_event, &initial_unprocessed_vector)
                {
                    // モード変更として捉えればかな入力の続きとなれる入力。
                    self.transition_composition_mode_by_capital_letter(
                        key_event,
                        initial_kanainput_composition_mode,
                        self.current_state_ref().get_capital_transition(),
                    );
                    let lower_key_event = key_event.to_lower();
                    self.input_as_continuous_kana(&lower_key_event);
                } else if let Some((converted, carry_over)) =
                    self.kana_converter.convert_non_partial(&alone_raw)
                {
                    // かな入力として成立しないが単純変換のある入力。続けて入力はできないがかな変換はできる文字。
                    let converted = converted.to_owned();
                    let carry_over = carry_over.clone();
                    self.input_converted_kana(
                        &converted,
                        carry_over,
                        false,
                        initial_unprocessed_vector,
                    );
                    return true;
                } else if let Some((converted, carry_over)) =
                    self.kana_converter.convert_non_partial(&alone_lower)
                {
                    // かな入力として成立しないがモード変更と捉えればかな単純変換の入力となる文字
                    if key_event.is_upper() {
                        let converted = converted.to_owned();
                        let carry_over = carry_over.clone();
                        let did_change_mode = self.transition_composition_mode_by_capital_letter(
                            key_event,
                            initial_kanainput_composition_mode,
                            false,
                        );

                        self.input_converted_kana(
                            &converted,
                            carry_over,
                            did_change_mode,
                            initial_unprocessed_vector,
                        );
                        return true;
                    }
                } else if self.kana_converter.can_continue(key_event, &[]) {
                    // かな入力として成立しない子音の連続等、続けては入力できないがkanabuilderで扱える文字。
                    // まずpre_convertedを整理してから入力として扱う。
                    self.output_converted_kana_if_any(
                        current_input_mode,
                        initial_kanainput_composition_mode,
                    );
                    self.current_state().clear_preconverted_kanainputs();

                    self.input_as_continuous_kana(key_event);
                } else if self.kana_converter.can_continue_lower(key_event, &[]) {
                    // 大文字をモード変更として捉えて小文字化すればkanabuilderで扱える入力
                    self.transition_composition_mode_by_capital_letter(
                        key_event,
                        initial_kanainput_composition_mode,
                        self.current_state_ref().get_capital_transition(),
                    );
                    let lower_key_event = key_event.to_lower();

                    // TODO: この現在のステートのクリーンアップは上と同じであるべきなので、stateの整理ができたらメソッドにまとめる。
                    self.output_converted_kana_if_any(
                        current_input_mode,
                        initial_kanainput_composition_mode,
                    );
                    self.current_state().clear_preconverted_kanainputs();

                    self.input_as_continuous_kana(&lower_key_event);
                } else {
                    // kana builderですら扱えないキー。
                    // スペースや記号を想定。
                    // 他と同様のクリンアップの後、直接入力として処理する。
                    // FIXME: key_charに依存しているので、複数文字を入れることが想定できていない。
                    // 特殊なキーボードで変な文字を入力することになるのを防ぐため、とりあえずAscii文字に限定している。コマンドや変換同様に制限を外したい。
                    if key_event.is_ascii_inputtable() {
                        self.output_converted_kana_if_any(
                            current_input_mode,
                            initial_kanainput_composition_mode,
                        );
                        self.current_state().clear_preconverted_kanainputs();

                        match &self.current_state_ref().composition_mode {
                            CompositionMode::Direct
                            | CompositionMode::PreComposition
                            | CompositionMode::PreCompositionOkurigana => {
                                self.current_state().push_string(&key_char.to_string());
                            }

                            _ => {
                                log::debug!("Unreachable. kana builder unacceptable key given in wrong composition mode.");
                                return false;
                            }
                        }
                    } else {
                        return false;
                    }
                }

                if self.current_state_ref().composition_mode == CompositionMode::PreComposition {
                    self.auto_start_henkan();
                }
                return true;
            };
        } else if key_event.is_ascii_inputtable() && key_event.is_modifierless_input() {
            // key was input, but not in rom-kana conversion related modes so skip rom-kana related and input as is.
            match &self.current_state_ref().composition_mode {
                CompositionMode::Direct => {
                    if let Some(key_char) = key_event.get_symbol_char() {
                        match &self.current_state_ref().input_mode {
                            InputMode::Ascii => {
                                self.current_state().push_string(&key_char.to_string());
                                return true;
                            }
                            InputMode::Zenkaku => {
                                let zenkaku = self.ascii_form_changer.adjust_ascii_char(key_char);
                                self.append_converted(&zenkaku);
                                return true;
                            }
                            _ => {
                                log::debug!(
                                    "Unreachable. Ascii or Zenkaku only should be in Direct mode can reach here. Ignoring key event: {:?}", key_event
                                );
                            }
                        }
                    }
                }
                CompositionMode::Abbreviation => {
                    if let Some(key_char) = key_event.get_symbol_char() {
                        self.current_state().push_string(&key_char.to_string());
                        return true;
                    }
                }
                _ => {
                    log::debug!(
                        "Unreachable by rom2kana check. Ignoring key event: {:?}",
                        key_event
                    )
                }
            }
        }

        false
    }

    // Returns true if consumes key event with these instructions and don't have to process further.
    // Returns false if explicitly handle as command but have to pass through the key event.
    fn process_instructions(
        &mut self,
        instructions: &Vec<Instruction>,
        initial_composition_mode: CompositionMode,
        initial_input_mode: InputMode,
    ) -> bool {
        for instruction in instructions {
            log::debug!("{:?}", &instruction);
            match instruction {
                Instruction::ChangeCompositionMode(composition_mode) => {
                    self.set_composition_mode(*composition_mode);
                }
                Instruction::ChangeInputMode(input_mode) => {
                    self.set_input_mode(*input_mode);
                }
                Instruction::ForceKanaConvert(input_mode) => {
                    self.output_converted_kana_if_any(*input_mode, initial_composition_mode);
                }
                Instruction::ClearUnconvertedInputs => {
                    self.current_state().clear_preconverted_kanainputs();
                }
                Instruction::ClearKanaConvertedInputs => {
                    self.current_state().clear_kanas();
                }
                Instruction::ClearUnconfirmedInputs => {
                    self.current_state().clear_unconfirmed();
                }
                Instruction::Abort => {
                    self.abort();
                }
                Instruction::ConfirmComposition => {
                    self.confirm_current_composition_candidate();
                }
                Instruction::PassthroughKeyEvent => {
                    return false;
                }
                Instruction::ConfirmAs(input_mode) => {
                    self.confirm_current_kana_to_composite(*input_mode);
                }
                Instruction::ConfirmDirect => {
                    return if self.state_stack.len() > 1 {
                        let confirmed = self.current_state_ref().get_confirmed_string().to_owned();
                        self.exit_register_mode(&confirmed);
                        true
                    } else {
                        false
                    }
                }
                Instruction::Purge => {
                    self.purge_current_composition_candidate();
                }
                Instruction::NextCandidatePointer => {
                    self.current_state().forward_candidate();
                }
                Instruction::PreviousCandidatePointer => {
                    self.current_state().backward_candidate();
                }
                Instruction::Delete => {
                    return self.current_state().delete();
                }
                Instruction::TryNextCandidate => {
                    self.try_next_candidate(initial_composition_mode, initial_input_mode);
                }
                Instruction::TryPreviousCandidate => {
                    if self.current_state_ref().composition_mode
                        != CompositionMode::CompositionSelection
                    {
                        log::debug!(
                            "Trying previous candidate on not composition selection mode. Ignore."
                        )
                    } else if !self.current_state_ref().get_candidate_list().has_previous() {
                        self.current_state().consolidate_converted_to_to_composite();
                        self.current_state().clear_candidate_list();
                        self.set_composition_mode(CompositionMode::PreComposition);
                    } else {
                        self.current_state().backward_candidate();
                    }
                }
                Instruction::TryNextCompletion => {
                    self.try_next_completion(initial_composition_mode, initial_input_mode);
                }
                Instruction::TryPreviousCompletion => {
                    if self.current_state_ref().composition_mode != CompositionMode::Completion {
                        log::debug!(
                            "Trying previous completion on not completion selection mode. Ignore."
                        )
                    } else if self.current_state_ref().get_candidate_list().has_previous() {
                        self.current_state().backward_candidate();
                    } // Do nothing when no previous
                }
                #[allow(unreachable_patterns)]
                _ => {
                    log::debug!("unimplemented instruction: {:?}", &instruction);
                }
            }
        }

        !instructions.is_empty()
    }

    // if has candidate and has next, pointer to the next candidate and set to composition selection mode.
    // if not, go to registeration mode.
    fn try_next_candidate(
        &mut self,
        initial_composition_mode: CompositionMode,
        initial_input_mode: InputMode,
    ) {
        if initial_composition_mode != CompositionMode::CompositionSelection {
            self.output_converted_kana_if_any(initial_input_mode, initial_composition_mode);
            self.update_candidate_list();
            if !self
                .current_state_ref()
                .get_to_composite_string()
                .is_empty()
                && self.current_state_ref().get_candidate_list().is_empty()
            {
                self.enter_register_mode(initial_composition_mode);
            } else if !self.current_state_ref().get_candidate_list().is_empty() {
                self.set_composition_mode(CompositionMode::CompositionSelection);
            }
        } else if !self.current_state_ref().get_candidate_list().has_next() {
            self.enter_register_mode(initial_composition_mode);
        } else {
            self.current_state().forward_candidate();
        }
    }

    // 次の補完候補があれば指す、そうでなければ何もしない。
    fn try_next_completion(
        &mut self,
        initial_composition_mode: CompositionMode,
        initial_input_mode: InputMode,
    ) {
        if initial_composition_mode != CompositionMode::Completion {
            self.output_converted_kana_if_any(initial_input_mode, initial_composition_mode);
            self.update_completion_list();
            if !self
                .current_state_ref()
                .get_to_composite_string()
                .is_empty()
                && self.current_state_ref().get_candidate_list().is_empty()
            {
                // 何もしない。
            } else if !self.current_state_ref().get_candidate_list().is_empty() {
                self.set_composition_mode(CompositionMode::Completion);
            }
        } else if !self.current_state_ref().get_candidate_list().has_next() {
            // 何もしない
        } else {
            self.current_state().forward_candidate();
            self.set_composition_mode(CompositionMode::Completion);
        }
    }

    /// check and attempt to start auto_start_henkan.
    /// return true if should go to next candidate
    fn auto_start_henkan(&mut self) {
        assert_eq!(
            self.current_state_ref().composition_mode,
            CompositionMode::PreComposition
        );
        // If composite_key ends with auto_start_henkan keywords and also the composite_key is not empty,
        // remove that from key and enter composition selection mode.
        let mut auto_start_henkan_keyword_matched = false;
        let composite_key = self.current_state_ref().get_composite_key();
        let raw_to_composite = composite_key.get_to_composite();
        for suffix in &self.config.auto_start_henkan_keywords.clone() {
            if !auto_start_henkan_keyword_matched
                && !raw_to_composite.eq(suffix)
                && raw_to_composite.ends_with(suffix)
            {
                // suffix matched the current composite_key's end
                // Now remove suffix from composite_key and put it to postfix.
                for _ in 0..suffix.chars().count() {
                    self.current_state().delete();
                }
                self.current_state().set_converted_to_postfix(suffix);
                auto_start_henkan_keyword_matched = true;
                break;
            }
        }

        if auto_start_henkan_keyword_matched {
            self.try_next_candidate(
                CompositionMode::PreComposition,
                self.current_state_ref().input_mode,
            );
        }
    }

    /// process key_event as character input without conversion or composition.
    /// 現在のCompositionModeによってどこに付加されるかが変わる。
    fn input_as_continuous_kana(&mut self, key_event: &CskkKeyEvent) -> bool {
        match &self.current_state_ref().composition_mode {
            CompositionMode::Direct | CompositionMode::PreComposition => {
                self.set_unconverted(
                    self.kana_converter.next_unprocessed_state(
                        key_event,
                        &self.current_state_ref().pre_conversion,
                    ),
                );
            }
            CompositionMode::PreCompositionOkurigana => {
                self.set_unconverted(
                    self.kana_converter.next_unprocessed_state(
                        key_event,
                        &self.current_state_ref().pre_conversion,
                    ),
                );
            }
            _ => {
                log::debug!("Unreachable.");
                return false;
            }
        }
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
    ///
    pub fn will_process(&self, key_event: &CskkKeyEvent) -> bool {
        let current_state = self.current_state_ref();
        let handler = self.get_handler_v2();
        let maybe_instruction = handler.get_instruction(
            key_event,
            &current_state.input_mode,
            &current_state.composition_mode,
        );
        let has_rom2kana_in_current_mode =
            has_rom2kana_conversion(&current_state.input_mode, &current_state.composition_mode);
        let will_be_character_input = key_event.is_modifierless_input()
            && ((has_rom2kana_in_current_mode
                && (self
                    .kana_converter
                    .can_continue(key_event, &current_state.pre_conversion)
                    || self.kana_converter.can_continue(key_event, &[])))
                || (!has_rom2kana_in_current_mode && key_event.is_ascii_inputtable()));

        maybe_instruction.is_some() || will_be_character_input
    }

    fn get_handler_v2(&self) -> &ConfigurableCommandHandler {
        &self.command_handler
    }

    /// Exposed for testing purpose. Recommended not to use.
    pub fn process_key_events_string(&mut self, key_event_string: &str) -> bool {
        self.process_key_events(&CskkKeyEvent::deserialize_seq(key_event_string).unwrap())
    }

    /// Mainly for test purpose, but exposed to test as library.
    fn process_key_events(&mut self, key_event_seq: &KeyEventSeq) -> bool {
        for key_event in key_event_seq {
            let processed = self.process_key_event(key_event);
            if !processed {
                log::debug!("Key event not processed: {:?}", key_event);
            }
            log::debug!("State stack: {:#?}", &self.state_stack);
        }
        true
    }

    /// Set to the specified rule.
    /// Rules will be read from XDG data directory which has libcskk/rules directory.
    pub fn set_rule(&mut self, rule: &str) -> Result<(), CskkError> {
        let rule_metadata = CskkRuleMetadata::load_metadata()?;
        let new_rule = rule_metadata.load_rule(rule)?;
        self.set_rule_inner(&new_rule)
    }

    /// Set to the specified rule from specified directory.
    /// For testing purpose. Use [set_rule] instead.
    pub fn set_rule_from_directory(
        &mut self,
        rule: &str,
        rule_dirpath: &str,
    ) -> Result<(), CskkError> {
        let rule_metadata = CskkRuleMetadata::load_metadata_from_directory(rule_dirpath)?;
        let new_rule = rule_metadata.load_rule(rule)?;
        self.set_rule_inner(&new_rule)
    }

    fn set_rule_inner(&mut self, new_rule: &CskkRule) -> Result<(), CskkError> {
        let new_kana_converter = Box::new(KanaBuilder::new(new_rule));
        let new_command_handler = ConfigurableCommandHandler::new(new_rule);
        self.kana_converter = new_kana_converter;
        self.command_handler = new_command_handler;
        Ok(())
    }

    /// Create a new cskk context.
    pub fn new(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        dictionaries: Vec<Arc<CskkDictionary>>,
    ) -> Result<Self, CskkError> {
        let rule_metadata = CskkRuleMetadata::load_metadata()?;
        let default_rule = rule_metadata.load_default_rule()?;

        let kana_converter = Box::new(KanaBuilder::new(&default_rule));
        let command_handler = ConfigurableCommandHandler::new(&default_rule);
        let initial_stack = vec![CskkState::new(input_mode, composition_mode)];

        Ok(Self {
            state_stack: initial_stack,
            kana_converter,
            command_handler,
            kana_form_changer: KanaFormChanger::default_kanaform_changer(),
            ascii_form_changer: AsciiFormChanger::default_ascii_form_changer(),
            dictionaries,
            config: CskkConfig::default(),
        })
    }

    /// Create a new cskk context.
    /// When setup fails, still returns an context that can convert nothing.
    /// This is for IMEs that cannot fail gracefully on creating a context.
    pub fn new_with_empty_fallback(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        dictionaries: Vec<Arc<CskkDictionary>>,
    ) -> Self {
        if let Ok(result) = Self::new(input_mode, composition_mode, dictionaries.clone()) {
            result
        } else {
            Self::new_empty(input_mode, composition_mode, dictionaries)
        }
    }

    ///
    /// Creates a context that can convert nothing.
    /// Exposed only for test purpose. Users may not use this method.
    ///
    /// Use [new] or [new_with_empty_fallback] instead.
    ///
    pub fn new_empty(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        dictionaries: Vec<Arc<CskkDictionary>>,
    ) -> Self {
        let kana_converter = Box::new(KanaBuilder::new_empty());
        let command_handler = ConfigurableCommandHandler::new_empty();
        let initial_stack = vec![CskkState::new(input_mode, composition_mode)];

        Self {
            state_stack: initial_stack,
            kana_converter,
            command_handler,
            kana_form_changer: KanaFormChanger::default_kanaform_changer(),
            ascii_form_changer: AsciiFormChanger::default_ascii_form_changer(),
            dictionaries,
            config: CskkConfig::default(),
        }
    }

    /// This method is for e2e test purpose.
    /// Use [new] for your use.
    pub fn new_from_specified_paths(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        dictionaries: Vec<Arc<CskkDictionary>>,
        kana_form_changer_filepath: &str,
        ascii_from_changer_filepath: &str,
        rule_filepath: &str,
    ) -> Self {
        let rule_metadata = CskkRuleMetadata::load_metadata_from_directory(rule_filepath).unwrap();
        let default_rule = rule_metadata.load_default_rule().unwrap();
        let kana_converter = Box::new(KanaBuilder::new(&default_rule));
        let command_handler = ConfigurableCommandHandler::new(&default_rule);

        let initial_stack = vec![CskkState::new(input_mode, composition_mode)];
        Self {
            state_stack: initial_stack,
            kana_converter,
            command_handler,
            kana_form_changer: KanaFormChanger::from_file(kana_form_changer_filepath),
            ascii_form_changer: AsciiFormChanger::from_file(ascii_from_changer_filepath),
            dictionaries,
            config: CskkConfig::default(),
        }
    }

    /// Abort処理をする。
    /// Direct時はRegisterモードを止め、それ以外は直前のモードに戻す。
    /// PreCompositionOkuriganaは一気にCompositionモードよりも前に戻す。
    fn abort(&mut self) {
        self.current_state().clear_preconverted_kanainputs();
        self.current_state().consolidate_converted_to_to_composite();
        self.current_state().clear_candidate_list();
        let before_composition_mode = self.current_state_ref().composition_mode;
        if before_composition_mode == CompositionMode::Direct {
            self.abort_register_mode();
        } // registerモード時でもcompositionmodeの復帰は必要なので、else不要

        self.current_state().abort_to_previous_mode();
        if before_composition_mode == CompositionMode::PreCompositionOkurigana
            && self.current_state_ref().composition_mode == CompositionMode::PreComposition
        {
            self.current_state().abort_to_previous_mode();
        }
    }
}

impl Display for CskkState {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            r#"{{
    inputmode: {:?}
    compositionmode: {:?}
    confirmed: {}"#,
            self.input_mode,
            self.composition_mode,
            self.get_confirmed_string()
        );
        write!(f, "    unconverted:");
        for c in &self.pre_conversion {
            write!(f, "{c}");
        }
        writeln!(f);
        writeln!(f, "}}");
        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::cskkstate::PreCompositionData;
    use crate::testhelper::init_test_logger;
    use xkbcommon::xkb::keysyms;

    fn new_test_context(input_mode: InputMode, composition_mode: CompositionMode) -> CskkContext {
        let dict = Arc::new(
            CskkDictionary::new_static_dict("tests/data/dictionaries/SKK-JISYO.S", "euc-jp", false)
                .unwrap(),
        );
        let dictionaries = vec![dict];

        let rule_metadata = CskkRuleMetadata::load_metadata_from_directory("assets/rules")
            .expect("Failed to load metadata for test context");
        let default_rule = rule_metadata.load_default_rule().unwrap();

        let kana_converter = Box::new(KanaBuilder::test_converter());
        let command_handler = ConfigurableCommandHandler::new(&default_rule);

        let initial_stack = vec![CskkState::new(input_mode, composition_mode)];
        CskkContext {
            state_stack: initial_stack,
            kana_converter,
            command_handler,
            kana_form_changer: KanaFormChanger::test_kana_form_changer(),
            ascii_form_changer: AsciiFormChanger::test_ascii_form_changer(),
            dictionaries,
            config: CskkConfig::default(),
            //rule_metadata,
        }
    }

    #[test]
    fn will_process() {
        let cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);
        let a = CskkKeyEvent::from_string_representation("a").unwrap();
        assert!(cskkcontext.will_process(&a));
        let copy = CskkKeyEvent::from_string_representation("C-c").unwrap();
        assert!(!cskkcontext.will_process(&copy));
    }

    #[test]
    fn process_key_event() {
        let mut cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);

        let a = CskkKeyEvent::from_string_representation("a").unwrap();
        let result = cskkcontext.process_key_event(&a);
        assert!(result);
    }

    #[test]
    fn retrieve_output() {
        let mut cskkcontext = new_test_context(InputMode::Ascii, CompositionMode::Direct);
        let a = CskkKeyEvent::from_string_representation("a").unwrap();
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
        let a = CskkKeyEvent::from_string_representation("a").unwrap();
        cskkcontext.process_key_event(&a);
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("a", actual);
        let after = cskkcontext.poll_output();
        assert_eq!(None, after);
    }

    #[test]
    fn get_preedit() {
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        let capital_a = CskkKeyEvent::from_string_representation("A").unwrap();
        cskkcontext.process_key_event(&capital_a);
        let actual = cskkcontext.get_preedit().unwrap_or_else(|| {
            panic!("No preedit. context: {:?}", cskkcontext.current_state_ref())
        });
        assert_eq!("▽あ", actual);
    }

    #[test]
    fn get_preedit_register_mode() {
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext
            .current_state()
            .push_string_for_composition_mode("ほげ", CompositionMode::PreComposition);
        cskkcontext.enter_register_mode(CompositionMode::Direct);
        cskkcontext.append_converted("あか");
        cskkcontext.append_unconverted(keysyms::KEY_s);
        let actual = cskkcontext.get_preedit().unwrap_or_else(|| {
            panic!("No preedit. context: {:?}", cskkcontext.current_state_ref())
        });
        assert_eq!("▼ほげ【あかs】", actual);
    }

    #[test]
    fn process_backspace() {
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext.process_key_event(&CskkKeyEvent::from_string_representation("h").unwrap());
        let actual = cskkcontext
            .process_key_event(&CskkKeyEvent::from_string_representation("BackSpace").unwrap());
        assert!(actual);
    }

    #[test]
    fn process_period() {
        init_test_logger();
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext.process_key_event(&CskkKeyEvent::from_string_representation("h").unwrap());
        let actual = cskkcontext
            .process_key_event(&CskkKeyEvent::from_string_representation("period").unwrap());
        assert!(actual);
    }

    #[test]
    fn dont_process_return_in_direct() {
        init_test_logger();
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        let actual = cskkcontext
            .process_key_event(&CskkKeyEvent::from_string_representation("Return").unwrap());
        assert!(!actual);
    }

    #[test]
    fn consume_key_on_delete_on_0_letter_precomposition() {
        init_test_logger();
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext.process_key_event(&CskkKeyEvent::from_string_representation("A").unwrap());
        cskkcontext
            .process_key_event(&CskkKeyEvent::from_string_representation("BackSpace").unwrap());
        let actual = cskkcontext
            .process_key_event(&CskkKeyEvent::from_string_representation("BackSpace").unwrap());
        assert!(actual);
    }

    #[test]
    fn preedit_details_basic() {
        init_test_logger();
        let mut cskkcontext = new_test_context(InputMode::Hiragana, CompositionMode::Direct);
        cskkcontext.process_key_event(&CskkKeyEvent::from_string_representation("A").unwrap());
        let result = cskkcontext.get_preedit_detail();
        assert_eq!(result.len(), 1);
        let top_state = result.get(0).unwrap();
        assert!(matches!(top_state, CskkStateInfo::PreComposition(_)));
        assert_eq!(
            *top_state,
            CskkStateInfo::PreComposition(PreCompositionData {
                confirmed: "".to_string(),
                kana_to_composite: "あ".to_string(),
                okuri: None,
                unconverted: None
            })
        )
    }
}
