use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::dictionary_candidate::DictionaryCandidate;
use crate::dictionary::CompositeKey;
use std::fmt::Write;

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

    // pub(crate) fn from_skk_jisyo_string(
    //     midashi: &str,
    //     raw_entry: &str,
    //     has_okuri: bool,
    // ) -> Result<Self, CskkError> {
    //     let mut chunk = raw_entry.split(';');
    //     if let Some(text) = chunk.next() {
    //         let kouho = DictEntry::process_lisp_fun(text);
    //         let annotation = chunk.next().map(DictEntry::process_lisp_fun);
    //         Ok(Candidate::new(
    //             midashi.to_string(),
    //             has_okuri,
    //             kouho.to_string(),
    //             annotation,
    //             kouho,
    //         ))
    //     } else {
    //         log::debug!("Failed to parse candidate from: {:?}", raw_entry);
    //         Err(CskkError::Error("No candidate".to_string()))
    //     }
    // }

    ///
    /// 辞書の候補からそのままの内容で候補リスト用のcandidateを返す。
    ///
    pub(in crate::dictionary) fn from_dictionary_candidate(
        composite_key: &CompositeKey,
        dictionary_cand: &DictionaryCandidate,
    ) -> Self {
        Self {
            midashi: composite_key.get_to_composite().to_string(),
            okuri: composite_key.has_okuri(),
            kouho_text: dictionary_cand.kouho_text.to_owned(),
            annotation: dictionary_cand.annotation.to_owned(),
            output: dictionary_cand.kouho_text.to_owned(),
        }
    }

    // entry string between '/'
    // {候補};アノテーション
    // {候補};*アノテーション
    // TODO: 将来的には [{優先送り仮名}/{候補}] のような優先送り仮名エントリも扱えると嬉しい
    pub(crate) fn to_skk_jisyo_string(&self) -> String {
        let mut result = String::new();
        result.push_str(&DictEntry::escape_dictionary_string(
            self.kouho_text.as_str(),
        ));
        if let Some(annotation) = &self.annotation {
            write!(
                result,
                ";{}",
                &DictEntry::escape_dictionary_string(annotation.as_str())
            )
            .expect("Failed to allocate jisyo string for candidate.");
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn skk_jisyo_string_no_annotation() {
        let candidate = Candidate::new(
            "みだし".to_string(),
            false,
            "候補".to_string(),
            None,
            "候補".to_string(),
        );
        assert_eq!("候補", candidate.to_skk_jisyo_string())
    }
}
