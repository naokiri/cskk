pub(crate) mod candidate;
pub(crate) mod composite_key;
pub(crate) mod dictentry;
pub mod empty_dict;
pub(crate) mod file_dictionary;
pub mod static_dict;
pub mod user_dictionary;

use crate::error::CskkError;
use crate::form_changer::numeric_form_changer::{
    numeric_to_daiji_as_number, numeric_to_kanji_each, numeric_to_simple_kanji_as_number,
    numeric_to_thousand_separator, numeric_to_zenkaku,
};
pub(crate) use candidate::Candidate;
pub(crate) use composite_key::CompositeKey;
use dictentry::DictEntry;
use empty_dict::EmptyDictionary;
use log::*;
use regex::Regex;
use static_dict::StaticFileDict;
use std::sync::{Arc, Mutex};
use user_dictionary::UserDictionary;

// C側に出す関係でSizedである必要があり、dyn Traitではなくenumでラップする。
#[derive(Debug)]
pub(crate) enum CskkDictionaryType {
    StaticFile(StaticFileDict),
    UserFile(UserDictionary),
    EmptyDict(EmptyDictionary),
}

// FIXME: Not sure if this is the correct inner type. Maybe we can remove Arc on other places?
#[derive(Debug)]
pub struct CskkDictionary {
    pub(crate) mutex: Mutex<CskkDictionaryType>,
}

impl CskkDictionary {
    fn new(dictionary: CskkDictionaryType) -> Self {
        Self {
            mutex: Mutex::new(dictionary),
        }
    }

    /// Library user interface for creating new static read-only dictionary.
    /// file_path: path string
    /// encode: label of encoding that encoding_rs can recognize. "utf-8", "euc-jp", "cp866" etc.
    pub fn new_static_dict(file_path: &str, encode: &str) -> Result<CskkDictionary, CskkError> {
        let dictionary = StaticFileDict::new(file_path, encode)?;
        Ok(CskkDictionary::new(CskkDictionaryType::StaticFile(
            dictionary,
        )))
    }

    /// Library user interface for creating new user readable and writable dictionary
    /// file_path: path string
    /// encode: label of encoding that encoding_rs can recognize. "utf-8", "euc-jp", "cp866" etc.
    pub fn new_user_dict(file_path: &str, encode: &str) -> Result<CskkDictionary, CskkError> {
        let dictionary = UserDictionary::new(file_path, encode)?;
        Ok(CskkDictionary::new(CskkDictionaryType::UserFile(
            dictionary,
        )))
    }

    /// Library user interface for creating fallback dictionary.
    /// Dictionary is required to create the context, so this dictionary is useful when no dictionary file is available.
    pub fn new_empty_dict() -> Result<CskkDictionary, CskkError> {
        Ok(CskkDictionary::new(CskkDictionaryType::EmptyDict(
            EmptyDictionary::default(),
        )))
    }
}

/// confirm the candidate.
/// This updates writable dictionaries candidate order or add new entry which confirmed.
/// Returns true if updated the dictionary.
pub(crate) fn confirm_candidate(
    dictionary: &mut Arc<CskkDictionary>,
    candidate: &Candidate,
) -> Result<bool, CskkError> {
    debug!("confirm: {:?}", candidate);
    // Using mutex in match on purpose, never acquiring lock again.
    #[allow(clippy::significant_drop_in_scrutinee)]
    match *dictionary.mutex.lock().unwrap() {
        CskkDictionaryType::StaticFile(ref mut dict) => dict.select_candidate(candidate),
        CskkDictionaryType::UserFile(ref mut dict) => dict.select_candidate(candidate),
        CskkDictionaryType::EmptyDict(ref mut dict) => dict.select_candidate(candidate),
    }
}

/// purge the candidate.
/// This updates writable dictionaries candidate order or add new entry which confirmed.
/// Returns true if updated the dictionary.
pub(crate) fn purge_candidate(
    dictionary: &mut Arc<CskkDictionary>,
    candidate: &Candidate,
) -> Result<bool, CskkError> {
    // Using mutex in match on purpose, never acquiring lock again.
    #[allow(clippy::significant_drop_in_scrutinee)]
    match *dictionary.mutex.lock().unwrap() {
        CskkDictionaryType::StaticFile(ref mut dict) => dict.purge_candidate(candidate),
        CskkDictionaryType::UserFile(ref mut dict) => dict.purge_candidate(candidate),
        CskkDictionaryType::EmptyDict(ref mut dict) => dict.purge_candidate(candidate),
    }
}

/// 現在ueno/libskk同様にDedupはkouho_textのみ、候補の順序はdictの順番通り。
/// annotationについては特に決めていないが、現在のところsortの仕様により先の候補が優先される。
pub(crate) fn get_all_candidates(
    dictionaries: &[Arc<CskkDictionary>],
    composite_key: &CompositeKey,
) -> Vec<Candidate> {
    get_all_candidates_inner(dictionaries, composite_key, false)
}

lazy_static! {
    static ref NUM_REGEX: Regex = Regex::new(r"\d+").unwrap();
}

///
/// Usually, replace numerics to # and search the dict for numeric composition.
/// If numeric-re-lookup, don't replace numerics for the "#4" type entries.
///
fn get_all_candidates_inner(
    dictionaries: &[Arc<CskkDictionary>],
    composite_key: &CompositeKey,
    is_numeric_re_lookup: bool,
) -> Vec<Candidate> {
    let mut deduped_candidates = vec![];
    let mut ordered_candidates = vec![];

    let mut composite_key = composite_key.to_owned();
    let mut matched_numbers = vec![];

    if !is_numeric_re_lookup {
        // FIXME: destructuring-bind is unstable yet in current Rust. Fix in future Rust.
        let pair = to_composite_to_numeric_dict_key(&composite_key);
        composite_key = pair.0;
        matched_numbers = pair.1;
    }

    for cskkdict in dictionaries.iter() {
        let lock = cskkdict.mutex.lock().unwrap();
        if let Some(dict_entry) = match &*lock {
            CskkDictionaryType::StaticFile(dict) => dict.lookup(&composite_key),
            CskkDictionaryType::UserFile(dict) => dict.lookup(&composite_key),
            CskkDictionaryType::EmptyDict(dict) => dict.lookup(&composite_key),
        } {
            ordered_candidates.extend(dict_entry.get_candidates().to_owned());
            deduped_candidates.extend(dict_entry.get_candidates().to_owned());
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
                if is_numeric_re_lookup {
                    result.push((*deduped).clone());
                } else {
                    result.append(&mut replace_numeric_match(
                        deduped,
                        &matched_numbers,
                        dictionaries,
                    ));
                }
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
/// 数字が含まれていた場合#に置きかえて数字と共にかえす。
/// 12がつ6にち -> (#がつ#にち, [12,6])
///
pub(crate) fn to_composite_to_numeric_dict_key(
    to_composite: &CompositeKey,
) -> (CompositeKey, Vec<String>) {
    let mut dict_key = to_composite.get_to_composite().to_owned();
    let mut matched_numbers = vec![];
    for numeric_match in NUM_REGEX.find_iter(to_composite.get_to_composite()) {
        let new_dict_key = dict_key.replacen(numeric_match.as_str(), "#", 1);
        dict_key = new_dict_key;
        matched_numbers.push(numeric_match.as_str().to_owned());
    }
    (
        CompositeKey::new(&dict_key, *to_composite.get_okuri()),
        matched_numbers,
    )
}

/// Return how many numeric string is in string to composite
///
/// compile_fail example for private fn
/// ```compile_fail
/// use cskk::dictionary::numeric_string_count;
/// assert_eq!(numeric_string_count("2かい"), 1);
/// assert_eq!(numeric_string_count("2がつ13にち"), 2);
/// ```
pub(crate) fn numeric_string_count(to_composite: &str) -> usize {
    NUM_REGEX.find_iter(to_composite).count()
}

/// Return how many numeric special string is in kouho string
///
/// compile_fail example for private fn
/// ```compile_fail
/// assert_eq!(numeric_entry_count("#1回"), 1);
/// assert_eq!(numeric_entry_count("#3日"), 1);
/// ```
///
pub(crate) fn numeric_entry_count(entry: &str) -> usize {
    lazy_static! {
        static ref NUM_ENTRY_REGEX: Regex = Regex::new(r"#[0123458]").unwrap();
    }
    NUM_ENTRY_REGEX.find_iter(entry).count()
}

fn replace_numeric_match(
    candidate: &Candidate,
    matched_numbers: &[String],
    dictionaries: &[Arc<CskkDictionary>],
) -> Vec<Candidate> {
    let output_text_list =
        replace_numeric_string(&candidate.kouho_text, matched_numbers, dictionaries);

    let mut result = vec![];
    for output_text in output_text_list {
        let mut new_candidate = candidate.clone();
        new_candidate.output = output_text;
        result.push(new_candidate)
    }
    result
}

/// given kouho_text that includes #[0123458], return the replaced text to be used for outputs.
pub(crate) fn replace_numeric_string(
    kouho_text: &str,
    numbers: &[String],
    dictionaries: &[Arc<CskkDictionary>],
) -> Vec<String> {
    lazy_static! {
        static ref NUMERIC_ENTRY_REGEX: Regex = Regex::new(r"#[0123458]").unwrap();
    }
    let mut current_output_texts = vec![kouho_text.to_string()];
    for (n, entry_match) in NUMERIC_ENTRY_REGEX.find_iter(kouho_text).enumerate() {
        match entry_match.as_str() {
            "#0" => {
                let mut replaced_output_texts = vec![];
                for output_text in &current_output_texts {
                    replaced_output_texts.push(output_text.replacen("#0", &numbers[n], 1));
                }
                current_output_texts = replaced_output_texts;
            }
            "#1" => {
                let mut replaced_output_texts = vec![];
                for kouho_text in &current_output_texts {
                    replaced_output_texts.push(kouho_text.replacen(
                        "#1",
                        &numeric_to_zenkaku(&numbers[n]),
                        1,
                    ));
                }
                current_output_texts = replaced_output_texts;
            }
            "#2" => {
                let mut replaced_output_texts = vec![];
                for kouho_text in &current_output_texts {
                    replaced_output_texts.push(kouho_text.replacen(
                        "#2",
                        &numeric_to_kanji_each(&numbers[n]),
                        1,
                    ));
                }
                current_output_texts = replaced_output_texts;
            }
            "#3" => {
                let mut replaced_output_texts = vec![];
                for output_text in &current_output_texts {
                    replaced_output_texts.push(output_text.replacen(
                        "#3",
                        &numeric_to_simple_kanji_as_number(&numbers[n]),
                        1,
                    ));
                }
                current_output_texts = replaced_output_texts;
            }
            "#4" => {
                let mut replaced_output_texts = vec![];
                let numeric_lookup_results = get_all_candidates_inner(
                    dictionaries,
                    &CompositeKey::new(&numbers[n], None),
                    true,
                );
                for kouho_text in &current_output_texts {
                    for numeric_lookup in &numeric_lookup_results {
                        replaced_output_texts.push(kouho_text.replacen(
                            "#4",
                            &numeric_lookup.kouho_text,
                            1,
                        ));
                    }
                }
                current_output_texts = replaced_output_texts;
            }
            "#5" => {
                let mut replaced_output_texts = vec![];
                for kouho_text in &current_output_texts {
                    replaced_output_texts.push(kouho_text.replacen(
                        "#5",
                        &numeric_to_daiji_as_number(&numbers[n], false),
                        1,
                    ));
                    replaced_output_texts.push(kouho_text.replacen(
                        "#5",
                        &numeric_to_daiji_as_number(&numbers[n], true),
                        1,
                    ));
                }
                current_output_texts = replaced_output_texts;
            }
            "#8" => {
                let mut replaced_output_texts = vec![];
                for kouho_text in &current_output_texts {
                    replaced_output_texts.push(kouho_text.replacen(
                        "#8",
                        &numeric_to_thousand_separator(&numbers[n]),
                        1,
                    ));
                }
                current_output_texts = replaced_output_texts;
            }
            _ => {}
        }
    }
    current_output_texts
}

///
/// Returns the nth candidate.
/// first selection_pointer == 0
///
#[allow(dead_code)]
pub(crate) fn get_nth_candidate(
    dictionaries: &[Arc<CskkDictionary>],
    composite_key: &CompositeKey,
    selection_pointer: usize,
) -> Option<Candidate> {
    let candidates = get_all_candidates(dictionaries, composite_key);
    candidates.get(selection_pointer).cloned()
}

pub(crate) trait Dictionary {
    /// midashiと一致するエントリを返す。
    fn lookup(&self, composite_key: &CompositeKey) -> Option<&DictEntry>;

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

    /// Reload dictionary.
    fn reload(&mut self) -> Result<(), CskkError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_numeric_string_count() {
        assert_eq!(numeric_string_count("123つぶ"), 1);
        assert_eq!(numeric_string_count("1にち1かい"), 2);
        assert_eq!(numeric_string_count("1じつせんしゅう"), 1);
    }
}
