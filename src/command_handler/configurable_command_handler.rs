use crate::rule::{CskkCommandRule, CskkRule};
use crate::{CompositionMode, CskkKeyEvent, InputMode, Instruction};

pub(crate) struct ConfigurableCommandHandler {
    command_rule: CskkCommandRule,
}

impl ConfigurableCommandHandler {
    pub(crate) fn new(rule: &CskkRule) -> Self {
        Self {
            command_rule: rule.get_command_rule().clone(),
        }
    }

    pub(crate) fn get_instruction(
        &self,
        key_event: &CskkKeyEvent,
        current_input_mode: &InputMode,
        current_compositon_mode: &CompositionMode,
    ) -> Option<Vec<Instruction>> {
        if let Some(inner_rules) = self.command_rule.get_inner_ruleset(current_compositon_mode) {
            if let Some(command_map) = inner_rules.get_command_map(current_input_mode) {
                return command_map.get(key_event).map(|x| x.to_owned());
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use crate::CskkRuleMetadata;
    use super::*;

    fn test_preset_handler() -> ConfigurableCommandHandler {
        let rule_metadata = CskkRuleMetadata::load_metadata_from_directory("shared/rules").unwrap();
        let default_rule = rule_metadata.load_default_rule().unwrap();
        ConfigurableCommandHandler::new(&default_rule)
    }

    #[test]
    fn get_instruction() {
        let handler = test_preset_handler();
        let result = handler.get_instruction(&CskkKeyEvent::from_str(&"BackSpace").unwrap(), &InputMode::Hiragana, &CompositionMode::Direct).unwrap();
        assert_eq!(Instruction::DeleteDirect, result[0]);
    }


}
