use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::dictionary_parser::CandidatePrototype;
use crate::Candidate;

// Candidateの辞書内のデータ。
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub(in crate::dictionary) struct DictionaryCandidate {
    // Raw kouho_text that might include "#0回" etc
    pub(in crate::dictionary) kouho_text: String,
    pub(in crate::dictionary) annotation: Option<String>,
}

impl DictionaryCandidate {
    pub(in crate::dictionary) fn from_candidate_prototype(
        candidate_prototype: &CandidatePrototype,
    ) -> Self {
        let kouho_text = DictEntry::process_lisp_fun(candidate_prototype.kouho);
        let annotation = candidate_prototype
            .annotation
            .map(DictEntry::process_lisp_fun);

        Self {
            kouho_text,
            annotation,
        }
    }

    // 送り仮名の厳密でないマッチから送り仮名の厳密マッチで新たに登録する際など。
    /// 候補リスト内のcandidateから新たにdictionary内部表現のcandidateを作る。
    pub(in crate::dictionary) fn from_candidate(candidate: &Candidate) -> Self {
        let kouho_text = candidate.kouho_text.to_owned();
        let annotation = candidate.annotation.to_owned();

        Self {
            kouho_text,
            annotation,
        }
    }
}
