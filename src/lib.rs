#[macro_use]
extern crate log;

//extern crate libc;
extern crate xkbcommon;
extern crate toml;
extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate bitmask;

use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use xkbcommon::xkb;

mod keyevent;
use keyevent::KeyEvent;
use keyevent::KeyEventSeq;

/// TBD 'Rule' applier
struct InputConverter {
    rule: Rule,
    unprocessed: String,
}

#[derive(Deserialize)]
struct Rule {
    meta: RuleMeta,
    convert: RuleConvert,
}

#[derive(Deserialize)]
struct RuleMeta {
    name: String,
    root: bool,
    description: String,
    import: Vec<String>,
}

#[derive(Deserialize)]
struct RuleConvert {
    command: HashMap<KeyEvent, String>,
    kana: HashMap<KeyEventSeq, Vec<String>>,
}


impl InputConverter {
    pub fn convert() {}
    pub fn reset() {}
    pub fn add() {}
    pub fn output() {}

    pub fn create(filename: &str) -> InputConverter {
        let mut file = File::open(filename).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");

//        let config :Rule = toml::from_str(&contents).expect("toml content error");
        let config :Rule = toml::from_str(&contents).expect("toml content error");


        InputConverter{
            rule: config,
            unprocessed: String::new()
        }
    }


}




/// Rough design prototype yet
pub(crate) enum InputMode {
    //Hiragana { converter: InputConverter{} },
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
    input_mode: InputMode,
    composition_state: CompositionState,
    pre_composition: Option<String>,
}

impl CskkContext {
    ///
    /// Not sure of params yet.
    ///
    fn key_press(&self, keysym: xkb::Keysym, keycode: xkb::Keycode) -> bool {
        match self.composition_state {
            CompositionState::Direct => {
                // Input as is for each mode
                match self.input_mode {
                    InputMode::Hiragana => {
                        //Hiragana.converter.add();
                        return false;
                    }
                    InputMode::Katakana => {
                        return true;
                    }
                    InputMode::HankakuKatakana => {
                        return true;
                    }
                    InputMode::Zenkaku => {
                        return true;
                    }
                    InputMode::Ascii => {
                        return true;
                    }
                }
            }
            _ => {
                return false;
            }
        }
    }


    fn key_release() {}

    fn set_state(&mut self, new_state: CompositionState) {
        self.composition_state = new_state;
    }

    fn set_mode(&mut self, new_mode: InputMode) {
        self.input_mode = new_mode;
    }

    pub fn new(input_mode: InputMode,
               composition_state: CompositionState,
               pre_composition: Option<String>) -> CskkContext {
        CskkContext {
            input_mode,
            composition_state,
            pre_composition,
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
    use super::*;

    extern crate env_logger;

    #[test]
    fn load_rule() {
        env_logger::init();

        let converter = InputConverter::create("src/rule/default.toml");

    }
}