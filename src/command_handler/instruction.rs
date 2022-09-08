use crate::{CompositionMode, CskkError, InputMode};
use regex::internal::Inst;
use regex::Regex;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

///
///
// FIXME: FinishKeyEvent, PassThroughKeyEvent, ConfirmDirect, DeleteDirect の4つのみキー処理を終わる可能性があるのを設定を書く時に間違えないよう明示的にしたい
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Instruction {
    // Abort current composition selection, registration
    Abort,
    ChangeInputMode(InputMode),
    // Try to convert preconversion if in input mode which has conversion. Mostly (or only?) just for single 'n' conversion.
    // (ctrl q)コマンドのように現在のinput_modeに依らない変換が必要なのでinputmodeも指定しなければならない。
    #[allow(clippy::upper_case_acronyms)]
    OutputNNIfAny(InputMode),
    FlushPreviousCarryOver,
    FlushConvertedKana,
    ClearUnconfirmedInputs,
    ChangeCompositionMode(CompositionMode),
    // モード変更などで入力を処理し、入力モードの入力としての処理をしない命令
    FinishKeyEvent,
    // keyeventを処理しなかったとして処理を終了する。ueno/libskkでの"*-unhandled"系命令用?
    #[allow(dead_code)]
    PassthroughKeyEvent,
    // 次の候補を指そうとする。候補があればポインタを進めるが、無い場合はRegisterモードへ移行する。
    TryNextCandidate,
    // 前の候補を指そうとする。
    TryPreviousCandidate,
    // 現在の変換候補リストを作りなおす
    UpdateCandidateList,
    // 変換候補ポインタを進める
    NextCandidatePointer,
    // 変換候補ポインタを戻す
    PreviousCandidatePointer,
    // 現在の変換候補で確定する
    ConfirmComposition,
    // 現在の変換前文字列を漢字変換せずに確定する。 Hiragana/Katakana/HankakuKatakana のみ
    ConfirmPreComposition(InputMode),
    // TODO: ConfirmAs(InputMode)
    ConfirmAsHiragana,
    ConfirmAsKatakana,
    #[allow(clippy::upper_case_acronyms)]
    ConfirmAsJISX0201,
    // Direct時に確定する。辞書編集時は動作があるのでEnterをイベント消費するが、そうでない場合はcskkでイベントを消費しない。
    ConfirmDirect,
    // 現在の候補を辞書から消す
    Purge,
    // PreComposition時に一文字消去する。
    // ueno/libskk StartStateHandler のdelete時？
    DeletePrecomposition,
    // Direct時に一文字消去する。消去可能時のみキー入力処理を終わる。
    // ueno/libskk NoneStateHandler のdelete時？
    DeleteDirect,
}
//
impl FromStr for Instruction {
    type Err = CskkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(simple_instruction) = match s {
            "Abort" => Some(Instruction::Abort),
            "FlushPreviousCarryOver" => Some(Instruction::FlushPreviousCarryOver),
            "FlushConvertedKana" => Some(Instruction::FlushConvertedKana),
            "ClearUnconfirmedInputs" => Some(Instruction::ClearUnconfirmedInputs),
            "FinishKeyEvent" => Some(Instruction::FinishKeyEvent),
            "UpdateCandidateList" => Some(Instruction::UpdateCandidateList),
            "NextCandidatePointer" => Some(Instruction::NextCandidatePointer),
            "PreviousCandidatePointer" => Some(Instruction::PreviousCandidatePointer),
            "ConfirmComposition" => Some(Instruction::ConfirmComposition),
            "ConfirmAsHiragana" => Some(Instruction::ConfirmAsHiragana),
            "ConfirmAsKatakana" => Some(Instruction::ConfirmAsKatakana),
            "ConfirmAsJISX0201" => Some(Instruction::ConfirmAsJISX0201),
            "ConfirmDirect" => Some(Instruction::ConfirmDirect),
            "Purge" => Some(Instruction::Purge),
            "DeletePrecomposition" => Some(Instruction::DeletePrecomposition),
            "DeleteDirect" => Some(Instruction::DeleteDirect),
            "TryNextCandidate" => Some(Instruction::TryNextCandidate),
            "TryPreviousCandidate" => Some(Instruction::TryPreviousCandidate),
            "PassthroughKeyEvent" => Some(Instruction::PassthroughKeyEvent),
            _ => None,
        } {
            return Ok(simple_instruction);
        }

        lazy_static! {
            static ref INPUT_MODE_REGEX: Regex =
                Regex::new(r"(.*)\((Hiragana|Katakana|HankakuKatakana|Zenkaku|Ascii)\)")
                    .expect("Instruction deserializer bug.");
        }

        let maybe_capture = INPUT_MODE_REGEX.captures(s);
        if let Some(capture) = maybe_capture {
            let input_mode = InputMode::from_str(&capture[2])
                .expect("Regex code is wrong in deserealizing input mode instruction");
            match &capture[1] {
                "OutputNNIfAny" => return Ok(Instruction::OutputNNIfAny(input_mode)),
                "ChangeInputMode" => return Ok(Instruction::ChangeInputMode(input_mode)),
                "ConfirmPreComposition" => {
                    return Ok(Instruction::ConfirmPreComposition(input_mode))
                }
                _ => {}
            }
        }

        lazy_static! {
            static ref COMPOSITION_MODE_REGEX: Regex =
                Regex::new(r"(.*)\((Direct|PreComposition|PreCompositionOkurigana|CompositionSelection|Abbreviation|Register)\)")
                    .expect("Instruction deserializer bug.");
        }
        let maybe_capture = COMPOSITION_MODE_REGEX.captures(s);
        if let Some(capture) = maybe_capture {
            let composition_mode = CompositionMode::from_str(&capture[2])
                .expect("Regex code is wrong in deserealizing composition mode instruction");
            #[allow(clippy::single_match)]
            match &capture[1] {
                "ChangeCompositionMode" => {
                    return Ok(Instruction::ChangeCompositionMode(composition_mode))
                }
                _ => {}
            }
        }

        Err(CskkError::ParseError(s.to_string()))
    }
}

impl<'de> Deserialize<'de> for Instruction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let result = Instruction::from_str(s);
        result.map_err(|_| Error::invalid_value(Unexpected::Str(s), &"invalid string"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Deserialize)]
    struct InstructionOnly {
        inst: Instruction,
    }

    #[test]
    fn deserialize() {
        let result = toml::from_str::<InstructionOnly>(r#"inst = "ConfirmDirect""#).unwrap();
        assert_eq!(Instruction::ConfirmDirect, result.inst);

        let result =
            toml::from_str::<InstructionOnly>(r#"inst = "ChangeInputMode(Ascii)""#).unwrap();
        assert_eq!(Instruction::ChangeInputMode(InputMode::Ascii), result.inst);
    }
}
