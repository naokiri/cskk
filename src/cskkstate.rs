use crate::candidate_list::CandidateList;
use crate::dictionary::candidate::Candidate;
use crate::kana_form_changer::KanaFormChanger;
use crate::skk_modes::{CompositionMode, InputMode};
use std::iter::FromIterator;

// FIXME: 全部1ファイルで収めていた頃の名残りで多くのステートのみ操作系メソッドがcontext内のままなので、できれば移してフィールドをpub(crate)からprivateにして何がステート操作かわかりやすくする
/// Rough prototype yet.
///
#[derive(Debug)]
pub(crate) struct CskkState {
    pub(crate) input_mode: InputMode,
    pub(crate) composition_mode: CompositionMode,
    // 直前のCompositionMode。Abort時に元に戻すmode。
    pub(crate) previous_composition_mode: CompositionMode,
    // 入力文字で、かな確定済みでないものすべて
    pub(crate) pre_conversion: Vec<char>,
    // 変換辞書のキーとなる部分。送りなし変換の場合はconverted_kana_to_composite と同じ。送りあり変換時には加えてconverted_kana_to_okuriの一文字目の子音や'>'付き。Abbrebiation変換の場合kana-convertされる前の入力など
    pub(crate) raw_to_composite: String,
    // 未確定入力の漢字の読み部分。この時点ではひらがな。出力時にInputModeにあわせて変換される。convertがあるInputMode時のみ使用
    pub(crate) converted_kana_to_composite: String,
    // 未確定入力のおくり仮名部分。常にひらがな。convertがあるInputMode時のみ使用
    pub(crate) converted_kana_to_okuri: String,
    // 現在の変換候補リスト
    pub(crate) candidate_list: CandidateList,
    // 入力を漢字変換した現在の選択肢の送り仮名部分。 TODO: 保持せずにconverted_kana_to_okuriで良い？
    pub(crate) composited_okuri: String,
    // 確定済み入力列。pollされた時に渡してflushされるもの。
    pub(crate) confirmed: String,
}

impl CskkState {
    pub fn new(input_mode: InputMode, composition_mode: CompositionMode) -> Self {
        CskkState {
            input_mode,
            composition_mode,
            previous_composition_mode: composition_mode,
            pre_conversion: vec![],
            raw_to_composite: "".to_string(),
            converted_kana_to_composite: "".to_string(),
            converted_kana_to_okuri: "".to_string(),
            composited_okuri: "".to_string(),
            confirmed: "".to_string(),
            candidate_list: CandidateList::new(),
        }
    }

    pub(crate) fn preedit_string(
        &self,
        kana_form_changer: &KanaFormChanger,
        current_input_mode: InputMode,
    ) -> String {
        let converted = &self.confirmed;
        let unconverted = &self.pre_conversion;
        let kana_to_composite = kana_form_changer
            .adjust_kana_string(current_input_mode, &self.converted_kana_to_composite);
        let kana_to_okuri =
            kana_form_changer.adjust_kana_string(current_input_mode, &self.converted_kana_to_okuri);
        let current_candidate = self.candidate_list.get_current_candidate();
        let fallback_candidate = Candidate::default();
        let composited = &current_candidate.unwrap_or(&fallback_candidate).kouho_text;
        let composited_okuri =  kana_form_changer.adjust_kana_string(current_input_mode, &self.composited_okuri);

        match self.composition_mode {
            CompositionMode::Direct => {
                converted.to_owned() + &String::from_iter(unconverted.iter())
            }
            CompositionMode::PreComposition => {
                "▽".to_owned()
                    + converted
                    + &kana_to_composite
                    + &String::from_iter(unconverted.iter())
            }
            CompositionMode::PreCompositionOkurigana => {
                "▽".to_owned()
                    + converted
                    + &kana_to_composite
                    + "*"
                    + &kana_to_okuri
                    + &String::from_iter(unconverted.iter())
            }
            CompositionMode::CompositionSelection => "▼".to_owned() + composited + &composited_okuri,
            CompositionMode::Register => {
                if kana_to_okuri.is_empty() {
                    "▼".to_string() + &kana_to_composite
                } else {
                    "▼".to_string() + &kana_to_composite + "*" + &kana_to_okuri
                }
            }
            CompositionMode::Abbreviation => {
                "Abbreviaton mode. detail not implemented.".to_string()
            }
        }
    }
}
