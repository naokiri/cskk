#[macro_use]
extern crate bitmask;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
//extern crate libc;
extern crate xkbcommon;

use keyevent::KeyEvent;
use keyevent::KeyEventSeq;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use xkbcommon::xkb;

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
pub(crate) enum InputMode {
    Hiragana,
    Katakana,
    HankakuKatakana,
    Zenkaku,
    Ascii,
}

/// Rough design prototype yet
pub(crate) enum CompositionState {
    Direct,
    PreComposition,
    CompositionSelection,
    // Sub-mode of PreComposition in ddskk
    Abbreviation,
    // Sub-mode of CompositionSelection in ddskk
    Register(Box<CompositionState>),
}

/// Rough design prototype yet
pub(crate) struct CskkContext {
    state_stack: Vec<CskkState>,
}

struct CskkState {
    input_mode: InputMode,
    composition_state: CompositionState,
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

    pub fn new(input_mode: InputMode,
               composition_state: CompositionState,
               pre_composition: Option<String>) -> CskkContext {
        let mut initial_stack = Vec::new();
        initial_stack.push(CskkState {
            input_mode,
            composition_state,
            pre_composition,
            unconverted: "".to_string(), // TODO
            converted: "".to_string(), // TODO
        });
        CskkContext {
            state_stack: initial_stack,
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
            CompositionState::Direct,
            None,
        ))
//    mem::transmute(initial_context)
}

#[cfg(test)]
mod tests {
    use keyevent::KeyEvent;
    use super::*;

    extern crate env_logger;


}