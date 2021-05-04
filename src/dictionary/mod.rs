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

// C側に出す関係でSizedである必要があり、dyn Traitではなくenumでラップする。
#[derive(Debug)]
pub enum CskkDictionaryType {
    StaticFile(StaticFileDict),
    UserFile(UserDictionary),
    EmptyDict(EmptyDictionary),
}

pub type CskkDictionary = Mutex<CskkDictionaryType>;

/// confirm the candidate.
/// This updates writable dictionaries candidate order or add new entry which confirmed.
/// Returns true if updated the dictionary.
pub fn confirm_candidate(
    dictionary: &mut Arc<CskkDictionary>,
    midashi: &str,
    okuri: bool,
    kouho_text: &str,
) -> Result<bool, CskkError> {
    let candidate = Candidate::new(
        Arc::new(midashi.to_string()),
        okuri,
        Arc::new(kouho_text.to_string()),
        None,
        None,
    );

    if let Ok(mut mut_dictionary) = dictionary.lock() {
        return match *mut_dictionary {
            CskkDictionaryType::StaticFile(ref mut dict) => dict.select_candidate(&candidate),
            CskkDictionaryType::UserFile(ref mut dict) => dict.select_candidate(&candidate),
            CskkDictionaryType::EmptyDict(ref mut dict) => dict.select_candidate(&candidate),
        };
    }
    Ok(false)
}
//}

/// purge the candidate.
/// This updates writable dictionaries candidate order or add new entry which confirmed.
/// Returns true if updated the dictionary.
pub fn purge_candidate(
    dictionary: &mut Arc<CskkDictionary>,
    midashi: &str,
    okuri: bool,
    kouho_text: &str,
) -> Result<bool, CskkError> {
    let candidate = Candidate::new(
        Arc::new(midashi.to_string()),
        okuri,
        Arc::new(kouho_text.to_string()),
        None,
        None,
    );

    if let Ok(mut mut_dictionary) = dictionary.lock() {
        return match *mut_dictionary {
            CskkDictionaryType::StaticFile(ref mut dict) => dict.purge_candidate(&candidate),
            CskkDictionaryType::UserFile(ref mut dict) => dict.purge_candidate(&candidate),
            CskkDictionaryType::EmptyDict(ref mut dict) => dict.purge_candidate(&candidate),
        };
    }

    Ok(false)
}

pub trait Dictionary {
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
    fn save_dictionary(&self) -> Result<bool, CskkError> {
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
