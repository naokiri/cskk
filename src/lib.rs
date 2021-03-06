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

use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::iter::FromIterator;

use log::debug;
#[allow(unused_imports)]
use log::log;

use crate::dictionary::on_memory_dict::OnMemoryDict;
use crate::command_handler::CommandHandler;
use crate::command_handler::kana_composition_handler::KanaCompositionHandler;
use crate::command_handler::direct_mode_command_handler::DirectModeCommandHandler;
use crate::command_handler::kana_precomposition_handler::KanaPrecompositionHandler;
use crate::kana_builder::KanaBuilder;
use crate::keyevent::KeyEvent;
use crate::keyevent::KeyEventSeq;
use crate::skk_modes::CompositionMode;
use crate::skk_modes::InputMode;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::kana_form_changer::KanaFormChanger;

pub mod skk_modes;
mod kana_builder;
mod keyevent;
mod command_handler;
mod dictionary;
mod kana_form_changer;

#[derive(Deserialize)]
#[allow(dead_code)]
struct RuleMeta {
    name: String,
    root: bool,
    description: String,
    import: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Instruction<'a> {
    #[allow(dead_code)]
    Abort,
    ChangeInputMode(InputMode),
    //StackRegisterMode,
    /// Try to convert preconversion if in input mode which has conversion. Mostly (or only?) just for single 'n' conversion.
    ForceConvertPreConversion,
    /// Aborts if empty after flush.
    FlushPreviousCarryOver,
    // FIXME: PrecompositionからspaceでCompositionを働かせるためにDelegateを作ったが、Delegate無限ループに陥いらないようにする仕組みがない。
    // delegate: delegate processing current key. Run the key event handling again on the next mode with current key.
    ChangeCompositionMode { composition_mode: CompositionMode, delegate: bool },
    // モード変更などで入力を処理し、入力モードの入力としての処理をしない命令
    FinishConsumingKeyEvent,
    // keyeventを処理しなかったとして処理を終了する。ueno/libskkでの"*-unhandled"系命令用
    #[allow(dead_code)]
    FinishNotConsumingKeyEvent,
    // 今の変換候補を変更する。
    SetComposition { kanji: &'a str },
    // 現在の変換候補で確定する
    ConfirmComposition,
    ConfirmAsKatakana,
    ConfirmAsJISX0201,
}


/// Rough design prototype yet
///
/// TODO: Rustのstructまわりの一部分mutに変更があったら非mutでstateアクセスしているところを直す
/// FIXME: Handler保持をもうちょっとスマートにしたい。
/// TODO: ueno/libskkの設計の理由がよくわからず同じようにhandlerをcontextに入れているが、せいぜいDict類くらいを保持するようにして移せるものはstaticに移す？
///
pub struct CskkContext {
    state_stack: Vec<RefCell<CskkState>>,
    direct_handler: DirectModeCommandHandler,
    kana_precomposition_handler: KanaPrecompositionHandler,
    kana_composition_handler: KanaCompositionHandler<OnMemoryDict>,
    kana_converter: Box<KanaBuilder>,
    kana_form_changer: KanaFormChanger
}

/// Rough prototype yet.
///
/// FIXME: どこまでRc (or Arc) で共有できるのか整理する。
#[derive(Debug)]
struct CskkState {
    input_mode: InputMode,
    composition_mode: CompositionMode,
    // 入力文字で、かな確定済みでないものすべて
    pre_conversion: Vec<char>,
    // 変換辞書のキーとなる部分。送りなし変換の場合はconverted_kana_to_composite と同じ。送りあり変換時には加えてconverted_kana_to_okuriの一文字目の子音や'>'付き。Abbrebiation変換の場合kana-convertされる前の入力など
    raw_to_composite: String,
    // 未確定入力をInputモードにあわせてかな変換したもののうち、漢字の読み部分。convertがあるInputMode時のみ使用
    converted_kana_to_composite: String,
    // 未確定入力をInputモードにあわせてかな変換したもののうち、おくり仮名部分。convertがあるInputMode時のみ使用
    converted_kana_to_okuri: String,
    // 入力を漢字変換した現在の選択肢。送り仮名含む。
    composited: String,
    // 確定済み入力列。pollされた時に渡してflushされるもの。
    confirmed: String,
    // 変換中の選択肢
    //composition_candidates: &Vec<Arc<Candidate>>,
    // 変換中の選択肢のうち、どれをさしているか
    selection_pointer: usize,
}

// TODO: ueno/libskkのskk_context_newのようにDict[]を指定できるようにする。
/// Returns newly allocated CSKKContext.
/// It is caller's responsibility to retain and free it.
#[no_mangle]
pub extern "C" fn create_new_context() -> Box<CskkContext> {
    Box::new(CskkContext::new(InputMode::Hiragana, CompositionMode::Direct))
}

/// Reset the context
#[no_mangle]
pub extern "C" fn skk_context_reset(context: &mut CskkContext) {
    // TODO: Flush all the state stack after implementing the register mode
    // TODO: あとまわし。他のテストがこけはじめたらちゃんと実装する。
    context.poll_output();
    context.reset_unconverted();
    context.reset_carry_over();
}

/// Test purpose
/// # Safety
///
/// This function must be called by a valid C string terminated by a NULL.
#[no_mangle]
pub unsafe extern "C" fn skk_context_process_key_events(context: &mut CskkContext, keyevents_cstring: *mut c_char) -> bool {
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
pub extern "C" fn skk_context_set_composition_mode(context: &mut CskkContext, composition_mode: CompositionMode) {
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
    CString::new(skk_context_poll_output_rs(context)).unwrap().into_raw()
}

/// 現在のoutputをpollingする。
///
pub fn skk_context_poll_output_rs(context: &mut CskkContext) -> String {
    if let Some(str) = context.poll_output() {
        return str;
    }
    "".to_string()
}

/// テスト用途。preedit文字列と同じ内容の文字列を取得する。
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

/// テスト用途。preedit文字列と同じ内容の文字列を取得する。
///
pub fn skk_context_get_preedit_rs(context: &CskkContext) -> String {
    context.get_preedit().unwrap()
}

/// テスト用途。
pub fn skk_context_get_compositon_mode(context: &CskkContext) -> CompositionMode {
    context.current_state().borrow().composition_mode.clone()
}

/// テスト用途。
pub fn skk_context_get_input_mode(context: &CskkContext) -> InputMode {
    context.current_state().borrow().input_mode.clone()
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


impl CskkContext {
    ///
    /// Retrieve and remove the current output string
    ///
    pub fn poll_output(&self) -> Option<String> {
        self.retrieve_output(true)
    }

    pub fn get_preedit(&self) -> Option<String> {
        let converted = self.retrieve_output(false);
        let unconverted = &self.current_state().borrow().pre_conversion;
        let kana_to_composite = &self.current_state().borrow().converted_kana_to_composite;
        let composited = &self.current_state().borrow().composited;

        match self.current_state().borrow().composition_mode {
            CompositionMode::Direct => {
                Some(String::from_iter(unconverted.iter()))
            }
            CompositionMode::PreComposition => {
                Some("▽".to_owned() + &converted.unwrap_or_else(|| "".to_string()) + kana_to_composite)
            }
            CompositionMode::PreCompositionOkurigana => {
                Some("▽".to_owned() + &converted.unwrap_or_else(|| "".to_string()) + kana_to_composite + "*" + &String::from_iter(unconverted.iter()))
            }
            CompositionMode::CompositionSelection => {
                Some("▼".to_owned() + composited)
            }
            _ => {
                // FIXME: putting Direct as _ for match, TODO other modes
                Some("Unimplemented".to_owned())
            }
        }
    }

    ///
    /// 確定済文字を返す。
    /// IM側からのpolling用途でなければ、状態を変えない。
    /// IMからのpollingで出力用途ならば、flushする。
    ///
    fn retrieve_output(&self, is_polling: bool) -> Option<String> {
        let current_state = self.current_state();
        if current_state.borrow().confirmed.is_empty() {
            None
        } else {
            let out = current_state.borrow().confirmed.clone();
            if is_polling {
                current_state.borrow_mut().confirmed.clear();
            }
            Some(out)
        }
    }

    fn append_confirmed_raw_char(&self, result: char) {
        let current_state = self.current_state();
        current_state.borrow_mut().confirmed.push(result);
    }


    fn append_converted(&self, result: &str) {
        let mut current_state = self.current_state().borrow_mut();
        let current_input_mode = &current_state.input_mode;
        let adjusted = &self.kana_form_changer.adjust_kana_string(current_input_mode, &result);
        current_state.confirmed.push_str(adjusted);
    }

    fn append_unconverted(&self, unconv: char) {
        let current_state = self.current_state();
        current_state.borrow_mut().pre_conversion.push(unconv);
    }

    fn append_converted_to_composite(&self, result: &str) {
        let current_state = self.current_state();
        let current_input_mode = current_state.borrow().input_mode.clone();
        let adjusted = &self.kana_form_changer.adjust_kana_string(&current_input_mode, &result);
        current_state.borrow_mut().converted_kana_to_composite.push_str(adjusted);
        current_state.borrow_mut().raw_to_composite.push_str(result);
    }

    fn append_converted_to_okuri(&self, result: &str) {
        let current_state = self.current_state();
        let current_input_mode = current_state.borrow().input_mode.clone();
        let adjusted = &self.kana_form_changer.adjust_kana_string(&current_input_mode, &result);
        current_state.borrow_mut().converted_kana_to_okuri.push_str(adjusted);
    }

    fn append_to_composite_iff_no_preconversion(&self, to_composite_last: char) {
        let mut current_state = self.current_state().borrow_mut();
        if current_state.pre_conversion.is_empty() {
            current_state.raw_to_composite.push_str(&to_composite_last.to_string())
        }
    }

    // to_compositeをconvertedの内容でリセットする。
    fn set_to_composite_to_converted_kana(&self) {
        let mut current_state = self.current_state().borrow_mut();
        current_state.raw_to_composite = current_state.converted_kana_to_composite.clone()
    }

    fn reset_unconverted(&self) {
        let current_state = self.current_state();
        current_state.borrow_mut().pre_conversion = vec![];
    }

    fn set_carry_over(&self, unconv: &[char]) {
        let current_state = self.current_state();
        current_state.borrow_mut().pre_conversion = unconv.to_owned();
    }

    fn set_composition_candidate(&self, kanji: &str) {
        let current_state = self.current_state();
        let okuri = current_state.borrow().converted_kana_to_okuri.to_owned();
        let mut kanji_okuri = kanji.to_owned();
        kanji_okuri.push_str(&okuri);
        current_state.borrow_mut().composited = kanji_okuri;
    }

    fn confirm_current_composition_candidate(&self) {
        let current_state = self.current_state();
        let composited = current_state.borrow().composited.to_owned();
        current_state.borrow_mut().confirmed.push_str(&composited);
        current_state.borrow_mut().composited = "".to_string()
    }

    fn confirm_current_kana_to_composite(&self) {
        let current_state = self.current_state();
        current_state.borrow_mut().pre_conversion = vec![];
        current_state.borrow_mut().converted_kana_to_composite = "".to_string();
        current_state.borrow_mut().converted_kana_to_okuri = "".to_string();
    }

    fn reset_carry_over(&self) -> bool {
        let current_state = self.current_state();
        let do_reset = !current_state.borrow().pre_conversion.is_empty();
        current_state.borrow_mut().pre_conversion = vec![];
        do_reset
    }

    fn set_composition_mode(&self, composition_mode: CompositionMode) {
        let mut current_state = self.current_state().borrow_mut();
        current_state.composition_mode = composition_mode;
        current_state.selection_pointer = 0;
    }

    fn set_input_mode(&self, input_mode: InputMode) {
        let mut current_state = self.current_state().borrow_mut();
        current_state.input_mode = input_mode
    }

    ///
    /// process that key event and change the internal states.
    /// if key_event is not processable by current CSKK state, then return false
    ///
    #[allow(dead_code)]
    pub fn process_key_event(&self, key_event: &KeyEvent) -> bool {
        self.process_key_event_inner(key_event, false)
    }

    fn process_key_event_inner(&self, key_event: &KeyEvent, is_delegated: bool) -> bool {
        let current_state = self.current_state();
        let handler = self.get_handler(&current_state.borrow().input_mode, &current_state.borrow().composition_mode);
        if !handler.can_process(key_event) {
            return false;
        }
        let instructions = handler.get_instruction(key_event, &current_state.borrow(), is_delegated);
        let mut must_delegate = false;
        for instruction in instructions {
            dbg!(&instruction);
            match instruction {
                Instruction::ChangeCompositionMode { composition_mode, delegate } => {
                    self.set_composition_mode(composition_mode);
                    must_delegate = delegate;
                }
                Instruction::ChangeInputMode(input_mode) => {
                    self.set_input_mode(input_mode);
                }
                Instruction::ForceConvertPreConversion => {
                    // I cannot think of other case than single 'n' being orphaned. This implementation would be enough for it.
                    let unprocessed = current_state.borrow().pre_conversion.clone();
                    if unprocessed.len() == 1 && unprocessed[0] == 'n' {
                        self.append_converted(&"ん");
                    }
                }
                Instruction::FlushPreviousCarryOver => {
                    self.reset_unconverted();
                    if self.current_state().borrow().converted_kana_to_composite.is_empty() {
                        return true;
                    }
                }
                Instruction::SetComposition { kanji } => {
                    self.set_composition_candidate(kanji);
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
                    self.confirm_current_kana_to_composite();
                }
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
        // ここまで来たらステート変更等の命令としての処理が済み、入力として処理する状態
        let current_composition_mode = &current_state.borrow().composition_mode.clone();
        let current_input_mode = &current_state.borrow().input_mode.clone();
        match current_composition_mode {
            CompositionMode::CompositionSelection
            => {
                // Do nothing.
            }
            CompositionMode::Direct |
            CompositionMode::PreComposition |
            CompositionMode::PreCompositionOkurigana => {
                match current_input_mode {
                    InputMode::Ascii => {
                        if let Some(key_char) = key_event.get_symbol_char() {
                            match current_composition_mode {
                                CompositionMode::Direct => {
                                    self.append_confirmed_raw_char(key_char);
                                }
                                _ => {
                                    debug!("Unreachable. Ascii should be always in Direct mode.");
                                    return false
                                }
                            }
                        }
                    }
                    InputMode::Zenkaku => {

                    }
                    InputMode::Hiragana |
                    InputMode::Katakana |
                    InputMode::HankakuKatakana => {
                        let unprocessed = &current_state.borrow().pre_conversion.clone();
                        if self.kana_converter.can_continue(key_event, &unprocessed) {
                            let combined_keys = KanaBuilder::combined_key(key_event, unprocessed);
                            match self.kana_converter.convert(&combined_keys) {
                                Some((converted, carry_over)) => {
                                    match current_composition_mode {
                                        CompositionMode::Direct => {
                                            self.append_converted(converted);
                                            self.set_carry_over(carry_over);
                                        }
                                        CompositionMode::PreComposition => {
                                            self.append_converted_to_composite(converted);
                                            self.set_carry_over(carry_over);
                                        }
                                        CompositionMode::PreCompositionOkurigana => {
                                            // 入力単独によらない特殊な遷移で、かな変換の結果によって▽モードから▼モードへ移行する。
                                            self.append_converted_to_okuri(converted);
                                            self.set_carry_over(carry_over);
                                            self.set_composition_mode(CompositionMode::CompositionSelection);
                                            return self.process_key_event_inner(key_event, true);
                                        }
                                        _ => {
                                            debug!("Unreachable.");
                                            return false;
                                        }
                                    }
                                }
                                None => {
                                    // かな変換できる可能性が残るのでFlushはされない
                                    if let Some(key_char) = key_event.get_symbol_char() {
                                        match current_composition_mode {
                                            CompositionMode::Direct |
                                            CompositionMode::PreComposition => {
                                                self.append_unconverted(key_char.to_ascii_lowercase())
                                            }
                                            CompositionMode::PreCompositionOkurigana => {
                                                self.append_to_composite_iff_no_preconversion(key_char.to_ascii_lowercase());
                                                self.append_unconverted(key_char.to_ascii_lowercase());
                                            }
                                            _ => {
                                                debug!("Unreachable.");
                                                return false;
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // "k g" 等かな変換が続けられない
                            self.reset_unconverted();
                            if let Some(key_char) = key_event.get_symbol_char() {
                                match current_composition_mode {
                                    CompositionMode::Direct |
                                    CompositionMode::PreComposition => {
                                        self.append_unconverted(key_char.to_ascii_lowercase())
                                    }
                                    CompositionMode::PreCompositionOkurigana => {
                                        self.set_to_composite_to_converted_kana();
                                        self.append_to_composite_iff_no_preconversion(key_char.to_ascii_lowercase());
                                        self.append_unconverted(key_char.to_ascii_lowercase());
                                    }
                                    _ => {
                                        debug!("Unreachable");
                                        return false;
                                    }
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
        // TODO: 入力として内部では処理しつつunhandledで返す命令は必要か調べる。ueno/libskkのviとの連携関連issueとか読むとわかるか？
        true
    }

    fn current_state(&self) -> &RefCell<CskkState> {
        self.state_stack.last().expect("State stack is empty!")
    }

    ///
    /// Returns if that key event can be processed by current CSKK
    /// Only checking, doesn't change internal states
    /// TODO: maybe not a proper impl for IM? can be replaced with just checking meta of keyevent?
    ///
    #[allow(dead_code)]
    pub fn will_process(&self, key_event: &KeyEvent) -> bool {
        let current_state = self.current_state();
        let handler = self.get_handler(&current_state.borrow().input_mode, &current_state.borrow().composition_mode);
        handler.can_process(key_event)
    }

    fn get_handler<'a>(&'a self, input_mode: &InputMode, composition_mode: &CompositionMode) -> Box<dyn CommandHandler + 'a> {
        // FIXME: this _ => default handler looks error prone
        match input_mode {
            InputMode::Hiragana => match composition_mode {
                CompositionMode::Direct => {
                    Box::new(&self.direct_handler)
                }
                CompositionMode::PreComposition => {
                    Box::new(&self.kana_precomposition_handler)
                }
                CompositionMode::PreCompositionOkurigana => {
                    Box::new(&self.kana_precomposition_handler)
                }
                CompositionMode::CompositionSelection => {
                    Box::new(&self.kana_composition_handler)
                }
                _ => { Box::new(&self.direct_handler) }
            }
            _ => { Box::new(&self.direct_handler) }
        }
    }

    /// Mainly for test purpose, but exposed to test as library.
    fn process_key_events_string(&self, key_event_string: &str) -> bool {
        self.process_key_events(&KeyEvent::deserialize_seq(key_event_string).unwrap())
    }

    /// Mainly for test purpose, but exposed to test as library.
    /// FIXME: Remove this clippy rule allow when parameterize on array length is stable in Rust. maybe 1.51?
    #[allow(clippy::ptr_arg)]
    fn process_key_events(&self, key_event_seq: &KeyEventSeq) -> bool {
        dbg!(key_event_seq);
        for key_event in key_event_seq {
            let processed = self.process_key_event(key_event);
            if !processed {
                dbg!("Key event not processed", key_event);
            }
            dbg!(self.current_state().borrow());
        }
        true
    }

    #[allow(dead_code)]
    pub fn new(input_mode: InputMode,
               composition_mode: CompositionMode) -> Self {
        let kana_direct_handler = DirectModeCommandHandler::new();
        // FIXME: Make ref to kana handler using rental crate or find something more simple.
        let kana_precomposition_handler = KanaPrecompositionHandler::new();
        // FIXME: Make ref to kana converter
        let kana_composition_handler = KanaCompositionHandler::new();
        let kana_converter = Box::new(KanaBuilder::default_converter());

        let mut initial_stack = Vec::new();
        initial_stack.push(RefCell::new(
            CskkState {
                input_mode,
                composition_mode,
                pre_conversion: vec![], // TODO
                raw_to_composite: "".to_string(), // TODO
                converted_kana_to_composite: "".to_string(), // TODO
                converted_kana_to_okuri: "".to_string(), // TODO
                composited: "".to_string(), // TODO
                confirmed: "".to_string(), // TODO
                selection_pointer: 0,
            }));
        Self {
            state_stack: initial_stack,
            direct_handler: kana_direct_handler,
            kana_precomposition_handler,
            kana_composition_handler,
            kana_converter,
            kana_form_changer: KanaFormChanger::default_kanaform_changer()
        }
    }
}

impl<'a> Display for Instruction<'a> {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut Formatter<>) -> fmt::Result {
        match self {
//            Instruction::Abort => {
//                writeln!(f, "Abort")
//            }
            Instruction::ChangeCompositionMode { composition_mode, delegate } => {
                writeln!(f, "ChangeComopositionMode: {:?} (delegate: {})", composition_mode, delegate)
            }
            _ => {
                writeln!(f, "Display-unsupported instruction. This is a TODO.")
            }
        }
    }
}

impl Display for CskkState {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut Formatter<>) -> fmt::Result {
        writeln!(f, r#"{{
            {:?}
            {:?}
            confirmed: {}"#,
                 self.input_mode, self.composition_mode,
                 self.confirmed);
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
    pub fn new_test_state(input_mode: InputMode,
                          composition_mode: CompositionMode,
                          pre_conversion: Vec<char>,
    ) -> Self {
        CskkState {
            input_mode,
            composition_mode,
            pre_conversion,
            raw_to_composite: "".to_string(),
            converted_kana_to_composite: "".to_string(), // TODO
            converted_kana_to_okuri: "".to_string(), // TODO
            composited: "".to_string(), // TODO
            confirmed: "".to_string(), // TODO
            selection_pointer: 0,
        }
    }
}


#[cfg(test)]
mod unit_tests {
    use std::sync::{Once};

    use super::*;

    pub static INIT_SYNC: Once = Once::new();

    // TODO: setup proper debug log and move to lib test
    #[allow(dead_code)]
    fn init() {
        INIT_SYNC.call_once(|| {
            let _ = env_logger::init();
        });
    }

    #[test]
    fn will_process() {
        let cskkcontext = CskkContext::new(
            InputMode::Ascii,
            CompositionMode::Direct,
        );
        let a = KeyEvent::from_str("a").unwrap();
        assert!(cskkcontext.will_process(&a));
        let copy = KeyEvent::from_str("C-c").unwrap();
        assert!(!cskkcontext.will_process(&copy));
    }

    #[test]
    fn process_key_event() {
        let cskkcontext = CskkContext::new(
            InputMode::Ascii,
            CompositionMode::Direct,
        );

        let a = KeyEvent::from_str("a").unwrap();
        let result = cskkcontext.process_key_event(&a);
        assert!(result);
    }

    #[test]
    fn retrieve_output() {
        let cskkcontext = CskkContext::new(
            InputMode::Ascii,
            CompositionMode::Direct,
        );
        let a = KeyEvent::from_str("a").unwrap();
        cskkcontext.process_key_event(&a);
        let cskkcontext = cskkcontext;
        let actual = cskkcontext.retrieve_output(false).unwrap();
        assert_eq!("a", actual);
        let actual = cskkcontext.retrieve_output(true).unwrap();
        assert_eq!("a", actual);
        let after = cskkcontext.retrieve_output(true);
        assert_eq!(None, after);
    }

    #[test]
    fn poll_output() {
        let cskkcontext = CskkContext::new(
            InputMode::Ascii,
            CompositionMode::Direct,
        );
        let a = KeyEvent::from_str("a").unwrap();
        cskkcontext.process_key_event(&a);
        let cskkcontext = cskkcontext;
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("a", actual);
        let after = cskkcontext.poll_output();
        assert_eq!(None, after);
    }

    #[test]
    fn get_preedit() {
        let cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
        );
        let capital_a = KeyEvent::from_str("A").unwrap();
        cskkcontext.process_key_event(&capital_a);
        let actual = cskkcontext.get_preedit().expect(&format!("No preedit. context: {:?}", cskkcontext.current_state().borrow()));
        assert_eq!("▽あ", actual);
    }
}