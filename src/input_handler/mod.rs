use std::fmt::Debug;

use crate::Instruction;
use crate::keyevent::KeyEvent;

pub mod kana_direct_handler;
pub mod kana_precomposition_handler;

pub(crate) trait InputHandler: Debug {
    ///
    /// True if key_event should be consumed by current handler.
    /// FIXME: Should this be in this trait?
    ///
    fn can_process(&self, key_event: &KeyEvent, unprocessed: &[char]) -> bool;
    fn get_instruction<'a>(&'a self, key_event: &KeyEvent, unprocessed: &[char]) -> Vec<Instruction<'a>>;
}

//// Union to put handlers in same collection. Will not be required when Rust expands the usage of return impl Trait
//pub enum InputHandlerType<A> {
//    KanaHandler(A),
//}
//
//impl<T> InputHandler for InputHandlerType<T> where T: InputHandler {
//    fn can_process(&self, key_event: &KeyEvent, unprocessed: &[char]) -> bool {
//        match self {
//            InputHandlerType::KanaHandler(x) => x.can_process(key_event, unprocessed)
//        }
//    }
//
//    fn get_instruction(&self, key_event: &KeyEvent, unprocessed: &[char]) -> Option<Instruction> {
//        match self {
//            InputHandlerType::KanaHandler(x) => x.get_instruction(key_event, unprocessed)
//        }
//    }
//}
//
impl<T> InputHandler for &T where T: InputHandler {
    fn can_process(&self, key_event: &KeyEvent, unprocessed: &[char]) -> bool {
        (*self).can_process(key_event, unprocessed)
    }

    fn get_instruction(&self, key_event: &KeyEvent, unprocessed: &[char]) -> Vec<Instruction> {
        (*self).get_instruction(key_event, unprocessed)
    }
}