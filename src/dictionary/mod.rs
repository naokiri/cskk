use static_dict::StaticFileDict;

use crate::dictionary::candidate::Candidate;
use crate::error::CskkError;
use dictentry::DictEntry;
use empty_dict::EmptyDictionary;
use std::sync::{Arc, Mutex};
use user_dictionary::UserDictionary;

pub(crate) mod candidate;
pub(crate) mod dictentry;
pub mod empty_dict;
pub(crate) mod file_dictionary;
pub mod static_dict;
pub mod user_dictionary;

use log::warn;

// C側に出す関係でSizedである必要があり、dyn Traitではなくenumでラップする。
#[derive(Debug)]
pub enum CskkDictionaryType {
    StaticFile(StaticFileDict),
    UserFile(UserDictionary),
    EmptyDict(EmptyDictionary),
}

// impl Dictionary for Arc<Mutex<CskkDictionaryType>>とかでもう少し透過的にできないか？
pub type CskkDictionary = Mutex<CskkDictionaryType>;

/// confirm the candidate.
/// This updates writable dictionaries candidate order or add new entry which confirmed.
/// Returns true if updated the dictionary.
pub(crate) fn confirm_candidate(
    dictionary: &mut Arc<CskkDictionary>,
    candidate: &Candidate,
) -> Result<bool, CskkError> {
    if let Ok(mut mut_dictionary) = dictionary.lock() {
        return match *mut_dictionary {
            CskkDictionaryType::StaticFile(ref mut dict) => dict.select_candidate(candidate),
            CskkDictionaryType::UserFile(ref mut dict) => dict.select_candidate(candidate),
            CskkDictionaryType::EmptyDict(ref mut dict) => dict.select_candidate(candidate),
        };
    }
    Ok(false)
}

/// purge the candidate.
/// This updates writable dictionaries candidate order or add new entry which confirmed.
/// Returns true if updated the dictionary.
pub(crate) fn purge_candidate(
    dictionary: &mut Arc<CskkDictionary>,
    candidate: &Candidate,
) -> Result<bool, CskkError> {
    if let Ok(mut mut_dictionary) = dictionary.lock() {
        return match *mut_dictionary {
            CskkDictionaryType::StaticFile(ref mut dict) => dict.purge_candidate(candidate),
            CskkDictionaryType::UserFile(ref mut dict) => dict.purge_candidate(candidate),
            CskkDictionaryType::EmptyDict(ref mut dict) => dict.purge_candidate(candidate),
        };
    }

    Ok(false)
}

/// 現在ueno/libskk同様にDedupはkouho_textのみ、候補の順序はdictの順番通り。
/// annotationについては特に決めていないが、現在のところsortの仕様により先の候補が優先される。
pub(crate) fn get_all_candidates(
    dictionaries: &[Arc<CskkDictionary>],
    raw_to_composite: &str,
) -> Vec<Candidate> {
    let mut deduped_candidates = vec![];
    let mut ordered_candidates = vec![];

    for cskkdict in dictionaries.iter() {
        if let Ok(lock) = cskkdict.lock() {
            if let Some(dict_entry) = match &*lock {
                CskkDictionaryType::StaticFile(dict) => dict.lookup(raw_to_composite, false),
                CskkDictionaryType::UserFile(dict) => dict.lookup(raw_to_composite, false),
                CskkDictionaryType::EmptyDict(dict) => dict.lookup(raw_to_composite, false),
            } {
                ordered_candidates.extend(dict_entry.get_candidates().to_owned());
                deduped_candidates.extend(dict_entry.get_candidates().to_owned());
            }
        } else {
            warn!("Dictionary read lock failed during getting candidates. Ignoring the dictionary.")
        }
    }

    if deduped_candidates.is_empty() {
        return vec![];
    }
    deduped_candidates.sort_by(|a, b| a.kouho_text.cmp(&b.kouho_text));
    deduped_candidates.dedup_by(|a, b| a.kouho_text == b.kouho_text);

    let mut result = vec![];
    for candidate in ordered_candidates {
        let mut matched_index = usize::MAX;
        for (pos, deduped) in deduped_candidates.iter().enumerate() {
            if (*deduped).eq(&candidate) {
                result.push((*deduped).clone());
                matched_index = pos;
            }
        }
        if matched_index < usize::MAX {
            deduped_candidates.remove(matched_index);
        }
    }

    result
}

///
/// Returns the nth candidate.
/// first selection_pointer == 0
///
#[allow(dead_code)]
pub(crate) fn get_nth_candidate(
    dictionaries: &[Arc<CskkDictionary>],
    to_composite: &str,
    selection_pointer: usize,
) -> Option<Candidate> {
    let candidates = get_all_candidates(dictionaries, to_composite);
    candidates.get(selection_pointer).cloned()
}

pub trait Dictionary {
    /// 今のところ数値変換等がないので、raw_to_compositeではなくmidashiとして完全一致を探す。
    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry>;

    fn is_read_only(&self) -> bool {
        true
    }
    //
    // TODO: midashi から始まるエントリを全て返す。i.e. skkserv 4 command
    // e.g.
    // complete('あ') -> ["あい", "あいさつ"]
    //
    // fn complete(&self, _midashi: &str) /* -> Option<&Vec<&str>>?*/
    /// Returns true if saved, false if kindly ignored.
    /// Safe to call to read_only dictionary.
    fn save_dictionary(&mut self) -> Result<bool, CskkError> {
        Ok(false)
    }

    /// Select that candidate.
    /// Supporting dictionary will add and move that candidate to the first place so that next time it comes to candidate early.
    /// Safe to call to read_only dictionary.
    fn select_candidate(&mut self, _candidate: &Candidate) -> Result<bool, CskkError> {
        Ok(false)
    }
    /// Remove that candidate if dictionary supports editing.
    /// Safe to call to read_only dictionary
    fn purge_candidate(&mut self, _candidate: &Candidate) -> Result<bool, CskkError> {
        Ok(false)
    }
}
