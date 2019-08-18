use std::fmt::Display;

/// Rough design prototype yet
/// SKKの入力モード
/// DDSKK 16.2 マニュアル 4.2 に依る
#[derive(Debug, Display, PartialEq, Eq, Hash)]
pub(crate) enum InputMode {
    // かなモード
    #[allow(dead_code)]
    Hiragana,
    // カナモード
    #[allow(dead_code)]
    Katakana,
    // JIS X 0201 カナ、 いわゆる半角カナ。 DDSKKでは独立したモード扱いではないので実装未定
    #[allow(dead_code)]
    HankakuKatakana,
    // 全英モード
    #[allow(dead_code)]
    Zenkaku,
    // アスキーモード
    #[allow(dead_code)]
    Ascii,
}

/// Rough design prototype yet
/// SKKの変換モード
/// DDSKK 16.2 マニュアル 4.3 に依る
#[derive(Debug, Display, PartialEq, Eq, Hash)]
pub(crate) enum CompositionMode {
    // ■モード
    #[allow(dead_code)]
    Direct,
    // ▽モード 見出しモード
    PreComposition,
    // ▽モードの送り仮名開始時
    #[allow(dead_code)]
    PreCompositionOkurigana,
    // ▼モード
    CompositionSelection,
    // SKK abbrev mode: Sub-mode of PreComposition
    #[allow(dead_code)]
    Abbreviation,
    // Sub-mode of CompositionSelection
    #[allow(dead_code)]
    Register(Box<CompositionMode>),
}