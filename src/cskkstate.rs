use crate::candidate_list::CandidateList;
use crate::dictionary::candidate::Candidate;
use crate::form_changer::KanaFormChanger;
use crate::skk_modes::{CompositionMode, InputMode};
use std::fmt::{Debug, Formatter};
use std::ops::Add;
use xkbcommon::xkb;
use xkbcommon::xkb::{keysym_get_name, Keysym};

// FIXME: 全部1ファイルで収めていた頃の名残りで多くのステートのみ操作系メソッドがcontext内のままなので、できれば移してフィールドをpub(crate)からprivateにして何がステート操作かわかりやすくする
/// Rough prototype yet.
///
pub(crate) struct CskkState {
    pub(crate) input_mode: InputMode,
    pub(crate) composition_mode: CompositionMode,
    // 直前のCompositionMode。Abort時に元に戻すmode。
    pub(crate) previous_composition_mode: CompositionMode,
    // 入力文字で、かな確定済みでないものすべて
    pub(crate) pre_conversion: Vec<Keysym>,
    // 変換辞書のキーとなる部分。送りなし変換の場合はconverted_kana_to_composite と同じ。
    // 送りあり変換時には'>'なども付く。Abbrebiation変換の場合kana-convertされる前の入力など
    // 送り仮名の最初の文字は含まれない。
    // そのまま所持せず、計算して出すようにしたいが現バージョンでは保持している。
    raw_to_composite: String,
    // 未確定入力の漢字の読み部分。ひらがな。出力時にInputModeにあわせて変換される。convertがあるInputMode時のみ使用
    pub(crate) converted_kana_to_composite: String,
    // 未確定入力の漢字の読み以外の部分。多くの場合送り仮名だが、auto_start_henkan等の強制的に変換を開始する場合にはおくりがな以外が入ることもある。convertがあるInputMode時のみ使用
    pub(crate) converted_kana_to_okuri: String,
    // 未確定入力のおくり仮名の最初の文字。
    pub(crate) okuri_first_letter: Option<char>,
    // 現在の変換候補リスト
    pub(crate) candidate_list: CandidateList,
    // 入力を漢字変換した現在の選択肢の送り仮名部分。 TODO: 保持せずにconverted_kana_to_okuriで良い？
    pub(crate) composited_okuri: String,
    // 確定済み入力列。pollされた時に渡してflushされるもの。
    confirmed: String,
    // 今のかな変換の間に大文字でモード変更をしたかどうか。このステートによってシフトを押したままキー入力をしてしまった時に連続してモード変更しないようにしている。
    capital_transition: bool,
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
            okuri_first_letter: None,
            composited_okuri: "".to_string(),
            confirmed: "".to_string(),
            candidate_list: CandidateList::new(),
            capital_transition: false,
        }
    }

    /// 現在の確定済み文字列を取得する
    pub(crate) fn get_confirmed_string(&self) -> &str {
        &self.confirmed
    }

    /// 現在の確定済み文字列を取得する
    pub(crate) fn flush_confirmed_string(&mut self) {
        self.confirmed.clear();
    }

    /// charを入力する。Directモードのみ起きうる動作である。
    pub(crate) fn push_char(&mut self, ch: char) {
        if self.composition_mode == CompositionMode::Direct {
            self.confirmed.push(ch);
        } else {
            log::error!(
                "Tried to append char on not direct mode. compositionmode: {:?}, ignored char: {}",
                self.composition_mode,
                ch
            )
        }
    }

    /// 現在のモードで1文字消去する。
    pub(crate) fn delete(&mut self) {
        match self.composition_mode {
            CompositionMode::Direct => {
                if !self.confirmed.is_empty() {
                    self.confirmed.pop();
                }
            }
            _ => {
                unimplemented!();
            }
        }
    }

    /// 今のcompositionmodeで変換済み文字列を入力する。
    /// composition等ではひらがな変換を想定、CompositionSelectionでは漢字変換済み文字列を想定。
    pub(crate) fn push_letter_or_word(&mut self, letter_or_word: &str) {
        self.push_letter_or_word_for_composition_mode(letter_or_word, self.composition_mode)
    }

    /// 指定のCompositionModeで変換済み文字列を入力する。
    pub(crate) fn push_letter_or_word_for_composition_mode(
        &mut self,
        letter_or_word: &str,
        composition_mode: CompositionMode,
    ) {
        match composition_mode {
            CompositionMode::Direct => {
                self.confirmed.push_str(letter_or_word);
            }
            CompositionMode::PreComposition => {}
            CompositionMode::PreCompositionOkurigana => {}
            CompositionMode::Abbreviation => {}
            CompositionMode::CompositionSelection => {
                self.confirmed.push_str(letter_or_word);
            }
            _ => {
                log::error!(
                    "Tried to enter kana in mode {:?}. This should never happen. Ignored kana input {}.",
                    self.composition_mode,
                    letter_or_word
                )
            }
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
        let composited = &current_candidate.unwrap_or(&fallback_candidate).output;
        let composited_okuri =
            kana_form_changer.adjust_kana_string(current_input_mode, &self.composited_okuri);

        match self.composition_mode {
            CompositionMode::Direct => {
                converted.to_owned()
                    + &unconverted
                        .iter()
                        .map(|keysym| xkb::keysym_get_name(*keysym))
                        .collect::<Vec<_>>()
                        .join("")
            }
            CompositionMode::PreComposition => {
                "▽".to_owned()
                    + converted
                    + &kana_to_composite
                    //+ &unconverted.iter().collect::<String>()
                + &unconverted.iter().map(|keysym| xkb::keysym_get_name(*keysym)).collect::<Vec<_>>().join("")
            }
            CompositionMode::PreCompositionOkurigana => {
                "▽".to_owned()
                    + converted
                    + &kana_to_composite
                    + "*"
                    + &kana_to_okuri
                    + &unconverted
                        .iter()
                        .map(|keysym| xkb::keysym_get_name(*keysym))
                        .collect::<Vec<_>>()
                        .join("")
                // + &unconverted.iter().collect::<String>()
            }
            CompositionMode::CompositionSelection => {
                "▼".to_owned() + composited + &composited_okuri
            }
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

    pub(crate) fn get_converted_kana_to_okuri(&self) -> &str {
        &self.converted_kana_to_okuri
    }

    /// 今のステートで変換する時の辞書のキーとして使うべき文字列を返す。
    pub(crate) fn get_composite_key(&self) -> String {
        if let Some(c) = self.okuri_first_letter {
            let mut s = self.raw_to_composite.to_owned();
            s.push(c);
            return s;
        }
        // // ローマ字ベースではない入力規則のため、送り仮名の最初の文字は後から付ける。
        // // if self.is_okuri_entered {
        // // ひらがなはUnicode Scalar Valueなのでchars()で十分。
        // if let Some(first_kana) = self.converted_kana_to_okuri.chars().next() {
        //     if let Some(okuri_first) =
        //     KanaFormChanger::kana_to_okuri_prefix(&first_kana.to_string())
        //     {
        //         let string = self.raw_to_composite.to_owned().add(okuri_first);
        //         return string;
        //     }
        // }
        // // }

        self.raw_to_composite.to_owned()
    }

    // FIXME: 本来はstate内での齟齬を防ぐために、これをpub(crate)ではなくして個別で操作できないように他の内容を操作するメソッドで同時管理したい。
    pub(crate) fn append_raw_to_composite(&mut self, str: &str) {
        self.raw_to_composite.push_str(str);
    }

    // FIXME: 本来は他の状態を変更するメソッドのみで、composite_keyは計算して出す値にしたい。
    /// delete 1 letter from raw_to_composite
    pub(crate) fn delete_char_from_raw_to_composite(&mut self) {
        self.raw_to_composite.pop();
    }

    // FIXME: 本来は他の状態を変更するメソッドのみで、composite_keyは計算して出す値にしたい。
    pub(crate) fn clear_raw_to_composite(&mut self) {
        self.raw_to_composite.clear();
    }

    pub(crate) fn try_set_okuri_first_letter(&mut self, c: char) {
        if self.okuri_first_letter == None {
            self.okuri_first_letter = Some(c);
        }
    }

    pub(crate) fn clear_okuri_first_letter(&mut self) {
        self.okuri_first_letter = None;
    }

    // FIXME: 本来はおくりがな等のセットで自動的にセットしたい
    pub(crate) fn set_capital_transition(&mut self, has_transitioned: bool) {
        self.capital_transition = has_transitioned;
    }

    pub(crate) fn get_capital_transition(&self) -> bool {
        self.capital_transition
    }
}

impl Debug for CskkState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let keysyms: Vec<String> = self
            .pre_conversion
            .iter()
            .map(|x| keysym_get_name(*x))
            .collect();
        f.debug_struct("CskkState")
            .field("Mode", &(&self.composition_mode, &self.input_mode))
            .field("previous_composition_mode", &self.previous_composition_mode)
            .field("pre_conversion", &keysyms)
            .field("raw_to_composite", &self.raw_to_composite)
            .field(
                "converted_kana_to_composite",
                &self.converted_kana_to_composite,
            )
            .field("converted_kana_to_okuri", &self.converted_kana_to_okuri)
            .field("okuri_first_letter", &self.okuri_first_letter)
            .field("composited_okuri", &self.composited_okuri)
            .field("confirmed", &self.confirmed)
            .field("capital_transition", &self.capital_transition)
            .field("candidate_list", &self.candidate_list)
            .finish()
    }
}

#[cfg(test)]
impl CskkState {
    pub fn new_test_state(
        input_mode: InputMode,
        composition_mode: CompositionMode,
        pre_conversion: Vec<Keysym>,
    ) -> Self {
        Self {
            input_mode,
            composition_mode,
            previous_composition_mode: composition_mode,
            pre_conversion,
            raw_to_composite: "".to_string(),
            converted_kana_to_composite: "".to_string(),
            converted_kana_to_okuri: "".to_string(),
            okuri_first_letter: None,
            composited_okuri: "".to_string(),
            confirmed: "".to_string(),
            candidate_list: CandidateList::new(),
            capital_transition: false,
        }
    }
}
