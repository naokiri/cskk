#[macro_use]
extern crate bitflags;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate xkbcommon;
extern crate sequence_trie;

use handler::Handler;
use keyevent::KeyEvent;
use keyevent::KeyEventSeq;
use std::cell::RefCell;
use std::collections::HashMap;

//use std::mem;

mod keyevent;
mod handler;

#[derive(Deserialize)]
struct RuleMeta {
    name: String,
    root: bool,
    description: String,
    import: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Command {
    Abort,
}

#[derive(Debug)]
pub(crate) enum Instruction<'a> {
    Operation { operation: &'a Command },
    //Input { converted: &'a String },
    Input { converted: &'a str, unconverted: &'a Vec<char> },
}

/// Rough design prototype yet
/// SKKの入力モード
/// DDSKK 16.2 マニュアル 4.2 に依る
pub(crate) enum InputMode {
    // かなモード
    Hiragana,
    // カナモード
    Katakana,
    // JIS X 0201 カナ、 いわゆる半角カナ。 DDSKKでは独立したモード扱いではないので実装未定
    HankakuKatakana,
    // 全英モード
    Zenkaku,
    // アスキーモード
    Ascii,
}

/// Rough design prototype yet
/// SKKの変換モード
/// DDSKK 16.2 マニュアル 4.3 に依る
pub(crate) enum CompositionMode {
    // ■モード
    Direct,
    // ▽モード
    PreComposition,
    // ▼モード
    CompositionSelection,
    // SKK abbrev mode: Sub-mode of PreComposition
    Abbreviation,
    // Sub-mode of CompositionSelection
    Register(Box<CompositionMode>),
}

/// Rough design prototype yet
///
/// TODO: Rustのstructまわりの一部分mutに変更があったら非mutでstateアクセスしているところを直す
pub(crate) struct CskkContext {
    state_stack: Vec<RefCell<CskkState>>,
    handler: Handler,
}

struct CskkState {
    input_mode: InputMode,
    composition_mode: CompositionMode,
    pre_composition: Option<String>,
    unconverted: String,
    converted: String,
}


impl CskkContext {
    ///
    /// Not sure of params yet.
    ///
//    fn key_press(&self, keysym: xkb::Keysym, keycode: xkb::Keycode) -> bool {
//        match self.composition_state {
//            CompositionState::Direct => {
//                // Input as is for each mode
//                match self.input_mode {
//                    InputMode::Hiragana => {
//                        // Hiragana.converter.add();
//                        return false;
//                    }
//                    InputMode::Katakana => {
//                        return true;
//                    }
//                    InputMode::HankakuKatakana => {
//                        return true;
//                    }
//                    InputMode::Zenkaku => {
//                        return true;
//                    }
//                    InputMode::Ascii => {
//                        return true;
//                    }
//                }
//            }
//            _ => {
//                return false;
//            }
//        }
//    }


    fn key_release() {}

//    fn set_state(&mut self, new_state: CompositionState) {
//        self.composition_state = new_state;
//    }

//    fn set_mode(&mut self, new_mode: InputMode) {
//        self.input_mode = new_mode;
//    }

    ///
    /// Retrieve and remove the current output string
    /// 
    fn poll_output(&mut self) -> Option<String> {
        let current_state = self.current_state();
        if current_state.borrow().converted.is_empty() {
            None
        } else {
            let out = current_state.borrow().converted.clone();
            current_state.borrow_mut().converted.clear();
            Some(out)
        }
    }

    fn append_input(&self, result: &str) {
        let current_state = self.current_state();
        current_state.borrow_mut().converted.push_str(result);
    }

    ///
    /// process that key event and change the internal states.
    /// if key_event is not processable by current CSKK state, then return false
    ///
    pub fn process_key_event(&self, key_event: &KeyEvent) -> bool {
        if !self.will_process(key_event) {
            false
        } else {
            let current_state = self.current_state();
            let handler = self.get_handler(&current_state.borrow().input_mode, &current_state.borrow().composition_mode);
            let instruction = handler.get_instruction(key_event, &vec![]);
            match instruction {
                Some(Instruction::Input { converted, unconverted: _ }) => {
                    self.append_input(converted);
                    true
                }
                Some(Instruction::Operation { .. }) => {
                    true
                }
                None => {
                    false
                }
            }
        }
    }

    #[cfg(test)]
    fn process_key_events(&mut self, key_event_seq: &KeyEventSeq) -> bool {
        for key_event in key_event_seq {
            if !self.process_key_event(key_event) {
                return false;
            }
        }
        return true;
    }

    fn current_state(&self) -> &RefCell<CskkState> {
        self.state_stack.last().expect("State stack is empty!")
    }

    ///
    /// Returns if that key event can be processed by current CSKK
    /// Only checking, doesn't change internal states
    ///
    pub fn will_process(&self, key_event: &KeyEvent) -> bool {
        let current_state = self.current_state();
        let handler = self.get_handler(&current_state.borrow().input_mode, &current_state.borrow().composition_mode);
        handler.can_process(key_event, &vec![])
    }


    fn get_handler(&self, _input_mode: &InputMode, _composition_mode: &CompositionMode) -> &Handler {
        &self.handler
    }

    pub fn new(input_mode: InputMode,
               composition_mode: CompositionMode,
               pre_composition: Option<String>) -> CskkContext {
        let handler = Handler::defalt_handler();

        let mut initial_stack = Vec::new();
        initial_stack.push(RefCell::new(
            CskkState {
                input_mode,
                composition_mode,
                pre_composition,
                unconverted: "".to_string(), // TODO
                converted: "".to_string(), // TODO
            }));
        CskkContext {
            state_stack: initial_stack,
            handler,
        }
    }
}

/// Prototype just checking ffi yet.
///
/// Makes initial context and returns the pointer to the context.
/// This library does not own the structure. It's callers' responsibility to retain them.
///
/// # parameter
/// opt_logger: Rust logger. If no logging, null.
///
fn new_context() -> Box<CskkContext> {
    Box::new(
        CskkContext::new(
            InputMode::Hiragana,
            CompositionMode::Direct,
            None,
        ))
//    mem::transmute(initial_context)
}

#[cfg(test)]
mod tests {
    use keyevent::KeyEvent;
    use super::*;

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
}