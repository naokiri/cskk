use std::fmt::Debug;

use xkbcommon::xkb;

use crate::dictionary::candidate::Candidate;
use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::{CskkDictionary, Dictionary};
use crate::keyevent::{KeyEvent, SkkKeyModifier};
use crate::skk_modes::CompositionMode;
use crate::Instruction::ChangeCompositionMode;
use crate::{CommandHandler, CskkState, Instruction};
use std::slice::IterMut;
use std::sync::Arc;

///
/// かな -> 漢字 ハンドラ。
///
#[derive(Debug)]
pub struct KanaCompositionHandler {
    dictionaries: Vec<CskkDictionary>,
}

impl KanaCompositionHandler {
    pub fn new(dictionaries: Vec<CskkDictionary>) -> Self {
        KanaCompositionHandler { dictionaries }
    }

    pub fn get_dictionaries(&mut self) -> IterMut<'_, CskkDictionary> {
        self.dictionaries.iter_mut()
    }

    // dictionary list order search, dedupe by kouho and add to list and return all candidates
    fn get_all_candidates(&self, a: &str) -> Option<&DictEntry> {
        for dictionary in self.dictionaries.iter() {
            if let Some(dict_entry) = match dictionary {
                CskkDictionary::StaticFile(dict) => dict.lookup(a, false),
                CskkDictionary::UserFile(dict) => dict.lookup(a, false),
            } {
                return Some(dict_entry);
            }
        }
        None
    }

    /// confirm the candidate.
    /// This updates writable dictionaries candidate order or add new entry which confirmed.
    /// Returns true if updated any dictionary.
    pub fn confirm_candidate(&mut self, midashi: &str, okuri: bool, kouho_text: &str) -> bool {
        let mut result = false;
        let candidate = Candidate::new(
            Arc::new(midashi.to_string()),
            okuri,
            Arc::new(kouho_text.to_string()),
            None,
            None,
        );
        for dictionary in self.dictionaries.iter_mut() {
            if let Ok(res) = match dictionary {
                CskkDictionary::StaticFile(ref mut dict) => dict.select_candidate(&candidate),
                CskkDictionary::UserFile(ref mut dict) => dict.select_candidate(&candidate),
            } {
                if res {
                    result = res;
                }
            }
        }
        result
    }

    /// purge the candidate.
    /// This updates writable dictionaries candidate order or add new entry which confirmed.
    /// Returns true if updated any dictionary.
    pub fn purge_candidate(&mut self, midashi: &str, okuri: bool, kouho_text: &str) -> bool {
        let mut result = false;
        let candidate = Candidate::new(
            Arc::new(midashi.to_string()),
            okuri,
            Arc::new(kouho_text.to_string()),
            None,
            None,
        );
        for dictionary in self.dictionaries.iter_mut() {
            if let Ok(res) = match dictionary {
                CskkDictionary::StaticFile(ref mut dict) => dict.purge_candidate(&candidate),
                CskkDictionary::UserFile(ref mut dict) => dict.purge_candidate(&candidate),
            } {
                if res {
                    result = res;
                }
            }
        }
        result
    }

    ///
    /// Returns the nth candidate.
    /// first selection_pointer == 0
    ///
    fn get_nth_candidate(
        &self,
        to_composite: &str,
        selection_pointer: usize,
    ) -> Option<&Candidate> {
        let dict_entry = self.get_all_candidates(to_composite);

        if let Some(entry) = dict_entry {
            let candidates = entry.get_candidates();
            candidates.get(selection_pointer)
        } else {
            None
        }
    }

    /// instruction to inidicate the candidate at the pointer.
    fn indicate_candidate(&self, to_composite: &str, selection_pointer: usize) -> Vec<Instruction> {
        let mut instructions = vec![];
        if let Some(candidate) = self.get_nth_candidate(to_composite, selection_pointer) {
            instructions.push(Instruction::SetComposition {
                kanji: candidate.kouho_text.to_string(),
            });
        } else {
            instructions.push(Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::Register,
                delegate: false,
            });
            instructions.push(Instruction::FinishConsumingKeyEvent);
        }
        instructions
    }
}

impl CommandHandler for KanaCompositionHandler {
    fn can_process(&self, key_event: &KeyEvent) -> bool {
        let symbol = key_event.get_symbol();
        (xkb::keysyms::KEY_space <= symbol && symbol <= xkb::keysyms::KEY_asciitilde)
            || symbol == xkb::keysyms::KEY_Return
    }

    fn get_instruction(
        &self,
        key_event: &KeyEvent,
        current_state: &CskkState,
        is_delegated: bool,
    ) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        let symbol = key_event.get_symbol();
        let modifier = key_event.get_modifier();
        // if symbol == xkb::keysyms::KEY_space {} else if xkb::keysyms::KEY_0 <= symbol && symbol <= xkb::keysyms::KEY_9 {
        //     // TODO: 選択肢から直接 キー0-9でとりあえずif文書いただけ
        // } else if !is_delegated && symbol == xkb::keysyms::KEY_greater {
        //     // TODO: 接尾辞変換 skk 16.2マニュアル 5.5.3
        // } else
        if !is_delegated
            && (xkb::keysyms::KEY_Return == symbol
                || (xkb::keysyms::KEY_j == symbol && modifier.contains(SkkKeyModifier::CONTROL)))
        {
            // 現在の変換で確定させ、Directに戻す
            instructions.push(Instruction::ConfirmComposition);
            instructions.push(Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::Direct,
                delegate: false,
            });
            instructions.push(Instruction::FinishConsumingKeyEvent);
            return instructions;
        } else if !is_delegated
            && (xkb::keysyms::KEY_g == symbol && modifier.contains(SkkKeyModifier::CONTROL))
        {
            // Abort
            instructions.push(Instruction::Abort);
            instructions.push(ChangeCompositionMode {
                composition_mode: CompositionMode::PreComposition,
                delegate: false,
            });
            instructions.push(Instruction::FinishConsumingKeyEvent);
        } else if !is_delegated && symbol == xkb::keysyms::KEY_X {
            instructions.push(Instruction::Purge);
            instructions.push(Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::Direct,
                delegate: false,
            });
            instructions.push(Instruction::FinishConsumingKeyEvent);
        } else if is_delegated {
            let raw_to_composite = &*current_state.raw_to_composite;
            let selection_pointer = current_state.selection_pointer;
            instructions.append(&mut self.indicate_candidate(raw_to_composite, selection_pointer));
        } else if symbol == xkb::keysyms::KEY_space {
            // 次の候補を返す
            let raw_to_composite = &*current_state.raw_to_composite;
            let selection_pointer = current_state.selection_pointer;
            instructions.push(Instruction::NextCandidatePointer);
            instructions
                .append(&mut self.indicate_candidate(raw_to_composite, selection_pointer + 1));
        } else if !is_delegated && (xkb::keysyms::KEY_a <= symbol && symbol <= xkb::keysyms::KEY_z)
            || (xkb::keysyms::KEY_A <= symbol && symbol <= xkb::keysyms::KEY_Z)
            || symbol == xkb::keysyms::KEY_BackSpace
        {
            // 現在の変換で確定させ、次のモードでキー入力を処理させる。 "I s i space k" の kのような時。
            // 後続で入力として処理させるので、Finishしない。
            instructions.push(Instruction::ConfirmComposition);
            instructions.push(ChangeCompositionMode {
                composition_mode: CompositionMode::Direct,
                delegate: true,
            });
        }

        instructions
    }
}

#[cfg(test)]
impl KanaCompositionHandler {
    fn test_handler(dictionaries: Vec<CskkDictionary>) -> Self {
        KanaCompositionHandler { dictionaries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::static_dict::StaticFileDict;

    #[test]
    fn can_process_single() {
        let dict =
            CskkDictionary::StaticFile(StaticFileDict::new("tests/data/SKK-JISYO.S", "euc-jp"));
        let handler = KanaCompositionHandler::test_handler(vec![dict]);
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap());
        assert!(result);
    }
}
