use crate::input_handler::InputHandler;
use crate::input_handler::kana_handler::KanaHandler;
use crate::Instruction;
use crate::keyevent::KeyEvent;

pub(crate) struct KanaPrecompositionHandler {
    kana_delegate_handler: Box<KanaHandler>,
}

impl KanaPrecompositionHandler{
    pub fn new(kana_delegate_handler: Box<KanaHandler>) -> Self {
        KanaPrecompositionHandler {
            kana_delegate_handler
        }
    }
}

impl InputHandler for KanaPrecompositionHandler {
    fn can_process(&self, key_event: &KeyEvent, unprocessed: &[char]) -> bool {
        unimplemented!()
    }

    fn get_instruction(&self, key_event: &KeyEvent, unprocessed: &[char]) -> Option<Instruction> {
        unimplemented!()
    }
}