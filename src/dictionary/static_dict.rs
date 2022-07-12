use crate::CskkError;
use std::collections::BTreeMap;

use crate::dictionary::file_dictionary::{load_dictionary, FileDictionary};
use crate::dictionary::{DictEntry, Dictionary};

#[derive(Debug)]
pub(crate) struct StaticFileDict {
    file_path: String,
    encode: String,
    // Midashi -> DictEntry map
    dictionary: BTreeMap<String, DictEntry>,
}

impl StaticFileDict {
    /// file_path: string
    /// encode: label of encoding that encoding_rs can recognize. "utf-8", "euc-jp", "cp866" etc.
    pub(crate) fn new(file_path: &str, encode: &str) -> Result<Self, CskkError> {
        let dictionary = load_dictionary(file_path, encode.as_bytes())?;
        Ok(StaticFileDict {
            file_path: String::from(file_path),
            encode: encode.to_string(),
            dictionary,
        })
    }
}

impl Dictionary for StaticFileDict {
    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry> {
        self.dictionary.get(midashi)
    }
}

impl FileDictionary for StaticFileDict {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn encode(&self) -> &str {
        &self.encode
    }

    fn set_dictionary(&mut self, dictionary: BTreeMap<String, DictEntry>) {
        self.dictionary = dictionary
    }
}
