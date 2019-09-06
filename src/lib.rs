#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate enum_display_derive;
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
use crate::input_handler::InputHandler;
use crate::input_handler::kana_composition_handler::KanaCompositionHandler;
use crate::input_handler::kana_direct_handler::KanaDirectHandler;
use crate::input_handler::kana_precomposition_handler::KanaPrecompositionHandler;
use crate::kana_converter::KanaConverter;
use crate::keyevent::KeyEvent;
#[cfg(test)]
use crate::keyevent::KeyEventSeq;
use crate::skk_modes::CompositionMode;
use crate::skk_modes::InputMode;

mod kana_converter;
mod keyevent;
mod input_handler;
mod skk_modes;
mod dictionary;

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
    //ChangeInputMode(InputMode),
    //StackRegisterMode,
    FlushPreviousCarryOver,
    // FIXME: PrecompositionからspaceでCompositionを働かせるためにDelegateを作ったが、Delegate無限ループに陥いらないようにする仕組みがない。
    ChangeCompositionMode { composition_mode: CompositionMode, delegate: bool },
    InsertInput(char),
    InputKanaDirect { converted: &'a str, carry_over: &'a Vec<char> },
    InputKanaPrecomposition { converted: &'a str, carry_over: &'a Vec<char> },
    SetComposition { kanji: &'a str, okuri: Option<&'a str> },
}


/// Rough design prototype yet
///
/// TODO: Rustのstructまわりの一部分mutに変更があったら非mutでstateアクセスしているところを直す
/// FIXME: Handler保持をもうちょっとスマートにしたい
#[allow(dead_code)]
struct CskkContext {
    state_stack: Vec<RefCell<CskkState>>,
    kana_direct_handler: KanaDirectHandler,
    kana_precomposition_handler: KanaPrecompositionHandler,
    kana_composition_handler: KanaCompositionHandler<OnMemoryDict>,
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
    // 未確定入力をInputモードにあわせてかな変換したもののうち、変換する部分。
    converted_kana_to_composite: String,
    // 未確定入力をInputモードにあわせてかな変換したもののうち、おくり仮名部分。
    converted_kana_to_okuri: String,
    // 入力を漢字変換したもの。
    composited: String,
    // 確定済み入力列。pollされた時に渡してflushされるもの。
    confirmed: String,
    // 変換中の選択肢
    //composition_candidates: &Vec<Arc<Candidate>>,
    // 変換中の選択肢のうち、どれをさしているか
    selection_pointer: usize,
}

impl CskkContext {
    // TODO: Write integration test that uses new, will_process, poll_output, process_key_event etc.
//    fn set_mode(&mut self, new_mode: InputMode) {
//        self.input_mode = new_mode;
//    }

    ///
    /// Retrieve and remove the current output string
    ///
    #[allow(dead_code)]
    pub fn poll_output(&self) -> Option<String> {
        self.retrieve_output(true)
    }

    #[allow(dead_code)]
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

    fn append_converted(&self, result: &str) {
        let current_state = self.current_state();
        current_state.borrow_mut().confirmed.push_str(result);
    }

    fn append_unconverted(&self, unconv: char) {
        let current_state = self.current_state();
        current_state.borrow_mut().pre_conversion.push(unconv);
    }

    fn append_converted_to_composite(&self, result: &str) {
        let current_state = self.current_state();
        current_state.borrow_mut().converted_kana_to_composite.push_str(result);
    }

    fn reset_unconverted(&self) {
        let current_state = self.current_state();
        current_state.borrow_mut().pre_conversion = vec![];
    }

    fn set_carry_over(&self, unconv: &[char]) {
        let current_state = self.current_state();
        current_state.borrow_mut().pre_conversion = unconv.to_owned();
    }

    fn set_composition_candidate(&self, kanji: &str, okuri: Option<&str>) {
        let current_state = self.current_state();
        if let Some(okuri_kana) = okuri {
            let mut kanji_okuri = kanji.to_owned();
            kanji_okuri.push_str(okuri_kana);
            current_state.borrow_mut().composited = kanji_okuri;
        } else {
            current_state.borrow_mut().composited = kanji.to_owned();
        }
    }

    // TODO: might not only for test
    #[cfg(test)]
    #[allow(dead_code)]
    fn reset_carry_over(&self) -> bool {
        let current_state = self.current_state();
        let do_reset = !current_state.borrow().pre_conversion.is_empty();
        current_state.borrow_mut().pre_conversion = vec![];
        do_reset
    }

    #[allow(dead_code)]
    fn set_composition_mode(&self, composition_mode: CompositionMode) {
        let mut current_state = self.current_state().borrow_mut();
        current_state.composition_mode = composition_mode;
        current_state.selection_pointer = 0;
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
        let instructions = handler.get_instruction(key_event, &current_state.borrow(), is_delegated);
        let mut is_delegated = false;
        for instruction in instructions {
            debug!("instruction: {}", instruction);
            match instruction {
                Instruction::InputKanaDirect { converted, carry_over } => {
                    self.append_converted(converted);
                    self.set_carry_over(carry_over);
                }
                Instruction::InputKanaPrecomposition { converted, carry_over } => {
                    self.append_converted_to_composite(converted);
                    self.set_carry_over(carry_over)
                }
                Instruction::ChangeCompositionMode { composition_mode, delegate } => {
                    self.set_composition_mode(composition_mode);
                    is_delegated = delegate;
                }
                Instruction::InsertInput(ch) => {
                    self.append_unconverted(ch);
                }

                Instruction::FlushPreviousCarryOver => {
                    self.reset_unconverted();
                }
                Instruction::SetComposition {kanji, okuri} => {
                    self.set_composition_candidate(kanji, okuri);
                }
                _ => {
                    debug!("unimplemented instruction: {}", instruction);
                }
            }
        }
        if is_delegated {
            self.process_key_event_inner(key_event, true)
        } else {
            true
        }
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
        handler.can_process(key_event, &[])
    }

    fn get_handler<'a>(&'a self, input_mode: &InputMode, composition_mode: &CompositionMode) -> Box<dyn InputHandler + 'a> {
        // FIXME: this _ => default handler looks error prone
        match input_mode {
            InputMode::Hiragana => match composition_mode {
                CompositionMode::Direct => {
                    Box::new(&self.kana_direct_handler)
                }
                CompositionMode::PreComposition => {
                    Box::new(&self.kana_precomposition_handler)
                }
                CompositionMode::CompositionSelection => {
                    Box::new(&self.kana_composition_handler)
                }
                _ => { Box::new(&self.kana_direct_handler) }
            }
            _ => { Box::new(&self.kana_direct_handler) }
        }
    }

    #[allow(dead_code)]
    pub fn new(input_mode: InputMode,
               composition_mode: CompositionMode) -> Self {
        let kana_converter = KanaConverter::default_converter();

        let kana_direct_handler = KanaDirectHandler::new(Box::new(kana_converter.clone()));
        // FIXME: Make ref to kana handler using rental crate or find something more simple.
        let kana_precomposition_handler = KanaPrecompositionHandler::new(Box::new(kana_converter.clone()));
        // FIXME: Make ref to kana converter
        let kana_composition_handler = KanaCompositionHandler::new();

        let mut initial_stack = Vec::new();
        initial_stack.push(RefCell::new(
            CskkState {
                input_mode,
                composition_mode,
                pre_conversion: vec![], // TODO
                converted_kana_to_composite: "".to_string(), // TODO
                converted_kana_to_okuri: "".to_string(), // TODO
                composited: "".to_string(), // TODO
                confirmed: "".to_string(), // TODO
                selection_pointer: 0,
            }));
        Self {
            state_stack: initial_stack,
            kana_direct_handler,
            kana_precomposition_handler,
            kana_composition_handler,
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
            Instruction::InputKanaDirect { converted, carry_over } => {
                write!(f, "Input {}, carry over ", converted);
                carry_over.iter().map(|c| write!(f, "{}", c));
                writeln!(f)
            }
            Instruction::ChangeCompositionMode { composition_mode, delegate } => {
                writeln!(f, "ChangeComopositionMode: {} (delegate: {})", composition_mode, delegate)
            }
            _ => {
                writeln!(f, "Display-unsupported instruction. This is a bug.")
            }
        }
    }
}

impl Display for CskkState {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut Formatter<>) -> fmt::Result {
        writeln!(f, r#"{{
            {}
            {}
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
impl CskkContext {
    fn process_key_events(&self, key_event_seq: &KeyEventSeq) -> bool {
        for key_event in key_event_seq {
            let processed = self.process_key_event(key_event);
            if !processed {
                debug!("Key {} not processed", key_event);
            }
            debug!("{}", self.current_state().borrow());
        }
        return true;
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
            converted_kana_to_composite: "".to_string(), // TODO
            converted_kana_to_okuri: "".to_string(), // TODO
            composited: "".to_string(), // TODO
            confirmed: "".to_string(), // TODO
            selection_pointer: 0,
        }
    }
}


#[cfg(test)]
mod tests {
    use std::sync::{Once, ONCE_INIT};

    use super::*;

    pub static INIT_SYNC: Once = ONCE_INIT;

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
        assert_eq!("あ", actual);
        let actual = cskkcontext.retrieve_output(true).unwrap();
        assert_eq!("あ", actual);
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
        assert_eq!("あ", actual);
        let after = cskkcontext.poll_output();
        assert_eq!(None, after);
    }

    #[test]
    fn basic_hiragana() {
        init();
        let cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
        );

        let a_gyou = KeyEvent::deserialize_seq("a i u e o").unwrap();
        cskkcontext.process_key_events(&a_gyou);
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("あいうえお", actual);
    }

    #[test]
    fn skip_on_impossible_hiragana() {
        init();
        let cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
        );

        let a_gyou = KeyEvent::deserialize_seq("b n y a").unwrap();
        cskkcontext.process_key_events(&a_gyou);
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("にゃ", actual);
    }

    #[test]
    fn simple_composition() {
        init();
        let cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
        );

        let pre_love = KeyEvent::deserialize_seq("A i").unwrap();
        cskkcontext.process_key_events(&pre_love);
        let actual_preedit = cskkcontext.get_preedit().unwrap();
        assert_eq!("▽あい", actual_preedit);
    }

    #[test]
    fn get_preedit() {
        let cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
        );
        let capital_a = KeyEvent::from_str("A").unwrap();
        cskkcontext.process_key_event(&capital_a);
        let actual = cskkcontext.get_preedit().expect(&format!("No preedit. context: {}", cskkcontext.current_state().borrow()));
        assert_eq!("▽あ", actual);
    }

    #[test]
    fn get_preedit_precompostion_from_mid() {
        let cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
        );
        let a = KeyEvent::from_str("k").unwrap();
        cskkcontext.process_key_event(&a);
        let actual = cskkcontext.get_preedit().expect(&format!("No preedit after k. context: {}", cskkcontext.current_state().borrow()));
        assert_eq!("k", actual);

        let large_a = KeyEvent::from_str("I").unwrap();
        cskkcontext.process_key_event(&large_a);
        let after = cskkcontext.get_preedit().unwrap();
        assert_eq!("▽き", after, "context: {}", cskkcontext.current_state().borrow());
    }

    #[test]
    fn basic_henkan() {
        let cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
        );
        let pre_love = KeyEvent::deserialize_seq("A i space").unwrap();
        cskkcontext.process_key_events(&pre_love);
        let actual = cskkcontext.get_preedit().expect(&format!("No preedit. context: {}", cskkcontext.current_state().borrow()));
        assert_eq!(actual, "▼愛");
    }
}