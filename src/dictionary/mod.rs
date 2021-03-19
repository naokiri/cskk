use static_dict::StaticFileDict;

use dictentry::DictEntry;
use user_dictionary::UserDictionary;
use anyhow::Result;

pub(crate) mod file_dictionary;
pub(crate) mod dictentry;
pub(crate) mod candidate;
pub mod static_dict;
pub mod user_dictionary;

// C側に出す関係でSizedである必要があり、dyn Traitではなくenumでラップする。
#[derive(Debug)]
pub enum CskkDictionary {
    StaticFile(StaticFileDict),
    UserFile(UserDictionary),
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
    fn save_dictionary(&self) -> Result<bool> {
        Ok(false)
    }

    // Select that candidate.
    // Supporting dictionary will move that candidate to the first place so that next time it comes to candidate early.
    // Safe to call to read_only dictionary.
    // fn select_candidate(&self, _candidate: &Candidate) {}
    // fn purge_candidate(&self/* ,midashi, &candidate */) -> bool
}




