use xkbcommon::xkb;

use crate::input_handler::InputHandler;
use crate::Instruction;
use crate::kana_converter::KanaConverter;
use crate::keyevent::KeyEvent;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct KanaCompositionHandler {}

impl KanaCompositionHandler {
    pub fn new() -> Self {
        KanaCompositionHandler {}
    }

    // dictionary list order search, dedupe by kouho and add to list and return all candidates
    fn get_all_candidates() {}
}

impl InputHandler for KanaCompositionHandler {
    fn can_process(&self, key_event: &KeyEvent, unprocessed: &[char]) -> bool {
        key_event.get_symbol() == xkb::keysyms::KEY_space
    }

    fn get_instruction(&self, key_event: &KeyEvent, unprocessed: &[char]) -> Vec<Instruction> {
        unimplemented!()
    }
}

#[cfg(test)]
impl KanaCompositionHandler {
    fn test_handler() -> Self {
        KanaCompositionHandler {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_process_single() {
        let handler = KanaCompositionHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec![]);
        assert!(!result);
    }
}
