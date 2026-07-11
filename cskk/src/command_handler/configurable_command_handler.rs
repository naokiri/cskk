use crate::rule::{CskkCommandRule, CskkRule};
use crate::{CompositionMode, CskkKeyEvent, InputMode, Instruction, SkkKeyModifier};

pub(crate) struct ConfigurableCommandHandler {
    command_rule: CskkCommandRule,
}

impl ConfigurableCommandHandler {
    pub(crate) fn new(rule: &CskkRule) -> Self {
        Self {
            command_rule: rule.get_command_rule().clone(),
        }
    }

    ///
    /// Returns command handler of empty command.
    ///
    pub(crate) fn new_empty() -> Self {
        Self {
            command_rule: CskkCommandRule::new_empty(),
        }
    }

    /// コマンドとして処理し、文字入力として処理しない場合にinstructionを返す。
    pub(crate) fn get_instruction(
        &self,
        key_event: &CskkKeyEvent,
        current_input_mode: &InputMode,
        current_composition_mode: &CompositionMode,
    ) -> Option<Vec<Instruction>> {
        let mut matching_key = key_event.clone();
        if matching_key.is_upper() {
            matching_key
                .get_modifier_mut()
                .remove(SkkKeyModifier::SHIFT);
        }

        if let Some(inner_rules) = self
            .command_rule
            .get_inner_ruleset(current_composition_mode)
        {
            if let Some(command_map) = inner_rules.get_command_map(current_input_mode) {
                return command_map.get(&matching_key).map(|x| x.to_owned());
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::CskkRuleMetadata;
    use std::str::FromStr;

    fn test_preset_handler() -> ConfigurableCommandHandler {
        let rule_metadata =
            CskkRuleMetadata::load_metadata_from_directory("../assets/rules").unwrap();
        let default_rule = rule_metadata.load_default_rule().unwrap();
        ConfigurableCommandHandler::new(&default_rule)
    }

    #[test]
    fn get_instruction() {
        let handler = test_preset_handler();
        let result = handler
            .get_instruction(
                &CskkKeyEvent::from_str("BackSpace").unwrap(),
                &InputMode::Hiragana,
                &CompositionMode::Direct,
            )
            .unwrap();
        assert_eq!(Instruction::Delete, result[0]);
    }
}
