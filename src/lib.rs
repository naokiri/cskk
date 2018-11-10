#[macro_use]
extern crate bitmask;
//#[macro_use]
//extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
//extern crate libc;
extern crate xkbcommon;

use keyevent::KeyEvent;

use std::collections::HashMap;

//use std::mem;

mod keyevent;

#[derive(Deserialize)]
struct RuleMeta {
    name: String,
    root: bool,
    description: String,
    import: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum Command {
    Abort,
}

enum Instruction {
    Operation { operation: Command },
    Input { converted: String, unconverted: String },
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
pub(crate) struct CskkContext {
    state_stack: Vec<CskkState>,
    handler: AHandler,
}

struct CskkState {
    input_mode: InputMode,
    composition_mode: CompositionMode,
    pre_composition: Option<String>,
    unconverted: String,
    converted: String,
}

struct AHandler {
    process_list: HashMap<KeyEvent, String>,
}

impl AHandler {
    pub fn new() -> AHandler {
        let mut process_list = HashMap::new();
        process_list.insert(KeyEvent::from_str("a").unwrap(),"あ".to_string());
        AHandler {
            process_list,
        }
    }

    pub fn can_process(&self, key_event: &KeyEvent) -> bool {
        self.process_list.contains_key(key_event)
    }
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
    /// Returns if that key event can be processed by current IM
    ///
    pub fn will_process(&self, key_event: &KeyEvent) -> bool {
        let current_state = self.state_stack.last().unwrap();
        let handler = self.get_handler(&current_state.input_mode, &current_state.composition_mode);
        handler.can_process(key_event)
    }

    fn get_handler(&self, _input_mode: &InputMode, _composition_mode: &CompositionMode ) -> &AHandler {
        return &self.handler;
    }

    pub fn new(input_mode: InputMode,
               composition_mode: CompositionMode,
               pre_composition: Option<String>) -> CskkContext {
        let handler = AHandler::new();

        let mut initial_stack = Vec::new();
        initial_stack.push(CskkState {
            input_mode,
            composition_mode,
            pre_composition,
            unconverted: "".to_string(), // TODO
            converted: "".to_string(), // TODO
        });
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


//    #[test]
//    fn process_key_event() {
//        skkcontext.process_key_event("a");
//        let actual = skkcontext.poll_output();
//        assert_eq!("あ", actual);
//    }
}