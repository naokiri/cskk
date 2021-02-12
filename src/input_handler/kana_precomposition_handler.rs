use xkbcommon::xkb;

use crate::{CskkState, Instruction};
use crate::input_handler::InputHandler;
use crate::kana_converter::KanaConverter;
use crate::keyevent::KeyEvent;
use crate::keyevent::SkkKeyModifier;
use crate::skk_modes::CompositionMode;

#[derive(Debug)]
pub(crate) struct KanaPrecompositionHandler {
    kana_converter: Box<KanaConverter>,
}

impl KanaPrecompositionHandler {
    pub fn new(kana_converter: Box<KanaConverter>) -> Self {
        KanaPrecompositionHandler {
            kana_converter
        }
    }
}

impl InputHandler for KanaPrecompositionHandler {
    fn can_process(&self, key_event: &KeyEvent, _unprocessed: &[char]) -> bool {
        let symbol = key_event.get_symbol();
        (0x0020..0x007F).contains(&symbol)
    }

    fn get_instruction(&self, key_event: &KeyEvent, current_state: &CskkState, _is_delegated: bool) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let unprocessed = &*current_state.pre_conversion;
        // TODO: ▽ひらがな + 'q' => ヒラガナ
        // TODO: ▽ひらがな + Ctrl-G => FlushAbort

        let symbol = key_event.get_symbol();
        let is_capital = xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z && key_event.get_modifier().contains(SkkKeyModifier::SHIFT);
        if symbol == xkb::keysyms::KEY_space {
            instructions.push(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::CompositionSelection, delegate: true });
            return instructions;
        } else if is_capital {
            instructions.push(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::CompositionSelection, delegate: false });
        }

        if self.kana_converter.can_continue(key_event, &unprocessed) {
            let key = KanaConverter::combined_key(key_event, unprocessed);
            match self.kana_converter.convert(&key) {
                Some((converted, carry_over)) => {
                    instructions.push(Instruction::InputKanaPrecomposition { converted, carry_over })
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
impl KanaPrecompositionHandler {
    fn test_handler() -> Self {
        let kana_converter = KanaConverter::default_converter();

        KanaPrecompositionHandler {
            kana_converter: Box::new(kana_converter),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::InputMode;

    use super::*;

    fn get_unprocessed_state(unprocessed: Vec<char>) -> CskkState {
        CskkState::new_test_state(InputMode::Hiragana,
                                  CompositionMode::PreComposition,
                                  unprocessed)
    }

    #[test]
    fn can_process_single() {
        let handler = KanaPrecompositionHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec![]);
        assert!(result);
    }

    #[test]
    fn can_process_intermediate() {
        let handler = KanaPrecompositionHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("k").unwrap(), &vec![]);
        assert!(result);
    }

    #[test]
    fn get_instruction() {
        let handler = KanaPrecompositionHandler::test_handler();

        let result = handler.get_instruction(&KeyEvent::from_str("b").unwrap(), &get_unprocessed_state(vec![]), false);
        assert_eq!(Instruction::InsertInput('b'), result[0]);

        let result = handler.get_instruction(&KeyEvent::from_str("n").unwrap(), &get_unprocessed_state(vec!['b']), false);
        assert_eq!(Instruction::FlushPreviousCarryOver, result[0]);
        assert_eq!(Instruction::InsertInput('n'), result[1]);

        let result = handler.get_instruction(&KeyEvent::from_str("y").unwrap(), &get_unprocessed_state(vec!['n']), false);
        assert_eq!(Instruction::InsertInput('y'), result[0]);

        let result = handler.get_instruction(&KeyEvent::from_str("a").unwrap(), &get_unprocessed_state(vec!['b', 'y']), false);
        assert_eq!(Instruction::InputKanaPrecomposition { converted: "びゃ", carry_over: &vec![] }, result[0]);
    }
}