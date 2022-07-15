use xkbcommon::xkb;

use crate::command_handler::CommandHandler;
use crate::keyevent::CskkKeyEvent;
use crate::keyevent::SkkKeyModifier;
use crate::skk_modes::{has_rom2kana_conversion, CompositionMode, InputMode};
use crate::{CskkState, Instruction};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DirectModeCommandHandler {}

impl DirectModeCommandHandler {
    pub(crate) fn new() -> Self {
        DirectModeCommandHandler {}
    }
}

impl CommandHandler for DirectModeCommandHandler {
    fn can_process(&self, key_event: &CskkKeyEvent) -> bool {
        let modifier = key_event.get_modifier();
        let symbol = key_event.get_symbol();
        if modifier.contains(SkkKeyModifier::CONTROL) {
            return matches!(
                symbol,
                xkb::keysyms::KEY_l
                    | xkb::keysyms::KEY_L
                    | xkb::keysyms::KEY_q
                    | xkb::keysyms::KEY_Q
                    | xkb::keysyms::KEY_j
                    | xkb::keysyms::KEY_J
                    | xkb::keysyms::KEY_g
                    | xkb::keysyms::KEY_G
                    | xkb::keysyms::KEY_h
                    | xkb::keysyms::KEY_H
            );
        }

        (xkb::keysyms::KEY_space..=xkb::keysyms::KEY_asciitilde).contains(&symbol)
    }

    fn get_instruction(
        &self,
        key_event: &CskkKeyEvent,
        current_state: &CskkState,
        _is_delegated: bool,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let symbol = key_event.get_symbol();
        let modifier = key_event.get_modifier();
        match symbol {
            // SKK 16.2 manual 4.2.2 input mode changing
            xkb::keysyms::KEY_l => match current_state.input_mode {
                InputMode::Hiragana | InputMode::Katakana | InputMode::HankakuKatakana => {
                    instructions.push(Instruction::OutputNNIfAny(current_state.input_mode));
                    instructions.push(Instruction::ChangeInputMode(InputMode::Ascii));
                    instructions.push(Instruction::FlushPreviousCarryOver);
                    instructions.push(Instruction::FinishConsumingKeyEvent);
                }
                _ => {}
            },
            xkb::keysyms::KEY_L => match current_state.input_mode {
                InputMode::Hiragana | InputMode::Katakana | InputMode::HankakuKatakana => {
                    instructions.push(Instruction::OutputNNIfAny(current_state.input_mode));
                    instructions.push(Instruction::ChangeInputMode(InputMode::Zenkaku));
                    instructions.push(Instruction::FlushPreviousCarryOver);
                    instructions.push(Instruction::FinishConsumingKeyEvent);
                }
                _ => {}
            },
            xkb::keysyms::KEY_q => {
                if modifier.contains(SkkKeyModifier::CONTROL) {
                    match current_state.input_mode {
                        InputMode::Hiragana | InputMode::Katakana => {
                            instructions.push(Instruction::OutputNNIfAny(current_state.input_mode));
                            instructions
                                .push(Instruction::ChangeInputMode(InputMode::HankakuKatakana));
                            instructions.push(Instruction::FlushPreviousCarryOver);
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                        InputMode::HankakuKatakana => {
                            instructions.push(Instruction::OutputNNIfAny(current_state.input_mode));
                            instructions.push(Instruction::ChangeInputMode(InputMode::Hiragana));
                            instructions.push(Instruction::FlushPreviousCarryOver);
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                        _ => {}
                    }
                } else {
                    match current_state.input_mode {
                        InputMode::Hiragana => {
                            instructions.push(Instruction::OutputNNIfAny(current_state.input_mode));
                            instructions.push(Instruction::ChangeInputMode(InputMode::Katakana));
                            instructions.push(Instruction::FlushPreviousCarryOver);
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                        InputMode::Katakana | InputMode::HankakuKatakana => {
                            instructions.push(Instruction::OutputNNIfAny(current_state.input_mode));
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
                        _ => {
                            instructions.push(Instruction::FinishConsumingKeyEvent);
                        }
                    }
                }
            }
            xkb::keysyms::KEY_g | xkb::keysyms::KEY_G => {
                if has_rom2kana_conversion(
                    &current_state.input_mode,
                    &current_state.composition_mode,
                ) && modifier.contains(SkkKeyModifier::CONTROL)
                {
                    instructions.push(Instruction::FlushPreviousCarryOver);
                    // To abort from registration mode
                    instructions.push(Instruction::Abort);
                    instructions.push(Instruction::FinishConsumingKeyEvent);
                }
            }
            xkb::keysyms::KEY_Return => {
                instructions.push(Instruction::ConfirmDirect);
            }
            xkb::keysyms::KEY_m => {
                if modifier.contains(SkkKeyModifier::CONTROL) {
                    instructions.push(Instruction::ConfirmDirect);
                }
            }
            xkb::keysyms::KEY_BackSpace => {
                instructions.push(Instruction::DeleteDirect);
            }
            xkb::keysyms::KEY_h | xkb::keysyms::KEY_H => {
                if modifier.contains(SkkKeyModifier::CONTROL) {
                    instructions.push(Instruction::DeleteDirect);
                }
            }
            _ => {}
        }

        if instructions.is_empty()
            && has_rom2kana_conversion(&current_state.input_mode, &current_state.composition_mode)
            && (xkb::keysyms::KEY_A..=xkb::keysyms::KEY_Z).contains(&symbol)
        {
            instructions.push(Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::PreComposition,
                delegate: false,
            });
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

    use crate::keyevent::SkkKeyModifier;
    use crate::InputMode;

    use super::*;
    use crate::testhelper::init_test_logger;

    fn get_test_state() -> CskkState {
        CskkState::new_test_state(InputMode::Hiragana, CompositionMode::Direct, vec![])
    }

    #[test]
    fn can_process_single() {
        let handler = DirectModeCommandHandler::test_handler();
        let result = handler.can_process(&CskkKeyEvent::from_string_representation("a").unwrap());
        assert!(result);
    }

    #[test]
    fn can_process_intermediate() {
        let handler = DirectModeCommandHandler::test_handler();
        let result = handler.can_process(&CskkKeyEvent::from_string_representation("k").unwrap());
        assert!(result);
    }

    #[test]
    fn handler_works() {
        let handler = DirectModeCommandHandler::test_handler();

        let result = handler.can_process(&CskkKeyEvent::from_keysym_strict(
            keysyms::KEY_apostrophe,
            SkkKeyModifier::NONE,
        ));
        assert!(result);

        let result = handler.can_process(&CskkKeyEvent::from_string_representation("b").unwrap());
        assert!(result);

        let result = handler.can_process(&CskkKeyEvent::from_string_representation("Y").unwrap());
        assert!(result);
    }

    #[test]
    fn get_instruction() {
        let handler = DirectModeCommandHandler::test_handler();

        let result = handler.get_instruction(
            &CskkKeyEvent::from_string_representation("b").unwrap(),
            &get_test_state(),
            false,
        );
        assert!(result.is_empty())
    }

    #[test]
    fn switch_mode() {
        init_test_logger();
        let handler = DirectModeCommandHandler::test_handler();
        let key_event = CskkKeyEvent::from_string_representation("B").unwrap();
        assert!(key_event.get_symbol() <= xkb::keysyms::KEY_asciitilde);
        assert!(xkb::keysyms::KEY_A <= key_event.get_symbol());

        let result = handler.get_instruction(&key_event, &get_test_state(), false);
        assert_eq!(
            Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::PreComposition,
                delegate: false
            },
            result[0]
        );
    }

    #[test]
    fn consume_ctrl_j_on_non_modechange() {
        init_test_logger();
        let handler = DirectModeCommandHandler::test_handler();
        let key_event = CskkKeyEvent::from_string_representation("C-j").unwrap();
        let result = handler.get_instruction(&key_event, &get_test_state(), false);
        assert_eq!(Instruction::FinishConsumingKeyEvent, result[0]);
    }
}
