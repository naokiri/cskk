use crate::dictionary::dictionary_candidate::{CompletionCandidate, DictionaryCandidate};
use crate::dictionary::CompositeKey;

// CandidateListに持たせる情報。
// libskk vala Candidate classを元に、単体で送り仮名の厳密マッチの登録に必要な情報を持たせている。TODO: libskk 由来なので重複した情報を整理、valaなので外に見せすぎ、特にcomposite_keyに含まれる情報は不要かも
#[derive(Debug, Clone)]
pub struct Candidate {
    // 取り回しの都合上DictEntryと重複して持つ
    pub(crate) midashi: String,
    // 厳密な送り仮名のある場合、その送り仮名を持つ。
    pub(crate) strict_okuri: Option<String>,
    // 送り仮名の有無。strict_okuriがNoneでも送りありエントリはtrueとなる。
    pub(crate) okuri: bool,
    // Raw kouho_text that might include "#0回" etc
    pub(crate) kouho_text: String,
    pub(crate) annotation: Option<String>,
    // Output to show the candidate. "第#0回"が"第壱回"のように後処理されている想定。
    pub(crate) output: String,
}

impl Default for Candidate {
    fn default() -> Self {
        Candidate {
            midashi: "エラー".to_string(),
            strict_okuri: None,
            okuri: false,
            kouho_text: "エラー".to_string(),
            annotation: None,
            output: "エラー".to_string(),
        }
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        if self.midashi.eq(&other.midashi) && self.kouho_text.eq(&other.kouho_text) {
            return true;
        }
        false
    }
}

impl Candidate {
    pub(crate) fn new(
        midashi: String,
        strict_okuri: Option<String>,
        okuri: bool,
        kouho_text: String,
        annotation: Option<String>,
        output: String,
    ) -> Self {
        Candidate {
            midashi,
            strict_okuri,
            okuri,
            kouho_text,
            annotation,
            output,
        }
    }

    ///
    /// 辞書の候補からそのままの内容で候補リスト用のcandidateを返す。辞書登録等にも使われるため、入力されたcomposite_keyベースで見出し等を作る。
    ///
    pub(in crate::dictionary) fn from_dictionary_candidate(
        composite_key: &CompositeKey,
        dictionary_cand: &DictionaryCandidate,
    ) -> Self {
        Self {
            midashi: composite_key.get_dict_key(),
            strict_okuri: composite_key.get_okuri().to_owned(),
            okuri: composite_key.has_okuri(),
            kouho_text: dictionary_cand.kouho_text.to_owned(),
            annotation: dictionary_cand.annotation.to_owned(),
            output: dictionary_cand.kouho_text.to_owned(),
        }
    }

    ///
    /// 補完のため、辞書から与えられた内容から候補を作る。
    ///
    pub(in crate::dictionary) fn from_completion_candidate(
        completion_candidate: &CompletionCandidate,
    ) -> Self {
        Self {
            midashi: completion_candidate.midashi.to_string(),
            strict_okuri: completion_candidate.okuri.to_owned(),
            okuri: completion_candidate.okuri.is_some(),
            kouho_text: completion_candidate.kouho_text.to_owned(),
            annotation: completion_candidate.annotation.to_owned(),
            output: completion_candidate.kouho_text.to_owned(),
        }
    }
}
