use xkbcommon::xkb;

use crate::{CskkState, Instruction};
use crate::command_handler::CommandHandler;
use crate::keyevent::KeyEvent;
use crate::keyevent::SkkKeyModifier;
use crate::skk_modes::{CompositionMode, InputMode, has_rom2kana_conversion};

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectModeCommandHandler {}

impl DirectModeCommandHandler {
    pub fn new() -> Self {
        DirectModeCommandHandler {}
    }
}

impl CommandHandler for DirectModeCommandHandler {
    fn can_process(&self, key_event: &KeyEvent) -> bool {
        let modifier = key_event.get_modifier();
        let symbol = key_event.get_symbol();
        if modifier.contains(SkkKeyModifier::CONTROL) {
            match symbol {
                xkb::keysyms::KEY_l |
                xkb::keysyms::KEY_L |
                xkb::keysyms::KEY_q |
                xkb::keysyms::KEY_Q |
                xkb::keysyms::KEY_j |
                xkb::keysyms::KEY_J |
                xkb::keysyms::KEY_g |
                xkb::keysyms::KEY_G => {
                    return true;
                }
                _ => {
                    return false;
                }
            }
        }


        xkb::keysyms::KEY_space <= symbol && symbol <= xkb::keysyms::KEY_asciitilde
    }

    fn get_instruction(&self, key_event: &KeyEvent, current_state: &CskkState, _is_delegated: bool) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let symbol = key_event.get_symbol();
        let modifier = key_event.get_modifier();
        match symbol {
            // SKK 16.2 manual 4.2.2 input mode changing
            xkb::keysyms::KEY_l => {
                match current_state.input_mode {
                    InputMode::Hiragana | InputMode::Katakana | InputMode::HankakuKatakana => {
                        instructions.push(Instruction::OutputNNIfAny(current_state.input_mode.clone()));
                        instructions.push(Instruction::ChangeInputMode(InputMode::Ascii));
                        instructions.push(Instruction::FlushPreviousCarryOver);
                        instructions.push(Instruction::FinishConsumingKeyEvent);
                    }
                    _ => {}
                }
            }
            xkb::keysyms::KEY_L => {
                match current_state.input_mode {
                    InputMode::Hiragana | InputMode::Katakana | InputMode::HankakuKatakana => {
                        instructions.push(Instruction::OutputNNIfAny(current_state.input_mode.clone()));
                        instructions.push(Instruction::ChangeInputMode(InputMode::Zenkaku));
                        instructions.push(Instruction::FlushPreviousCarryOver);
                        instructions.push(Instruction::FinishConsumingKeyEvent);
                    }
                    _ => {}
                }
            }
            xkb::keysyms::KEY_q => {
                if modifier.contains(SkkKeyModifier::CONTROL) {
                    match current_state.input_mode {
                        InputMode::Hiragana | InputMode::Katakana => {
                            instructions.push(Instruction::OutputNNIfAny(current_state.input_mode.clone()));
                            instructions.push(Instruction::ChangeInputMode(InputMode::HankakuKatakana));
                            instructions.push(Instruction::FlushPreviousCarryOver);
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                        InputMode::HankakuKatakana => {
                            instructions.push(Instruction::OutputNNIfAny(current_state.input_mode.clone()));
                            instructions.push(Instruction::ChangeInputMode(InputMode::Hiragana));
                            instructions.push(Instruction::FlushPreviousCarryOver);
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                        _ => {}
                    }
                } else {
                    match current_state.input_mode {
                        InputMode::Hiragana => {
                            instructions.push(Instruction::OutputNNIfAny(current_state.input_mode.clone()));
                            instructions.push(Instruction::ChangeInputMode(InputMode::Katakana));
                            instructions.push(Instruction::FlushPreviousCarryOver);
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                        InputMode::Katakana | InputMode::HankakuKatakana => {
                            instructions.push(Instruction::OutputNNIfAny(current_state.input_mode.clone()));
                            instructions.push(Instruction::ChangeInputMode(InputMode::Hiragana));
                            instructions.push(Instruction::FlushPreviousCarryOver);
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                        _ => {}
                    }
                }
            }
            // xkb::keysyms::KEY_Q => {
            //     match current_state.input_mode {
            //         InputMode::Hiragana |
            //         InputMode::Katakana => {
            //             instructions.push(Instruction::ChangeCompositionMode {composition_mode: CompositionMode::PreComposition, delegate: false})；
            //             instructions.push(Instruction::FinishConsumingKeyEvent);
            //         }
            //         _ => {}
            //     }
            // }
            xkb::keysyms::KEY_j => {
                if modifier.contains(SkkKeyModifier::CONTROL) {
                    match current_state.input_mode {
                        InputMode::Ascii | InputMode::Zenkaku => {
                            instructions.push(Instruction::ChangeInputMode(InputMode::Hiragana));
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                        _ => {}
                    }
                }
            }
            xkb::keysyms::KEY_g |
            xkb::keysyms::KEY_G => {
                if has_rom2kana_conversion(&current_state.input_mode, &current_state.composition_mode) {
                    instructions.push(Instruction::FlushPreviousCarryOver);
                    instructions.push(Instruction::FinishConsumingKeyEvent);
                }
            }
            _ => {}
        }

        if instructions.is_empty() &&
            has_rom2kana_conversion(&current_state.input_mode, &current_state.composition_mode) &&
            xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z {
                instructions.push(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::PreComposition, delegate: false });
        }
        instructions
    }
}

#[cfg(test)]
impl DirectModeCommandHandler {
    fn test_handler() -> Self {
        DirectModeCommandHandler {}
    }
}

#[cfg(test)]
mod tests {
    use xkbcommon::xkb::keysyms;

    use crate::InputMode;
    use crate::keyevent::SkkKeyModifier;

    use super::*;

    fn init() {
        crate::unit_tests::INIT_SYNC.call_once(|| {
            let _ = env_logger::init();
        });
    }

    fn get_test_state() -> CskkState {
        CskkState::new_test_state(InputMode::Hiragana,
                                  CompositionMode::Direct,
                                  vec![])
    }

    #[test]
    fn can_process_single() {
        let handler = DirectModeCommandHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap());
        assert!(result);
    }

    #[test]
    fn can_process_intermediate() {
        let handler = DirectModeCommandHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("k").unwrap());
        assert!(result);
    }

    #[test]
    fn handler_works() {
        let handler = DirectModeCommandHandler::test_handler();

        let result = handler.can_process(&KeyEvent::from_keysym(keysyms::KEY_apostrophe, SkkKeyModifier::NONE));
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("b").unwrap());
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("Y").unwrap());
        assert!(result);
    }

    #[test]
    fn get_instruction() {
        let handler = DirectModeCommandHandler::test_handler();

        let result = handler.get_instruction(&KeyEvent::from_str("b").unwrap(), &get_test_state(), false);
        assert!(result.is_empty())
    }

    #[test]
    fn switch_mode() {
        init();
        let handler = DirectModeCommandHandler::test_handler();
        let key_event = KeyEvent::from_str("B").unwrap();
        assert!(key_event.get_symbol() <= xkb::keysyms::KEY_asciitilde);
        assert!(xkb::keysyms::KEY_A <= key_event.get_symbol());

        let result = handler.get_instruction(&key_event, &get_test_state(), false);
        assert_eq!(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::PreComposition, delegate: false }, result[0]);
    }
}