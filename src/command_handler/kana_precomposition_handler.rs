use xkbcommon::xkb;

use crate::command_handler::CommandHandler;
use crate::keyevent::{KeyEvent, SkkKeyModifier};
use crate::skk_modes::{CompositionMode, InputMode};
use crate::{CskkState, Instruction};

// PreComposition とそのサブモード
#[derive(Debug)]
pub struct KanaPrecompositionHandler {}

impl KanaPrecompositionHandler {
    pub fn new() -> Self {
        KanaPrecompositionHandler {}
    }
}

impl CommandHandler for KanaPrecompositionHandler {
    fn can_process(&self, key_event: &KeyEvent) -> bool {
        let symbol = key_event.get_symbol();
        xkb::keysyms::KEY_space <= symbol && symbol <= xkb::keysyms::KEY_asciitilde
    }

    fn get_instruction(
        &self,
        key_event: &KeyEvent,
        current_state: &CskkState,
        _is_delegated: bool,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let current_composition_mode = &current_state.composition_mode;
        let current_input_mode = current_state.input_mode;
        debug_assert!(
            current_composition_mode == &CompositionMode::PreComposition
                || current_composition_mode == &CompositionMode::PreCompositionOkurigana
        );
        // TODO: ▽ひらがな + Ctrl-G => FlushAbort

        let symbol = key_event.get_symbol();
        let modifier = key_event.get_modifier();
        // Does not check if key_event's modifier contains SHIFT because keysym is different for 'a' and 'A'.
        let is_capital = xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z;
        if symbol == xkb::keysyms::KEY_space {
            // space
            instructions.push(Instruction::OutputNNIfAny(current_input_mode));
            instructions.push(Instruction::FlushPreviousCarryOver);
            instructions.push(Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::CompositionSelection,
                delegate: true,
            });
            return instructions;
        } else if is_capital && current_state.composition_mode == CompositionMode::PreComposition {
            // 大文字
            if !current_state.raw_to_composite.is_empty() {
                instructions.push(Instruction::ChangeCompositionMode {
                    composition_mode: CompositionMode::PreCompositionOkurigana,
                    delegate: false,
                });
            }
        } else if symbol == xkb::keysyms::KEY_greater {
            // TODO: SKK16.2 マニュアル 5.5.3 接頭辞変換
        } else if symbol == xkb::keysyms::KEY_q && !modifier.contains(SkkKeyModifier::CONTROL) {
            // q
            if current_input_mode == InputMode::Katakana
                || current_input_mode == InputMode::HankakuKatakana
            {
                instructions.push(Instruction::OutputNNIfAny(InputMode::Hiragana));
                instructions.push(Instruction::FlushPreviousCarryOver);
                instructions.push(Instruction::ConfirmAsHiragana);
            } else {
                instructions.push(Instruction::OutputNNIfAny(InputMode::Katakana));
                instructions.push(Instruction::FlushPreviousCarryOver);
                instructions.push(Instruction::ConfirmAsKatakana);
            }
            instructions.push(Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::Direct,
                delegate: false,
            });
            instructions.push(Instruction::FinishConsumingKeyEvent);
        } else if symbol == xkb::keysyms::KEY_q && modifier.contains(SkkKeyModifier::CONTROL) {
            // C-q
            instructions.push(Instruction::FlushPreviousCarryOver);
            instructions.push(Instruction::ConfirmAsJISX0201);
            instructions.push(Instruction::FinishConsumingKeyEvent);
        } else if symbol == xkb::keysyms::KEY_g && modifier.contains(SkkKeyModifier::CONTROL) {
            // C-g
            instructions.push(Instruction::FlushPreviousCarryOver);
            instructions.push(Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::Direct,
                delegate: false,
            });
            instructions.push(Instruction::FinishConsumingKeyEvent);
        }

        instructions
    }
}

#[cfg(test)]
impl KanaPrecompositionHandler {
    fn test_handler() -> Self {
        KanaPrecompositionHandler {}
    }
}

#[cfg(test)]
mod tests {
    use crate::InputMode;

    use super::*;

    fn get_test_state() -> CskkState {
        CskkState::new_test_state(InputMode::Hiragana, CompositionMode::PreComposition, vec![])
    }

    #[test]
    fn can_process_single() {
        let handler = KanaPrecompositionHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap());
        assert!(result);
    }

    #[test]
    fn can_process_intermediate() {
        let handler = KanaPrecompositionHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("k").unwrap());
        assert!(result);
    }

    #[test]
    fn delegate_to_compositionselection_on_space() {
        let handler = KanaPrecompositionHandler::test_handler();

        let result = handler.get_instruction(
            &KeyEvent::from_str("space").unwrap(),
            &get_test_state(),
            false,
        );
        assert_eq!(Instruction::OutputNNIfAny(InputMode::Hiragana), result[0]);
        assert_eq!(Instruction::FlushPreviousCarryOver, result[1]);
        assert_eq!(
            Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::CompositionSelection,
                delegate: true
            },
            result[2]
        );
    }

    #[test]
    fn go_to_okuri_submode() {
        let mut test_state = get_test_state();
        test_state.raw_to_composite = "あ".to_string();
        let handler = KanaPrecompositionHandler::test_handler();
        let result = handler.get_instruction(&KeyEvent::from_str("K").unwrap(), &test_state, false);
        assert_eq!(
            Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::PreCompositionOkurigana,
                delegate: false
            },
            result[0]
        );
    }
}
