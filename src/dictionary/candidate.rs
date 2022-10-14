use crate::dictionary::dictionary_candidate::DictionaryCandidate;
use crate::dictionary::CompositeKey;

// CandidateListに持たせる情報。
// libskk vala Candidate classを元に、単体で送り仮名の厳密マッチの登録に必要な情報を持たせている。TODO: libskk 由来なので重複した情報を整理、valaなので外に見せすぎ、特にcomposite_keyに含まれる情報は不要かも
#[derive(Debug, Clone)]
pub struct Candidate {
    // 取り回しの都合上DictEntryと重複して持つ
    pub(crate) midashi: String,
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
        okuri: bool,
        kouho_text: String,
        annotation: Option<String>,
        output: String,
    ) -> Self {
        Candidate {
            midashi,
            okuri,
            kouho_text,
            annotation,
            output,
        }
    }

    ///
    /// 辞書の候補からそのままの内容で候補リスト用のcandidateを返す。
    ///
    pub(in crate::dictionary) fn from_dictionary_candidate(
        composite_key: &CompositeKey,
        dictionary_cand: &DictionaryCandidate,
    ) -> Self {
        Self {
            midashi: composite_key.get_dict_key(),
            okuri: composite_key.has_okuri(),
            kouho_text: dictionary_cand.kouho_text.to_owned(),
            annotation: dictionary_cand.annotation.to_owned(),
            output: dictionary_cand.kouho_text.to_owned(),
        }
    }
}
