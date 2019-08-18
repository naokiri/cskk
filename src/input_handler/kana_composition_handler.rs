use std::borrow::Borrow;
use std::sync::Arc;

use xkbcommon::xkb;

use crate::{CskkState, Instruction};
use crate::dictionary::candidate::Candidate;
use crate::dictionary::on_memory_dict::OnMemoryDict;
use crate::input_handler::InputHandler;
use crate::keyevent::KeyEvent;
use crate::dictionary::{DictEntry, Dictionary};

///
/// かな -> 漢字 ハンドラ。とりあえず送りなしのみ。
///
#[derive(Debug)]
pub(crate) struct KanaCompositionHandler<Dict : Dictionary> {
    dictionary: Dict,
}

impl KanaCompositionHandler<OnMemoryDict> {
    pub fn new() -> Self {
        KanaCompositionHandler {
            dictionary: OnMemoryDict::new()
        }
    }

    // dictionary list order search, dedupe by kouho and add to list and return all candidates
    fn get_all_candidates(&self, a: &str) -> Option<&DictEntry> {
        self.dictionary.lookup(a, false)
    }
}

impl InputHandler for KanaCompositionHandler {
    fn can_process(&self, key_event: &KeyEvent, unprocessed: &[char]) -> bool {
        key_event.get_symbol() == xkb::keysyms::KEY_space
    }

    fn get_instruction(&self, key_event: &KeyEvent, current_state: &CskkState) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let ref to_composite = *current_state.converted_kana_to_composite;
        let dict_entry = self.get_all_candidates(to_composite);
        let selection_pointer = current_state.selection_pointer + 1;
        if let Some(entry) = dict_entry {
            let candidates = entry.get_candidates();
            let candidate = candidates.get(selection_pointer);
            match candidate {
                Some(candidate) => {
                    instructions.push(Instruction::SetComposition {
                        kanji: &candidate.clone().kouho_text,
                        okuri: None,
                    });
                }
                None => {
                    unimplemented!("no more entry. Delegate to registration mode.")
                }
            }
        } else {
            unimplemented!("no entry. Delegate to registration mode.")
        }

        instructions
    }
}

#[cfg(test)]
impl KanaCompositionHandler {
//    fn test_handler() -> Self {
//        KanaCompositionHandler {
//            dictionary:
//        }
//    }
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
