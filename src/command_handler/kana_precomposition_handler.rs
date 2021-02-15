use xkbcommon::xkb;

use crate::{CskkState, Instruction};
use crate::command_handler::CommandHandler;
use crate::keyevent::KeyEvent;
use crate::skk_modes::CompositionMode;

// PreComposition とそのサブモード
#[derive(Debug)]
pub(crate) struct KanaPrecompositionHandler {
}

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

    fn get_instruction(&self, key_event: &KeyEvent, current_state: &CskkState, _is_delegated: bool) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let current_composition_mode = &current_state.composition_mode;
        debug_assert!(current_composition_mode == &CompositionMode::PreComposition || current_composition_mode == &CompositionMode::PreCompositionOkurigana);
        // TODO: ▽ひらがな + 'q' => ヒラガナ
        // TODO: ▽ひらがな + Ctrl-G => FlushAbort

        let symbol = key_event.get_symbol();
        // Does not check if key_event's modifier contains SHIFT because keysym is different for 'a' and 'A'.
        let is_capital = xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z;
        if symbol == xkb::keysyms::KEY_space {
            instructions.push(Instruction::FlushPreviousCarryOver);
            // TODO: "K space" 等 flush後空の時にはcomposition selectionではなくFlushAbortしないといけない。CompositionSelection側で処理？
            instructions.push(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::CompositionSelection, delegate: true });
            return instructions;
        } else if is_capital && current_state.composition_mode == CompositionMode::PreComposition {
            instructions.push(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::PreCompositionOkurigana , delegate: false });
            // instructions.push(Instruction::SetCompositionOkuri);
        } else if symbol == xkb::keysyms::KEY_greater {
            // TODO: SKK16.2 マニュアル 5.5.3 接頭辞変換
        }

        instructions
    }
}


#[cfg(test)]
impl KanaPrecompositionHandler {
    fn test_handler() -> Self {
        let kana_converter = KanaConverter::default_converter();

        KanaPrecompositionHandler {
            // kana_converter: Box::new(kana_converter),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::InputMode;

    use super::*;

    fn get_test_state() -> CskkState {
        CskkState::new_test_state(InputMode::Hiragana,
                                  CompositionMode::PreComposition,
                                  vec![])
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

        let result = handler.get_instruction(&KeyEvent::from_str("space").unwrap(), &get_test_state(), false);
        assert_eq!(Instruction::FlushPreviousCarryOver, result[0]);
        assert_eq!(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::CompositionSelection, delegate: true }, result[1]);
    }

    #[test]
    fn go_to_okuri_submode() {
        let handler = KanaPrecompositionHandler::test_handler();
        let result = handler.get_instruction(&KeyEvent::from_str("K").unwrap(), &get_test_state(), false);
        assert_eq!(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::PreCompositionOkurigana , delegate: false }, result[0]);
    }
}