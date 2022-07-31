use crate::CskkError;
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

/// Rough design prototype yet
/// SKKの入力モード
/// DDSKK 16.2 マニュアル 4.2 に依る
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Deserialize, Serialize)]
#[repr(C)]
pub enum InputMode {
    // かなモード
    Hiragana,
    // カナモード
    Katakana,
    // JIS X 0201 カナ、 いわゆる半角カナ。 DDSKKでは独立したモード扱いではないので実装未定
    HankakuKatakana,
    // 全英モード
    Zenkaku,
    // アスキーモード
    Ascii,
}

impl FromStr for InputMode {
    type Err = CskkError;

    fn from_str(s: &str) -> Result<InputMode, CskkError> {
        match s {
            "Hiragana" => Ok(InputMode::Hiragana),
            "Katakana" => Ok(InputMode::Katakana),
            "HankakuKatakana" => Ok(InputMode::HankakuKatakana),
            "Zenkaku" => Ok(InputMode::Zenkaku),
            "Ascii" => Ok(InputMode::Ascii),

            s => Err(CskkError::ParseError(s.to_string())),
        }
    }
}

/// Rough design prototype yet
/// SKKの変換モード
/// DDSKK 16.2 マニュアル 4.3 に依る
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Deserialize, Serialize)]
#[repr(C)]
pub enum CompositionMode {
    // ■モード 確定入力モード
    Direct,
    // ▽モード 見出しモード
    PreComposition,
    // ▽モードの送り仮名時。Sub mode of PreComposition
    PreCompositionOkurigana,
    // ▼モード
    CompositionSelection,
    // SKK abbrev mode: Sub-mode of PreComposition
    Abbreviation,
    // Sub-mode of CompositionSelection?
    // Implies that state stack has at least one more deeper state for registration input
    // ▼たじゅうに【▼とうろくできる【こんなふうに】】
    Register,
}

impl FromStr for CompositionMode {
    type Err = CskkError;

    fn from_str(s: &str) -> Result<CompositionMode, CskkError> {
        match s {
            "Direct" => Ok(CompositionMode::Direct),
            "PreComposition" => Ok(CompositionMode::PreComposition),
            "PreCompositionOkurigana" => Ok(CompositionMode::PreCompositionOkurigana),
            "CompositionSelection" => Ok(CompositionMode::CompositionSelection),
            "Abbreviation" => Ok(CompositionMode::Abbreviation),
            "Register" => Ok(CompositionMode::Register),

            s => Err(CskkError::ParseError(s.to_string())),
        }
    }
}

pub fn has_rom2kana_conversion(input_mode: &InputMode, composition_mode: &CompositionMode) -> bool {
    match composition_mode {
        CompositionMode::Direct
        | CompositionMode::PreComposition
        | CompositionMode::PreCompositionOkurigana => matches!(
            input_mode,
            InputMode::Hiragana | InputMode::Katakana | InputMode::HankakuKatakana
        ),
        _ => false,
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub enum PeriodStyle {
    // Dumb naming for C FFI as C-style enum
    /// Use "。"  for "."
    PeriodJa,
    /// Use "．"  for "."
    PeriodEn,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C)]
pub enum CommaStyle {
    // Dumb naming for C FFI as C-style enum
    /// Use "、" for ","
    CommaJa,
    /// Use "，" for ","
    CommaEn,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serde_check() {
        let result = toml::to_string::<InputMode>(&InputMode::Hiragana).unwrap();
        assert_eq!(r#""Hiragana""#, result);
        let result = toml::from_str::<InputMode>(r#""Katakana""#).unwrap();
        assert_eq!(InputMode::Katakana, result);
    }
}
