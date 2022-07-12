use std::fmt::Debug;

use xkbcommon::xkb;

use crate::dictionary::CskkDictionary;
use crate::keyevent::{CskkKeyEvent, SkkKeyModifier};
use crate::skk_modes::CompositionMode;
use crate::{CommandHandler, CskkState, Instruction};
use std::sync::Arc;

///
/// かな -> 漢字 ハンドラ。
///
#[derive(Debug)]
pub(crate) struct KanaCompositionHandler {
    dictionaries: Vec<Arc<CskkDictionary>>,
}

impl KanaCompositionHandler {
    pub(crate) fn new(dictionaries: Vec<Arc<CskkDictionary>>) -> Self {
        KanaCompositionHandler { dictionaries }
    }

    pub(crate) fn set_dictionaries(&mut self, dictionaries: Vec<Arc<CskkDictionary>>) {
        self.dictionaries = dictionaries;
    }
}

impl CommandHandler for KanaCompositionHandler {
    fn can_process(&self, key_event: &CskkKeyEvent) -> bool {
        let symbol = key_event.get_symbol();
        (xkb::keysyms::KEY_space..=xkb::keysyms::KEY_asciitilde).contains(&symbol)
            || symbol == xkb::keysyms::KEY_Return
    }

    fn get_instruction(
        &self,
        key_event: &CskkKeyEvent,
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
            && ((xkb::keysyms::KEY_g == symbol && modifier.contains(SkkKeyModifier::CONTROL))
                || xkb::keysyms::KEY_Escape == symbol)
        {
            // Abort
            instructions.push(Instruction::Abort);
            instructions.push(Instruction::ChangeCompositionMode {
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
            let candidate_list = &current_state.candidate_list;
            if candidate_list.is_empty() {
                instructions.push(Instruction::ChangeCompositionMode {
                    composition_mode: CompositionMode::Register,
                    delegate: false,
                })
            }
            instructions.push(Instruction::FinishConsumingKeyEvent);
        } else if symbol == xkb::keysyms::KEY_space {
            // 次の候補に進む
            let raw_to_composite = &current_state.raw_to_composite;
            let candidate_list = &current_state.candidate_list;
            let current_composition = current_state.candidate_list.get_current_to_composite();
            if !current_composition.eq(raw_to_composite) {
                instructions.push(Instruction::UpdateCandidateList);
            } else if candidate_list.has_next() {
                instructions.push(Instruction::NextCandidatePointer);
            } else {
                instructions.push(Instruction::ChangeCompositionMode {
                    composition_mode: CompositionMode::Register,
                    delegate: false,
                })
            }
            instructions.push(Instruction::FinishConsumingKeyEvent);
        } else if symbol == xkb::keysyms::KEY_x {
            let raw_to_composite = &current_state.raw_to_composite;
            let candidate_list = &current_state.candidate_list;
            let current_composition = current_state.candidate_list.get_current_to_composite();
            if !current_composition.eq(raw_to_composite) {
                instructions.push(Instruction::UpdateCandidateList);
            } else if candidate_list.has_previous() {
                instructions.push(Instruction::PreviousCandidatePointer);
            } else {
                instructions.push(Instruction::Abort);
                instructions.push(Instruction::ChangeCompositionMode {
                    composition_mode: CompositionMode::PreComposition,
                    delegate: false,
                });
            }
            instructions.push(Instruction::FinishConsumingKeyEvent);
        } else if !is_delegated
            && (key_event.is_ascii_inputtable() || symbol == xkb::keysyms::KEY_BackSpace)
        {
            // 現在の変換で確定させ、次のモードでキー入力を処理させる。 "I s i space k" の kのような時。
            // 後続で入力として処理させるので、Finishしない。
            instructions.push(Instruction::ConfirmComposition);
            instructions.push(Instruction::ChangeCompositionMode {
                composition_mode: CompositionMode::Direct,
                delegate: true,
            });
        }

        instructions
    }
}

#[cfg(test)]
impl KanaCompositionHandler {
    fn test_handler(dictionaries: Vec<Arc<CskkDictionary>>) -> Self {
        KanaCompositionHandler { dictionaries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::static_dict::StaticFileDict;
    use crate::dictionary::CskkDictionaryType;

    #[test]
    fn can_process_single() {
        let dict = CskkDictionary::new_static_dict(
            "tests/data/SKK-JISYO.S",
            "euc-jp",
        ).expect("New static dict");
        let handler = KanaCompositionHandler::test_handler(vec![Arc::new(dict)]);
        let result = handler.can_process(&CskkKeyEvent::from_str("a").unwrap());
        assert!(result);
    }
}
