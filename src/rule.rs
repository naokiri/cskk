use crate::env::filepath_from_xdg_data_dir;
use crate::{CompositionMode, CskkError, CskkKeyEvent, InputMode, Instruction};
use std::collections::HashMap;
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
    rules: CskkRuleDirectoryMetadata,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CskkRuleDirectoryMetadata {
    entry: Vec<CskkRuleDirectoryMetadataEntry>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct CskkRuleDirectoryMetadataEntry {
    name: String,
    description: String,
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
}

impl CskkCommandRule {
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
            _ => None,
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
        let filepath = filepath_from_xdg_data_dir("libcskk/rules/default/rule.toml")?;
        Self::load_rule_file(Path::new(&filepath))
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
        let base_directory = xdg::BaseDirectories::new()?;
        if let Some(rule_directory) = base_directory.find_data_file("libcskk/rules") {
            let mut metadata_file = rule_directory.clone();
            metadata_file.push("metadata.toml");
            let mut file = File::open(metadata_file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let result = toml::from_str::<CskkRuleDirectoryMetadata>(&contents)?;

            Ok(CskkRuleMetadata {
                base_dir: rule_directory,
                rules: result,
            })
        } else {
            Err(CskkError::Error("No rule metadata file".to_string()))
        }
    }

    /// ignore xdg directory spec and load metadata from specified directory
    /// This method is for integration test purpose.
    /// Use [load_metadata] for your usecase.
    pub(crate) fn load_metadata_from_directory(file: &str) -> Result<Self, CskkError> {
        let rule_directory = PathBuf::from_str(file)?;
        let mut metadata_file = rule_directory.clone();
        metadata_file.push("metadata.toml");
        let mut file = File::open(metadata_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let result = toml::from_str::<CskkRuleDirectoryMetadata>(&contents)?;

        Ok(CskkRuleMetadata {
            base_dir: rule_directory,
            rules: result,
        })
    }

    /// Load the first rule
    pub(crate) fn load_default_rule(&self) -> Result<CskkRule, CskkError> {
        let base_direcotry = &self.base_dir;
        if let Some(rule) = &self.rules.entry.get(0) {
            let mut file_path = base_direcotry.clone();
            file_path.push(&rule.path);
            file_path.push("rule.toml");
            let result = CskkRule::load_rule_file(file_path.as_path())?;
            Ok(result)
        } else {
            Err(CskkError::Error(
                "No available rule in metadata".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompositionMode, CskkKeyEvent, InputMode, Instruction};
    #[derive(Deserialize, Debug)]
    struct CommandOnly {
        #[serde(flatten)]
        command: CskkCommandRule,
    }

    #[test]
    fn load_preset_file() {
        let filepath = "shared/rules/default/rule.toml";
        let result = CskkRule::load_rule_file(Path::new(&filepath)).unwrap();
        // println!("{:?}", result.conversion);
        println!("{:?}", result.command);
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
        let result = toml::from_str::<CommandOnly>(&str).unwrap();
        println!("{:?}", result);
        println!("{:?}", result.command.direct);
    }
}
