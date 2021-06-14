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

use crate::form_changer::numeric_form_changer::{numeric_to_daiji_as_number, numeric_to_kanji_each, numeric_to_simple_kanji_as_number, numeric_to_zenkaku, numeric_to_thousand_separator};
use log::*;
use regex::Regex;

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
    debug!("confirm: {:?}", candidate);
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
    get_all_candidates_inner(dictionaries, raw_to_composite, false)
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
    raw_to_composite: &str,
    is_numeric_re_lookup: bool,
) -> Vec<Candidate> {
    let mut deduped_candidates = vec![];
    let mut ordered_candidates = vec![];

    let mut dict_key = raw_to_composite.to_string();
    let mut matched_numbers = vec![];

    if !is_numeric_re_lookup {
        // FIXME: destructuring-bind is unstable yet in current Rust. Fix in future Rust.
        let pair = to_composite_to_numeric_dict_key(raw_to_composite);
        dict_key = pair.0;
        matched_numbers = pair.1;
    }

    for cskkdict in dictionaries.iter() {
        if let Ok(lock) = cskkdict.lock() {
            if let Some(dict_entry) = match &*lock {
                CskkDictionaryType::StaticFile(dict) => dict.lookup(&dict_key, false),
                CskkDictionaryType::UserFile(dict) => dict.lookup(&dict_key, false),
                CskkDictionaryType::EmptyDict(dict) => dict.lookup(&dict_key, false),
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
pub(crate) fn to_composite_to_numeric_dict_key(to_composite: &str) -> (String, Vec<String>) {
    let mut dict_key = to_composite.to_string();
    let mut matched_numbers = vec![];
    for numeric_match in NUM_REGEX.find_iter(to_composite) {
        let new_dict_key = dict_key.replacen(numeric_match.as_str(), "#", 1);
        dict_key = new_dict_key;
        matched_numbers.push(numeric_match.as_str().to_owned());
    }
    (dict_key, matched_numbers)
}

/// Return how many numeric string is in string to composite
pub(crate) fn numeric_string_count(to_composite: &str) -> usize {
    NUM_REGEX.find_iter(to_composite).count()
}

/// Return how many numeric special string is in kouho string
pub(crate) fn numeric_entry_count(entry: &str) -> usize {
    lazy_static! {
        static ref NUM_ENTRY_REGEX: Regex = Regex::new(r"#[012345]").unwrap();
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
                let numeric_lookup_results =
                    get_all_candidates_inner(dictionaries, &numbers[n], true);
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
