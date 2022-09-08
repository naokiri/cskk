use crate::candidate_list::CandidateList;
use crate::dictionary::candidate::Candidate;
use crate::form_changer::KanaFormChanger;
use crate::skk_modes::{CompositionMode, InputMode};
use std::fmt::{Debug, Formatter};
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
    converted_kana_to_composite: String,
    // 未確定入力の漢字の読み以外の部分。多くの場合送り仮名であり、その想定のもとに変数名を付けてしまったが、auto_start_henkan等の強制的に変換を開始する場合にはおくりがな以外のpostfixが入ることもある。convertがあるInputMode時のみ使用
    converted_kana_to_okuri: String,
    // 現在の変換候補リスト
    candidate_list: CandidateList,
    // 入力を漢字変換した現在の選択肢の送り仮名部分。 TODO: 保持せずにconverted_kana_to_okuriで良い？
    composited_okuri: String,
    // 確定済み入力列。pollされた時に渡してflushされるもの。
    confirmed: String,
    // 今のかな変換の間に大文字でモード変更をしたかどうか。このステートによってシフトを押したままキー入力をしてしまった時に連続してモード変更しないようにしている。
    capital_transition: bool,
    // 現在送り仮名を入力しているかどうか。postfixを送り仮名として用いるべきかどうか。
    use_okurigana: bool,
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
            capital_transition: false,
            use_okurigana: false,
        }
    }

    /// 現在の確定済み文字列を取得する
    pub(crate) fn get_confirmed_string(&self) -> &str {
        &self.confirmed
    }

    /// 現在の変換前文字列を取得する
    pub(crate) fn get_to_composite_string(&self) -> &str {
        &self.converted_kana_to_composite
    }

    /// 現在の送り文字列を取得する。
    pub(crate) fn get_okuri_string(&self) -> &str {
        &self.converted_kana_to_okuri
    }

    /// 入力した中のかな変換前の入力を全て消す。かな変換済みのものは消えない。
    pub(crate) fn clear_preconverted_kanainputs(&mut self) {
        self.pre_conversion.clear();
        self.capital_transition = false;
    }

    /// 入力した中のかな入力を全て消す。漢字変換済みのものは消えない。
    pub(crate) fn clear_kanas(&mut self) {
        self.clear_preconverted_kanainputs();
        self.converted_kana_to_composite.clear();
        self.converted_kana_to_okuri.clear();

        self.use_okurigana = false;
    }

    /// 確定済みではない入力を全て消す。
    pub(crate) fn clear_unconfirmed(&mut self) {
        self.raw_to_composite.clear();
        self.clear_candidate_list();
        self.clear_kanas();
    }

    /// 確定済みのものも含めて入力を全て消す
    pub(crate) fn clear_all(&mut self) {
        self.confirmed.clear();
        self.clear_unconfirmed();
    }

    /// 現在の確定済み文字列のみを消去する
    pub(crate) fn flush_confirmed_string(&mut self) {
        self.confirmed.clear();
    }

    /// 現在のモードで1文字消去する。
    /// 1文字でも消去されたら、trueを返す。
    /// 何も処理されなかったら、falseを返す。
    pub(crate) fn delete(&mut self) -> bool {
        match self.composition_mode {
            CompositionMode::Direct => {
                // かな変換前の入力を1文字消そうとする
                let mut deleted = self.pre_conversion.pop().is_some();
                // できなければ確定済み文字列を1文字消そうとする。
                if !deleted {
                    deleted = self.confirmed.pop().is_some();
                }
                deleted
            }
            CompositionMode::PreComposition => {
                // かな変換前の入力を1文字消そうとする
                let mut deleted = self.pre_conversion.pop().is_some();
                // できなければ未確定かなを1文字消そうとする。
                if !deleted {
                    deleted = self.converted_kana_to_composite.pop().is_some();
                    self.raw_to_composite.pop();
                }
                // それもできなければ初めてDirectにモード変更する。未確定文字0文字状態が許容される。
                if !deleted {
                    self.composition_mode = CompositionMode::Direct;
                }
                deleted
            }
            CompositionMode::PreCompositionOkurigana => {
                // かな変換前の入力を1文字消そうとする
                let mut deleted = self.pre_conversion.pop().is_some();
                // できなければおくりがなを1文字消そうとする
                if !deleted {
                    deleted = self.converted_kana_to_okuri.pop().is_some();
                    self.raw_to_composite.pop();
                }
                // 結果として送り仮名もかな変換前の入力もなくなったら、PreCompositionにモード変更する。送り0文字状態を許容しない。
                if self.pre_conversion.is_empty() && self.converted_kana_to_okuri.is_empty() {
                    self.composition_mode = CompositionMode::PreComposition;
                }
                deleted
            }
            _ => {
                unimplemented!();
            }
        }
    }

    /// 現在のcompositionmodeで変換済み文字列を入力する。
    /// PreComposition等ではひらがな変換を想定、CompositionSelectionでは漢字変換済み文字列を想定。
    pub(crate) fn push_string(&mut self, letter_or_word: &str) {
        self.push_string_for_composition_mode(letter_or_word, self.composition_mode)
    }

    /// 指定のCompositionModeで変換済み文字列を入力する。
    pub(crate) fn push_string_for_composition_mode(
        &mut self,
        letter_or_word: &str,
        composition_mode: CompositionMode,
    ) {
        match composition_mode {
            CompositionMode::Direct => {
                self.confirmed.push_str(letter_or_word);
            }
            CompositionMode::PreComposition => {
                self.converted_kana_to_composite.push_str(letter_or_word);
                self.raw_to_composite.push_str(letter_or_word);
            }
            CompositionMode::PreCompositionOkurigana => {
                self.converted_kana_to_okuri.push_str(letter_or_word);
                self.use_okurigana = true;
            }
            CompositionMode::Abbreviation => {
                // TODO: Abbreviationモード実装
            }
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

    /// 現在の漢字変換前の本体とおくりがなの文字列をまとめて漢字変換前の文字列にする
    /// 例: ▼悲し -> ▽かなし とする時のかな文字の操作。
    /// Abort時のみのはず。
    pub(crate) fn consolidate_converted_to_to_composite(&mut self) {
        let okuri = self.converted_kana_to_okuri.to_owned();
        // these 2 lines should be a method later
        self.converted_kana_to_composite.push_str(&okuri);
        self.raw_to_composite.push_str(&okuri);

        self.converted_kana_to_okuri.clear();
        self.use_okurigana = false;
    }

    /// 送り仮名ではないが変換語の後につけるべき文字列を設定する。
    pub(crate) fn set_converted_to_postfix(&mut self, letter_or_word: &str) {
        self.converted_kana_to_okuri = letter_or_word.to_string();
        self.use_okurigana = false;
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

    /// 今のステートで変換する時の辞書のキーとして使うべき文字列を返す。
    pub(crate) fn get_composite_key(&self) -> String {
        // ローマ字ベースではない入力規則に対応するため、送り仮名の最初の文字はひらがなから対応表を引く。
        if self.use_okurigana {
            // ひらがなはUnicode Scalar Valueなのでchars()で十分。
            if let Some(first_kana) = self.converted_kana_to_okuri.chars().next() {
                if let Some(okuri_first) =
                    KanaFormChanger::kana_to_okuri_prefix(&first_kana.to_string())
                {
                    let mut string = self.raw_to_composite.to_owned();
                    string.push_str(okuri_first);
                    return string;
                }
            }
        }

        self.raw_to_composite.to_owned()
    }

    // FIXME: 本来はおくりがな等のセットで自動的にセットしたい
    pub(crate) fn set_capital_transition(&mut self, has_transitioned: bool) {
        self.capital_transition = has_transitioned;
    }

    pub(crate) fn get_capital_transition(&self) -> bool {
        self.capital_transition
    }
}

// candidate_lists
impl CskkState {
    pub(crate) fn get_candidate_list(&self) -> &CandidateList {
        &self.candidate_list
    }

    pub(crate) fn forward_candidate(&mut self) -> bool {
        self.candidate_list.forward_candidate()
    }

    pub(crate) fn backward_candidate(&mut self) -> bool {
        self.candidate_list.backward_candidate()
    }

    /// iが範囲内ならばポインタをその位置にしてtrueを返す。範囲外ならばなにもせずfalseを返す。
    pub(crate) fn set_candidate_pointer_index(&mut self, i: usize) -> bool {
        self.candidate_list.set_selection_pointer(i)
    }

    /// 現在の変換に候補を追加し、追加したうちの最初の候補を選択した状態にする。
    /// candidate_listにすでにto_compositeが存在することが暗黙の前提
    pub(crate) fn add_new_candidates_for_existing_string_to_composite(
        &mut self,
        candidates: Vec<Candidate>,
    ) {
        self.candidate_list.add_new_candidates(candidates);
        self.composited_okuri = self.converted_kana_to_okuri.to_string();
    }

    pub(crate) fn clear_candidate_list(&mut self) {
        self.candidate_list.clear();
        self.composited_okuri.clear();
    }

    /// 現在の変換候補を設定し、最初の候補を指す
    pub(crate) fn set_new_candidate_list(&mut self, candidates: Vec<Candidate>) {
        let raw_to_composite = self.get_composite_key();
        self.candidate_list.set(raw_to_composite, candidates);
        self.composited_okuri = self.converted_kana_to_okuri.to_string();
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
            .field("use_okurigana", &self.use_okurigana)
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
            composited_okuri: "".to_string(),
            confirmed: "".to_string(),
            candidate_list: CandidateList::new(),
            capital_transition: false,
            use_okurigana: false,
        }
    }
}
