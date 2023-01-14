use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::dictionary_parser::CandidatePrototype;
use crate::Candidate;
use std::fmt::{Display, Formatter};

pub(in crate::dictionary) trait DictionaryEntry {
    fn get_kouho_text(&self) -> &str;
    fn get_annotation(&self) -> &Option<String>;
}

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

impl Display for DictionaryCandidate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.annotation.is_some() {
            write!(
                f,
                "{};{}",
                DictEntry::escape_dictionary_string(&self.kouho_text),
                DictEntry::escape_dictionary_string(self.annotation.as_ref().unwrap())
            )
        } else {
            write!(
                f,
                "{}",
                DictEntry::escape_dictionary_string(&self.kouho_text)
            )
        }
    }
}

impl DictionaryEntry for DictionaryCandidate {
    fn get_kouho_text(&self) -> &str {
        &self.kouho_text
    }

    fn get_annotation(&self) -> &Option<String> {
        &self.annotation
    }
}

// 補完データ
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub(in crate::dictionary) struct CompletionCandidate {
    pub(in crate::dictionary) midashi: String,
    pub(in crate::dictionary) okuri: Option<String>,
    // Raw kouho_text that might include "#0回" etc
    pub(in crate::dictionary) kouho_text: String,
    pub(in crate::dictionary) annotation: Option<String>,
}

impl CompletionCandidate {
    pub fn from_dictionary_candidate(
        midashi: &str,
        okuri: &Option<String>,
        dictionary_candidate: &DictionaryCandidate,
    ) -> Self {
        Self {
            midashi: midashi.to_string(),
            okuri: okuri.to_owned(),
            kouho_text: dictionary_candidate.kouho_text.to_owned(),
            annotation: dictionary_candidate.annotation.to_owned(),
        }
    }
}

impl DictionaryEntry for CompletionCandidate {
    fn get_kouho_text(&self) -> &str {
        &self.kouho_text
    }

    fn get_annotation(&self) -> &Option<String> {
        &self.annotation
    }
}
