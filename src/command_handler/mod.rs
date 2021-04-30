use crate::keyevent::CskkKeyEvent;
use crate::{CskkState, Instruction};

pub mod direct_mode_command_handler;
pub mod kana_composition_handler;
pub mod kana_precomposition_handler;

/// 文字入力以外の命令としてキー入力を解釈するもののハンドラ
///
pub(crate) trait CommandHandler {
    ///
    /// True if key_event should be consumed by current handler.
    /// TODO: input_mode関連でcurrent_stateなしでは正しく返せない考慮漏れがあった。command_handler周辺は実際にlibで必要になったら書き直す必要があるかも。実際にlibで必要になるまで後回し。
    /// FIXME: Should this be in this trait?
    /// FIXME: get_instructionの配列長さで呼び出し側で置きかえていってしまっているので廃止するかも
    fn can_process(&self, key_event: &CskkKeyEvent) -> bool;
    fn get_instruction(
        &self,
        key_event: &CskkKeyEvent,
        current_state: &CskkState,
        is_delegated: bool,
    ) -> Vec<Instruction>;
}

impl<T> CommandHandler for &T
where
    T: CommandHandler,
{
    fn can_process(&self, key_event: &CskkKeyEvent) -> bool {
        (*self).can_process(key_event)
    }

    /// Only return instruction, never change state.
    fn get_instruction(
        &self,
        key_event: &CskkKeyEvent,
        current_state: &CskkState,
        is_delegated: bool,
    ) -> Vec<Instruction> {
        (*self).get_instruction(key_event, current_state, is_delegated)
    }
}
