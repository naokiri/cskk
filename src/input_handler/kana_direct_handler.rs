use xkbcommon::xkb;

use crate::{CskkState, Instruction};
use crate::input_handler::InputHandler;
use crate::kana_converter::KanaConverter;
use crate::keyevent::KeyEvent;
use crate::keyevent::SkkKeyModifier;
use crate::skk_modes::CompositionMode;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct KanaDirectHandler {
    kana_converter: Box<KanaConverter>,
}

impl KanaDirectHandler {
    pub fn new(kana_converter: Box<KanaConverter>) -> Self {
        KanaDirectHandler {
            kana_converter
        }
    }
}

impl InputHandler for KanaDirectHandler {
    fn can_process(&self, key_event: &KeyEvent, _unprocessed: &[char]) -> bool {
        let modifier = key_event.get_modifier();
        if modifier.contains(SkkKeyModifier::CONTROL) {
            return false;
        }

        let symbol = key_event.get_symbol();
        xkb::keysyms::KEY_a <= symbol && symbol <= xkb::keysyms::KEY_asciitilde
    }

    fn get_instruction(&self, key_event: &KeyEvent, current_state: &CskkState, _is_delegated: bool) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let ref unprocessed = *current_state.pre_conversion;
        // TODO: reset to ascii direct mode on l

        let symbol = key_event.get_symbol();
        if xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z {
            instructions.push(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::PreComposition, delegate: false });
        }

        if self.kana_converter.can_continue(key_event, &unprocessed) {
            let key = KanaConverter::combined_key(key_event, unprocessed);

            match self.kana_converter.convert(&key) {
                Some((converted, carry_over)) => {
                    if xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z {
                        instructions.push(Instruction::InputKanaPrecomposition { converted, carry_over });
                    } else {
                        instructions.push(Instruction::InputKanaDirect { converted, carry_over });
                    }
                }
                None => {
                    if let Some(key_char) = key_event.get_symbol_char() {
                        instructions.push(Instruction::InsertInput(key_char))
                    }
                }
            }
        } else {
            instructions.push(Instruction::FlushPreviousCarryOver);
            if let Some(key_char) = key_event.get_symbol_char() {
                instructions.push(Instruction::InsertInput(key_char))
            }
        }
        instructions
    }
}

#[cfg(test)]
impl KanaDirectHandler {
    fn test_handler() -> Self {
        let kana_converter = KanaConverter::default_converter();

        KanaDirectHandler {
            kana_converter: Box::new(kana_converter),
        }
    }
}

#[cfg(test)]
mod tests {
    use xkbcommon::xkb::keysyms;

    use crate::InputMode;
    use crate::keyevent::SkkKeyModifier;

    use super::*;

    fn init() {
        crate::tests::INIT_SYNC.call_once(|| {
            let _ = env_logger::init();
        });
    }

    fn get_unprocessed_state(unprocessed: Vec<char>) -> CskkState {
        CskkState::new_test_state(InputMode::Hiragana,
                                  CompositionMode::Direct,
                                  unprocessed)
    }

    #[test]
    fn can_process_single() {
        let handler = KanaDirectHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec![]);
        assert!(result);
    }

    #[test]
    fn can_process_intermediate() {
        let handler = KanaDirectHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("k").unwrap(), &vec![]);
        assert!(result);
    }

    #[test]
    fn handler_works() {
        let handler = KanaDirectHandler::test_handler();

        let result = handler.can_process(&KeyEvent::from_keysym(keysyms::KEY_apostrophe, SkkKeyModifier::NONE), &vec!['n']);
        assert!(!result);

        let result = handler.can_process(&KeyEvent::from_str("b").unwrap(), &vec![]);
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("y").unwrap(), &vec!['b']);
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec!['b', 'y']);
        assert!(result);
    }

    #[test]
    fn get_instruction() {
        let handler = KanaDirectHandler::test_handler();

        let result = handler.get_instruction(&KeyEvent::from_str("b").unwrap(), &get_unprocessed_state(vec![]),false);
        assert_eq!(Instruction::InsertInput('b'), result[0]);

        let result = handler.get_instruction(&KeyEvent::from_str("n").unwrap(), &get_unprocessed_state(vec!['b']), false);
        assert_eq!(Instruction::FlushPreviousCarryOver, result[0]);
        assert_eq!(Instruction::InsertInput('n'), result[1]);

        let result = handler.get_instruction(&KeyEvent::from_str("y").unwrap(), &get_unprocessed_state(vec!['n']), false);
        assert_eq!(Instruction::InsertInput('y'), result[0]);

        let result = handler.get_instruction(&KeyEvent::from_str("a").unwrap(), &get_unprocessed_state(vec!['b', 'y']), false);
        assert_eq!(Instruction::InputKanaDirect { converted: &"びゃ", carry_over: &Vec::with_capacity(0) }, result[0]);
    }

    #[test]
    fn switch_mode() {
        init();
        let handler = KanaDirectHandler::test_handler();
        let key_event = KeyEvent::from_str("B").unwrap();
        assert!(key_event.get_symbol() <= xkb::keysyms::KEY_asciitilde);
        assert!(xkb::keysyms::KEY_A <= key_event.get_symbol());

        let result = handler.get_instruction(&key_event, &get_unprocessed_state(vec![]), false);
        assert_eq!(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::PreComposition, delegate: false }, result[0]);
    }
}