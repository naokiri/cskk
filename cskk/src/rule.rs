use crate::{CompositionMode, CskkError, CskkKeyEvent, InputMode, Instruction};
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Deserialize, Debug)]
pub(crate) struct CskkRule {
    conversion: HashMap<String, (String, String)>,
    #[serde(flatten)]
    command: CskkCommandRule,
}

// Metadata.toml file
pub(crate) struct CskkRuleMetadata {
    base_dir: PathBuf,
    rules: BTreeMap<String, CskkRuleMetadataEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CskkRuleMetadataEntry {
    pub name: String,
    pub description: String,
    // path directory of actual rule.toml
    path: String,
}

#[derive(Deserialize, Default, Clone, Debug)]
pub(crate) struct CskkCommandRule {
    #[serde(default)]
    direct: CskkCommandRuleInner,
    #[serde(default)]
    pre_composition: CskkCommandRuleInner,
    #[serde(default)]
    pre_compisition_okurigana: CskkCommandRuleInner,
    #[serde(default)]
    composition_selection: CskkCommandRuleInner,
    #[serde(default)]
    abbreviation: CskkCommandRuleInner,
    #[serde(default)]
    completion: CskkCommandRuleInner,
}

impl CskkCommandRule {
    pub(crate) fn new_empty() -> Self {
        Self {
            direct: CskkCommandRuleInner::new_empty(),
            pre_composition: CskkCommandRuleInner::new_empty(),
            pre_compisition_okurigana: CskkCommandRuleInner::new_empty(),
            composition_selection: CskkCommandRuleInner::new_empty(),
            abbreviation: CskkCommandRuleInner::new_empty(),
            completion: CskkCommandRuleInner::new_empty(),
        }
    }

    pub(crate) fn get_inner_ruleset(
        &self,
        composition_mode: &CompositionMode,
    ) -> Option<&CskkCommandRuleInner> {
        match composition_mode {
            CompositionMode::Direct => Some(&self.direct),
            CompositionMode::PreComposition => Some(&self.pre_composition),
            CompositionMode::PreCompositionOkurigana => Some(&self.pre_compisition_okurigana),
            CompositionMode::CompositionSelection => Some(&self.composition_selection),
            CompositionMode::Abbreviation => Some(&self.abbreviation),
            CompositionMode::Completion => Some(&self.completion),
            _ => {
                // Shouldn't reach here but safe to not to stop on release.
                debug_assert!(false);
                None
            }
        }
    }
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub(crate) struct CskkCommandRuleInner {
    #[serde(default)]
    hiragana: HashMap<CskkKeyEvent, Vec<Instruction>>,
    #[serde(default)]
    katakana: HashMap<CskkKeyEvent, Vec<Instruction>>,
    #[serde(default)]
    hankakukatakana: HashMap<CskkKeyEvent, Vec<Instruction>>,
    #[serde(default)]
    zenkaku: HashMap<CskkKeyEvent, Vec<Instruction>>,
    #[serde(default)]
    ascii: HashMap<CskkKeyEvent, Vec<Instruction>>,
}

impl CskkCommandRuleInner {
    pub(crate) fn new_empty() -> Self {
        Self {
            hiragana: HashMap::new(),
            katakana: HashMap::new(),
            hankakukatakana: HashMap::new(),
            zenkaku: HashMap::new(),
            ascii: HashMap::new(),
        }
    }

    pub(crate) fn get_command_map(
        &self,
        input_mode: &InputMode,
    ) -> Option<&HashMap<CskkKeyEvent, Vec<Instruction>>> {
        match input_mode {
            InputMode::Hiragana => Some(&self.hiragana),
            InputMode::Katakana => Some(&self.katakana),
            InputMode::HankakuKatakana => Some(&self.hankakukatakana),
            InputMode::Zenkaku => Some(&self.zenkaku),
            InputMode::Ascii => Some(&self.ascii),
        }
    }
}

impl CskkRule {
    #[allow(dead_code)]
    pub(crate) fn load_default_rule_file() -> Result<Self, CskkError> {
        let base_dirs = xdg::BaseDirectories::new();
        let filepath = base_dirs
            .find_data_file("libcskk/rules/default/rule.toml")
            .ok_or_else(|| CskkError::RuleError("Default rule file not found".to_string()))?;
        Self::load_rule_file(&filepath)
    }

    pub(crate) fn load_rule_file(filepath: &Path) -> Result<Self, CskkError> {
        let mut file = File::open(filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let result = toml::from_str::<CskkRule>(&contents)?;
        Ok(result)
    }

    pub(crate) fn get_conversion_rule(&self) -> &HashMap<String, (String, String)> {
        &self.conversion
    }

    pub(crate) fn get_command_rule(&self) -> &CskkCommandRule {
        &self.command
    }
}

impl CskkRuleMetadata {
    /// Find which rules directory to use and load the metadata only.
    pub(crate) fn load_metadata() -> Result<Self, CskkError> {
        let base_directory = xdg::BaseDirectories::new();
        if let Some(rule_directory) = base_directory.find_data_file("libcskk/rules") {
            Self::load_metadata_from_path(&rule_directory)
        } else {
            Err(CskkError::RuleError("No rule metadata file".to_string()))
        }
    }

    /// ignore xdg directory spec and load metadata from specified directory
    /// This method is exposed for integration test purpose.
    /// Use [load_metadata] for your usecase.
    pub(crate) fn load_metadata_from_directory(file: &str) -> Result<Self, CskkError> {
        let rule_directory = PathBuf::from_str(file)?;
        Self::load_metadata_from_path(&rule_directory)
    }

    fn load_metadata_from_path(rule_directory: &PathBuf) -> Result<Self, CskkError> {
        let rule_directory = rule_directory.to_owned();
        let mut metadata_file = rule_directory.clone();
        metadata_file.push("metadata.toml");
        let mut file = File::open(metadata_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        //let result = toml::from_str::<CskkRuleDirectoryMetadata>(&contents)?;
        let result = toml::from_str::<BTreeMap<String, CskkRuleMetadataEntry>>(&contents)?;
        Ok(CskkRuleMetadata {
            base_dir: rule_directory,
            rules: result,
        })
    }

    /// Load the rule named "default"
    pub(crate) fn load_default_rule(&self) -> Result<CskkRule, CskkError> {
        self.load_rule("default")
    }

    ///
    /// 引数ruleのidのrule.tomlファイルを読み出す。
    ///
    pub(crate) fn load_rule(&self, rule: &str) -> Result<CskkRule, CskkError> {
        let base_direcotry = &self.base_dir;
        if let Some(rule) = &self.rules.get(rule) {
            let mut file_path = base_direcotry.clone();
            file_path.push(&rule.path);
            file_path.push("rule.toml");
            let result = CskkRule::load_rule_file(file_path.as_path())?;
            Ok(result)
        } else {
            Err(CskkError::RuleError("Unknown rule specified.".to_string()))
        }
    }

    /// 使えるルールの(キー、名称、説明)を返す
    pub(crate) fn get_rule_list(self) -> BTreeMap<String, CskkRuleMetadataEntry> {
        self.rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize, Debug)]
    struct CommandOnly {
        #[serde(flatten)]
        command: CskkCommandRule,
    }

    #[test]
    fn load_preset_file() {
        let filepath = "../assets/rules/default/rule.toml";
        let result = CskkRule::load_rule_file(Path::new(&filepath)).unwrap();
        println!("{:?}", result.command);
    }

    #[test]
    fn load_preset_metadata() {
        let filepath = "../assets/rules";
        let result = CskkRuleMetadata::load_metadata_from_directory(filepath).unwrap();
        println!("{:?}", result.rules);
        let rule_load = result.load_default_rule();
        assert!(rule_load.is_ok())
    }

    #[test]
    fn read_command_rule() {
        let str = r#"
        [direct.hiragana]
        "C-g" = ["Abort"]
        "q" = ["ChangeInputMode(Katakana)"]
        # Comment example, and empty set example
        [direct.katakana]
        "#;
        let result = toml::from_str::<CommandOnly>(str).unwrap();
        println!("{result:?}");
        println!("{:?}", result.command.direct);
    }
}
