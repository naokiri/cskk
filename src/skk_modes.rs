/// Rough design prototype yet
/// SKKの入力モード
/// DDSKK 16.2 マニュアル 4.2 に依る
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

/// Rough design prototype yet
/// SKKの変換モード
/// DDSKK 16.2 マニュアル 4.3 に依る
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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
