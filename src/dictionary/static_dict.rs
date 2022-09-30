use crate::CskkError;
use std::collections::BTreeMap;

use crate::dictionary::file_dictionary::{load_dictionary, DictionaryEntriesPair, FileDictionary};
use crate::dictionary::{CompositeKey, DictEntry, Dictionary};

#[derive(Debug)]
pub(crate) struct StaticFileDict {
    file_path: String,
    encode: String,
    // Midashi -> DictEntry map
    okuri_ari_dictionary: BTreeMap<String, DictEntry>,
    okuri_nashi_dictionary: BTreeMap<String, DictEntry>,
}

impl StaticFileDict {
    /// file_path: string
    /// encode: label of encoding that encoding_rs can recognize. "utf-8", "euc-jp", "cp866" etc.
    pub(crate) fn new(file_path: &str, encode: &str) -> Result<Self, CskkError> {
        let dictionary = load_dictionary(file_path, encode.as_bytes())?;
        Ok(StaticFileDict {
            file_path: String::from(file_path),
            encode: encode.to_string(),
            okuri_ari_dictionary: dictionary.okuri_ari,
            okuri_nashi_dictionary: dictionary.okuri_nashi,
        })
    }
}

impl Dictionary for StaticFileDict {
    fn lookup(&self, composite_key: &CompositeKey) -> Option<&DictEntry> {
        FileDictionary::lookup(self, composite_key)
    }

    fn reload(&mut self) -> Result<(), CskkError> {
        FileDictionary::reload(self)
    }
}

impl FileDictionary for StaticFileDict {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn encode(&self) -> &str {
        &self.encode
    }

    fn set_dictionary(&mut self, dictionary: DictionaryEntriesPair) {
        self.okuri_ari_dictionary = dictionary.okuri_ari;
        self.okuri_nashi_dictionary = dictionary.okuri_nashi;
    }

    fn get_okuri_ari_dictionary(&self) -> &BTreeMap<String, DictEntry> {
        &self.okuri_ari_dictionary
    }

    fn get_okuri_nashi_dictionary(&self) -> &BTreeMap<String, DictEntry> {
        &self.okuri_nashi_dictionary
    }
}
