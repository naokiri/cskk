#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate enum_display_derive;
extern crate env_logger;
#[cfg(test)]
#[macro_use]
extern crate log;
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

use crate::input_handler::InputHandler;
use crate::input_handler::kana_handler::KanaHandler;
use crate::input_handler::kana_precomposition_handler::KanaPrecompositionHandler;
use crate::keyevent::KeyEvent;
#[cfg(test)]
use crate::keyevent::KeyEventSeq;
use crate::skk_modes::CompositionMode;
use crate::skk_modes::InputMode;

mod keyevent;
mod input_handler;
mod skk_modes;

#[derive(Deserialize)]
struct RuleMeta {
    name: String,
    root: bool,
    description: String,
    import: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Instruction<'a> {
    Abort,
    //ChangeInputMode(InputMode),
    //StackRegisterMode,
    FlushUnconverted { new_start: char },
    InputKana { converted: &'a str, carry_over: &'a Vec<char> },
    StartComposition { converted: &'a str, carry_over: &'a Vec<char> },
}


/// Rough design prototype yet
///
/// TODO: Rustのstructまわりの一部分mutに変更があったら非mutでstateアクセスしているところを直す
struct CskkContext {
    state_stack: Vec<RefCell<CskkState>>,
    //handlers: HashMap<InputMode, HashMap<CompositionMode, InputHandlerType<KanaHandler>>>,
    kana_handler: KanaHandler,
    kana_precomposition_handler: KanaPrecompositionHandler,
}

#[derive(Debug)]
struct CskkState {
    input_mode: InputMode,
    composition_mode: CompositionMode,
    unconverted: Vec<char>,
    converted: String,
}

impl CskkContext {
    fn key_release() {}

//    fn set_mode(&mut self, new_mode: InputMode) {
//        self.input_mode = new_mode;
//    }

    ///
    /// Retrieve and remove the current output string
    ///
    pub fn poll_output(&self) -> Option<String> {
        self.retrieve_output(true)
    }

    pub fn get_preedit(&self) -> Option<String> {
        self.retrieve_output(false)
    }

    fn retrieve_output(&self, is_polling: bool) -> Option<String> {
        let current_state = self.current_state();
        if current_state.borrow().composition_mode == CompositionMode::Direct {
            if current_state.borrow().converted.is_empty() {
                None
            } else {
                let out = current_state.borrow().converted.clone();
                if is_polling {
                    current_state.borrow_mut().converted.clear();
                }
                Some(out)
            }
        } else {
            None
        }
    }

    fn append_input(&self, result: &str) {
        let current_state = self.current_state();
        current_state.borrow_mut().converted.push_str(result);
    }

    fn set_carry_over(&self, unconv: &[char]) {
        let current_state = self.current_state();
        current_state.borrow_mut().unconverted = unconv.to_owned();
    }

    fn append_unconverted(&self, unconv: char) {
        let current_state = self.current_state();
        current_state.borrow_mut().unconverted.push(unconv);
    }

    fn reset_carry_over(&self) -> bool {
        let current_state = self.current_state();
        let do_reset = !current_state.borrow().unconverted.is_empty();
        current_state.borrow_mut().unconverted = vec![];
        do_reset
    }

    fn move_to_pre_composition(&self) {
        let mut current_state = self.current_state().borrow_mut();
        current_state.composition_mode = CompositionMode::PreComposition;
    }

    ///
    /// process that key event and change the internal states.
    /// if key_event is not processable by current CSKK state, then return false
    ///
    pub fn process_key_event(&self, key_event: &KeyEvent) -> bool {
        let current_state = self.current_state();
        let handler = self.get_handler(&current_state.borrow().input_mode, &current_state.borrow().composition_mode);
        let instruction = handler.get_instruction(key_event, &current_state.borrow().unconverted);
        match instruction {
            Some(Instruction::InputKana { converted, carry_over }) => {
                self.append_input(converted);
                self.set_carry_over(carry_over);
                true
            }
            Some(Instruction::StartComposition { converted, carry_over }) => {
                self.append_input(converted);
                self.set_carry_over(carry_over);
                self.move_to_pre_composition();
                true
            }
            Some(Instruction::FlushUnconverted { new_start }) => {
                self.append_unconverted(new_start);
                true
            }
            _ => {
                self.flush_and_retry(key_event)
            }
        }
    }

    fn flush_and_retry(&self, key_event: &KeyEvent) -> bool {
        if self.reset_carry_over() {
            self.process_key_event(key_event)
        } else {
            false
        }
    }

    #[cfg(test)]
    fn process_key_events(&mut self, key_event_seq: &KeyEventSeq) -> bool {
        for key_event in key_event_seq {
            if !self.process_key_event(key_event) {
                self.reset_carry_over();
            }
            debug!("{}", self.current_state().borrow());
        }
        return true;
    }

    fn current_state(&self) -> &RefCell<CskkState> {
        self.state_stack.last().expect("State stack is empty!")
    }

    ///
    /// Returns if that key event can be processed by current CSKK
    /// Only checking, doesn't change internal states
    /// TODO: maybe not a proper impl for IM? can be replaced with just checking meta of keyevent?
    ///
    pub fn will_process(&self, key_event: &KeyEvent) -> bool {
        let current_state = self.current_state();
        let handler = self.get_handler(&current_state.borrow().input_mode, &current_state.borrow().composition_mode);
        handler.can_process(key_event, &[])
    }


    fn get_handler<'a>(&'a self, input_mode: &InputMode, composition_mode: &CompositionMode) -> Box<InputHandler + 'a> {
        Box::new(&self.kana_handler)
    }

    pub fn new(input_mode: InputMode,
               composition_mode: CompositionMode,
               pre_composition: Option<String>) -> Self {
        let kana_handler = KanaHandler::default_handler();
        // TODO: Make ref to kana handler using rental crate or find something more simple.
        let kana_precomposition_handler = KanaPrecompositionHandler::new(Box::new(kana_handler.clone()));

        let mut initial_stack = Vec::new();
        initial_stack.push(RefCell::new(
            CskkState {
                input_mode,
                composition_mode,
                unconverted: vec![], // TODO
                converted: "".to_string(), // TODO
            }));
        Self {
            state_stack: initial_stack,
            kana_handler: kana_handler,
            kana_precomposition_handler,
        }
    }
}

impl Display for CskkState {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut Formatter<>) -> fmt::Result {
        writeln!(f, r#"{{
            {}
            {}
            converted: {}
        "#, self.input_mode, self.composition_mode, self.converted);
        write!(f, "  unconverted:'");
        for c in self.unconverted.to_vec() {
            write!(f, "{}", c);
        }
        writeln!(f);
        writeln!(f, "}}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Once, ONCE_INIT};

    use super::*;

    static INIT_SYNC: Once = ONCE_INIT;

    pub fn init() {
        INIT_SYNC.call_once(|| {
            let _ = env_logger::init();
        });
    }

    #[test]
    fn will_process() {
        let cskkcontext = CskkContext::new(
            InputMode::Ascii,
            CompositionMode::Direct,
            None,
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
            None,
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
            None,
        );
        let a = KeyEvent::from_str("a").unwrap();
        cskkcontext.process_key_event(&a);
        let mut cskkcontext = cskkcontext;
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
            None,
        );
        let a = KeyEvent::from_str("a").unwrap();
        cskkcontext.process_key_event(&a);
        let mut cskkcontext = cskkcontext;
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("あ", actual);
        let after = cskkcontext.poll_output();
        assert_eq!(None, after);
    }

    #[test]
    fn basic_hiragana() {
        init();
        let mut cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
            None,
        );

        let a_gyou = KeyEvent::deserialize_seq("a i u e o").unwrap();
        cskkcontext.process_key_events(&a_gyou);
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("あいうえお", actual);
    }

    #[test]
    fn skip_on_impossible_hiragana() {
        init();
        let mut cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
            None,
        );

        let a_gyou = KeyEvent::deserialize_seq("b n y a").unwrap();
        cskkcontext.process_key_events(&a_gyou);
        let actual = cskkcontext.poll_output().unwrap();
        assert_eq!("にゃ", actual);
    }

    #[test]
    fn simple_composition() {
        init();
        let mut cskkcontext = CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
            None,
        );

        let pre_love = KeyEvent::deserialize_seq("A i").unwrap();
        cskkcontext.process_key_events(&pre_love);
        let actual_output = cskkcontext.poll_output().unwrap();
        assert_eq!("", actual_output);
        let cskkcontext = cskkcontext;
        let actual_preedit = cskkcontext.get_preedit().unwrap();
        assert_eq!("▽あい", actual_preedit);
    }

    #[test]
    fn get_preedit() {
        let cskkcontext = CskkContext::new(
            InputMode::Ascii,
            CompositionMode::Direct,
            None,
        );
        let a = KeyEvent::from_str("a").unwrap();
        cskkcontext.process_key_event(&a);
        let actual = cskkcontext.get_preedit().unwrap();
        assert_eq!("", actual);

        let large_a = KeyEvent::from_str("A").unwrap();
        cskkcontext.process_key_event(&large_a);
        let after = cskkcontext.get_preedit().unwrap();
        assert_eq!("▽あ", actual);
    }

    #[test]
    fn get_preedit_precompostion_from_mid() {
        let cskkcontext = CskkContext::new(
            InputMode::Ascii,
            CompositionMode::Direct,
            None,
        );
        let a = KeyEvent::from_str("k").unwrap();
        cskkcontext.process_key_event(&a);
        let actual = cskkcontext.get_preedit().unwrap();
        assert_eq!("", actual);

        let large_a = KeyEvent::from_str("I").unwrap();
        cskkcontext.process_key_event(&large_a);
        let after = cskkcontext.get_preedit().unwrap();
        assert_eq!("▽き", actual);
    }
}