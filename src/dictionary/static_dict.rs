use std::collections::BTreeMap;

use crate::dictionary::{DictEntry, Dictionary, load_dictionary};

#[derive(Debug)]
pub struct StaticFileDict {
    file_path: String,
    encode: String,
    dictionary: BTreeMap<String, DictEntry>,
}

impl StaticFileDict {
    pub fn new(file_path: &str, encode: &str) -> Self {
        let dictionary = load_dictionary(file_path, encode.as_bytes());
        StaticFileDict {
            file_path: String::from(file_path),
            encode: encode.to_string(),
            dictionary,
        }
    }

    pub fn reload(&mut self) {
        let dictionary = load_dictionary(&self.file_path, self.encode.as_bytes());
        self.dictionary = dictionary;
    }
}

impl Dictionary for StaticFileDict {
    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry> {
        self.dictionary.get(midashi)
    }

    fn is_read_only(&self) -> bool {
        true
    }
}