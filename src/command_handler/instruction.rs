use crate::{CompositionMode, CskkError, InputMode};
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
    ForceKanaConvert(InputMode),
    ClearUnconvertedInputs,
    ClearKanaConvertedInputs,
    ClearUnconfirmedInputs,
    ChangeCompositionMode(CompositionMode),
    // モード変更などで入力を処理し、入力モードの入力としての処理をしない命令
    FinishKeyEvent,
    // keyeventを処理しなかったとして処理を終了する。ueno/libskkでの"*-unhandled"系命令用?
    PassthroughKeyEvent,
    // 次の候補を指そうとする。候補があればポインタを進めるが、無い場合はRegisterモードへ移行する。
    TryNextCandidate,
    // 前の候補を指そうとする。
    TryPreviousCandidate,
    // 変換候補ポインタを進める
    NextCandidatePointer,
    // 変換候補ポインタを戻す
    PreviousCandidatePointer,
    // 現在の変換候補で確定する
    ConfirmComposition,
    // かな変換のあるInputModeを指定し、漢字変換前のかなをそのモードで入力する
    ConfirmAs(InputMode),
    // Direct時に確定する。辞書編集時は動作があるのでEnterをイベント消費するが、そうでない場合はcskkでイベントを消費しない。
    ConfirmDirect,
    // 現在の候補を辞書から消す
    Purge,
    // 一文字消去する。消去可能時のみキー入力処理を終わる。
    Delete,
}
//
impl FromStr for Instruction {
    type Err = CskkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(simple_instruction) = match s {
            "Abort" => Some(Instruction::Abort),
            "ClearUnconvertedInputs" => Some(Instruction::ClearUnconvertedInputs),
            "ClearKanaConvertedInputs" => Some(Instruction::ClearKanaConvertedInputs),
            "ClearUnconfirmedInputs" => Some(Instruction::ClearUnconfirmedInputs),
            "FinishKeyEvent" => Some(Instruction::FinishKeyEvent),
            "NextCandidatePointer" => Some(Instruction::NextCandidatePointer),
            "PreviousCandidatePointer" => Some(Instruction::PreviousCandidatePointer),
            "ConfirmComposition" => Some(Instruction::ConfirmComposition),
            "ConfirmDirect" => Some(Instruction::ConfirmDirect),
            "Purge" => Some(Instruction::Purge),
            "Delete" => Some(Instruction::Delete),
            "TryNextCandidate" => Some(Instruction::TryNextCandidate),
            "TryPreviousCandidate" => Some(Instruction::TryPreviousCandidate),
            "PassthroughKeyEvent" => Some(Instruction::PassthroughKeyEvent),
            // 以下旧版の互換性維持のため。メジャーバージョンアップで消しうる。
            "ConfirmAsHiragana" => Some(Instruction::ConfirmAs(InputMode::Hiragana)),
            "ConfirmAsKatakana" => Some(Instruction::ConfirmAs(InputMode::Katakana)),
            "ConfirmAsJISX0201" => Some(Instruction::ConfirmAs(InputMode::HankakuKatakana)),
            "FlushPreviousCarryOver" => Some(Instruction::ClearUnconvertedInputs),
            "FlushConvertedKana" => Some(Instruction::ClearKanaConvertedInputs),
            "DeleteDirect" => Some(Instruction::Delete),
            "DeletePrecomposition" => Some(Instruction::Delete),
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
                "ChangeInputMode" => return Ok(Instruction::ChangeInputMode(input_mode)),
                "ForceKanaConvert" => return Ok(Instruction::ForceKanaConvert(input_mode)),
                "ConfirmAs" => return Ok(Instruction::ConfirmAs(input_mode)),
                // 旧版の互換性維持のため
                "OutputNNIfAny" => return Ok(Instruction::ForceKanaConvert(input_mode)),
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
