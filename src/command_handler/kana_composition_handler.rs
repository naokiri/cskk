use std::fmt::Debug;

use xkbcommon::xkb;

use crate::{CskkState, Instruction};
use crate::dictionary::{DictEntry, Dictionary};
#[cfg(test)]
use crate::dictionary::on_memory_dict::OnMemoryDict;
use crate::command_handler::CommandHandler;
use crate::keyevent::{KeyEvent, SkkKeyModifier};
use crate::skk_modes::CompositionMode;

///
/// かな -> 漢字 ハンドラ。
///
#[derive(Debug)]
pub(crate) struct KanaCompositionHandler<Dict: Dictionary> {
    dictionary: Dict,
}

impl<Dict: Dictionary + Debug> KanaCompositionHandler<Dict> {
    pub fn new() -> Self {
        KanaCompositionHandler {
            dictionary: Dict::new()
        }
    }

    // dictionary list order search, dedupe by kouho and add to list and return all candidates
    fn get_all_candidates(&self, a: &str) -> Option<&DictEntry> {
        self.dictionary.lookup(a, false)
    }
}

#[allow(clippy::if_same_then_else)]
impl<Dict: Dictionary + Debug> CommandHandler for KanaCompositionHandler<Dict> {
    fn can_process(&self, key_event: &KeyEvent) -> bool {
        let symbol = key_event.get_symbol();
        (xkb::keysyms::KEY_space <= symbol && symbol <= xkb::keysyms::KEY_asciitilde) || symbol == xkb::keysyms::KEY_Return
    }

    fn get_instruction(&self, key_event: &KeyEvent, current_state: &CskkState, is_delegated: bool) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let symbol = key_event.get_symbol();
        let modifier = key_event.get_modifier();
        if symbol == xkb::keysyms::KEY_space {} else if xkb::keysyms::KEY_0 <= symbol && symbol <= xkb::keysyms::KEY_9 {
            // TODO: 選択肢から直接 キー0-9でとりあえずif文書いただけ
        } else if !is_delegated && symbol == xkb::keysyms::KEY_greater {
            // TODO: 接尾辞変換 skk 16.2マニュアル 5.5.3
        } else if !is_delegated && xkb::keysyms::KEY_a <= symbol && symbol <= xkb::keysyms::KEY_z {
            // TODO: 現在の変換で確定させ、次のモードでキー入力を処理させる。 "I s i space k" の kのような時。
        } else if !is_delegated && (xkb::keysyms::KEY_Return == symbol || (xkb::keysyms::KEY_j == symbol && SkkKeyModifier::CONTROL == modifier)) {
            // 現在の変換で確定させ、Directに戻す
            instructions.push(Instruction::ConfirmComposition);
            instructions.push(Instruction::ChangeCompositionMode { composition_mode: CompositionMode::Direct, delegate: false });
            instructions.push(Instruction::FinishConsumingKeyEvent);
        }

        let raw_to_composite = &*current_state.raw_to_composite;
        dbg!(raw_to_composite);
        let dict_entry = self.get_all_candidates(raw_to_composite);
        let mut selection_pointer = current_state.selection_pointer;
        if !is_delegated {
            selection_pointer += 1;
        }

        if let Some(entry) = dict_entry {
            let candidates = entry.get_candidates();
            let candidate = candidates.get(selection_pointer);
            match candidate {
                Some(candidate) => {
                    instructions.push(Instruction::SetComposition {
                        kanji: &candidate.kouho_text
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
impl KanaCompositionHandler<OnMemoryDict> {
    fn test_handler() -> Self {
        KanaCompositionHandler {
            dictionary: OnMemoryDict::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_process_single() {
        let handler = KanaCompositionHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap());
        assert!(result);
    }
}
