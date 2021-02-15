use xkbcommon::xkb;

use crate::{CskkState, Instruction};
use crate::command_handler::CommandHandler;
use crate::keyevent::KeyEvent;
use crate::keyevent::SkkKeyModifier;
use crate::skk_modes::CompositionMode;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct KanaDirectHandler {
}

impl KanaDirectHandler {
    pub fn new() -> Self {
        KanaDirectHandler {
        }
    }
}

impl CommandHandler for KanaDirectHandler {
    fn can_process(&self, key_event: &KeyEvent) -> bool {
        let modifier = key_event.get_modifier();
        if modifier.contains(SkkKeyModifier::CONTROL) {
            return false;
        }

        let symbol = key_event.get_symbol();
        xkb::keysyms::KEY_space <= symbol && symbol <= xkb::keysyms::KEY_asciitilde
    }

    fn get_instruction(&self, key_event: &KeyEvent, _current_state: &CskkState, _is_delegated: bool) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        // TODO: ASCIIモード実装する時には、reset to ascii direct mode on l

        let symbol = key_event.get_symbol();
        if xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z {
            instructions.push(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::PreComposition, delegate: false });
        }
        instructions
    }
}

#[cfg(test)]
impl KanaDirectHandler {
    fn test_handler() -> Self {
        KanaDirectHandler {
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

    fn get_test_state() -> CskkState {
        CskkState::new_test_state(InputMode::Hiragana,
                                  CompositionMode::Direct,
                                  vec![])
    }

    #[test]
    fn can_process_single() {
        let handler = KanaDirectHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap());
        assert!(result);
    }

    #[test]
    fn can_process_intermediate() {
        let handler = KanaDirectHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("k").unwrap());
        assert!(result);
    }

    #[test]
    fn handler_works() {
        let handler = KanaDirectHandler::test_handler();

        let result = handler.can_process(&KeyEvent::from_keysym(keysyms::KEY_apostrophe, SkkKeyModifier::NONE));
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("b").unwrap());
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("Y").unwrap());
        assert!(result);
    }

    #[test]
    fn get_instruction() {
        let handler = KanaDirectHandler::test_handler();

        let result = handler.get_instruction(&KeyEvent::from_str("b").unwrap(), &get_test_state(), false);
        assert!(result.is_empty())
    }

    #[test]
    fn switch_mode() {
        init();
        let handler = KanaDirectHandler::test_handler();
        let key_event = KeyEvent::from_str("B").unwrap();
        assert!(key_event.get_symbol() <= xkb::keysyms::KEY_asciitilde);
        assert!(xkb::keysyms::KEY_A <= key_event.get_symbol());

        let result = handler.get_instruction(&key_event, &get_test_state(), false);
        assert_eq!(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::PreComposition, delegate: false }, result[0]);
    }
}