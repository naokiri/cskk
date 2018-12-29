use std::fmt::Display;

/// Rough design prototype yet
/// SKKの入力モード
/// DDSKK 16.2 マニュアル 4.2 に依る
#[derive(Debug, Display, PartialEq, Eq, Hash)]
pub(crate) enum InputMode {
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
#[derive(Debug, Display, PartialEq, Eq, Hash)]
pub(crate) enum CompositionMode {
    // ■モード
    Direct,
    // ▽モード
    PreComposition,
    // ▽モードの送り仮名開始時
    PreCompositionOkurigana,
    // ▼モード
    CompositionSelection,
    // SKK abbrev mode: Sub-mode of PreComposition
    Abbreviation,
    // Sub-mode of CompositionSelection
    Register(Box<CompositionMode>),
}

pub(crate) struct SkkMode {
    input_mode: InputMode,
    composition_mode: CompositionMode,
}

