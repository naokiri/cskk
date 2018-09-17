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
    unprocessed: KeyEventSeq,
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
    kana: HashMap<KeyEventSeq, (Unprocessed, Converted)>,
}

enum Command {
    Abort,
}

enum Instruction {
    Operation { operation: Command },
    Kana { kana: String, keyevent: String },
}

type Unprocessed = String; // Mikakutei string TODO: Might not String but KeyEventRep? Fix when implement here.
type Converted = String; // Kakutei string

impl InputConverter {
    pub fn convert() {}
    pub fn reset() {}

    ///
    /// Consume one KeyEvent and make the next unconverted string and converted string.
    ///
    pub fn process_key_event(&mut self, key_event: KeyEvent, mut unprocessed: &KeyEventSeq) -> Result<(Option<Unprocessed>, Option<Converted>), String> {
        let convert = &self.rule.convert;

        match convert.command.get(&key_event) {
            Some(x) => {
                match x.as_ref() {
                    "abort" => {
                        // TODO: abort and input the unprocessed key as is
                        ;
                    }
                    _ => {
                        ;// Do nothing
                    }
                }
            }
            None => {
                ;// Do nothing
            }
        }

        self.unprocessed.append(key_event.clone());

        return Err("".to_string());
    }

    fn keyevent_to_instruction(keyevent: KeyEvent, convert: RuleConvert) -> Instruction {
        Instruction::Operation { operation: Command::Abort }
    }

    pub fn output() {}

    pub fn create(filename: &str) -> InputConverter {
        let mut file = File::open(filename).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");

        let config: Rule = toml::from_str(&contents).expect("toml content error");

        InputConverter {
            rule: config,
            unprocessed: KeyEventSeq::new(),
        }
    }
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
    Abbreviation, // Sub-mode of PreComposition in ddskk
    Register(Box<CompositionState>), // Sub-mode of CompositionSelection in ddskk
}

/// Rough design prototype yet
pub(crate) struct CskkContext {
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
    fn key_press(&self, keysym: xkb::Keysym, keycode: xkb::Keycode) -> bool {
        match self.composition_state {
            CompositionState::Direct => {
                // Input as is for each mode
                match self.input_mode {
                    InputMode::Hiragana => {
                        // Hiragana.converter.add();
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
            unconverted: "".to_string(), // TODO
            converted: "".to_string(), // TODO
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