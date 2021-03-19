use std::collections::BTreeMap;

use crate::dictionary::{DictEntry, Dictionary};
use crate::dictionary::file_dictionary::{FileDictionary, load_dictionary};

#[derive(Debug)]
pub struct StaticFileDict {
    file_path: String,
    encode: String,
    // Midashi -> DictEntry map
    dictionary: BTreeMap<String, DictEntry>,
}

impl StaticFileDict {
    pub fn new(file_path: &str, encode: &str) -> Self {
        if let Ok(dictionary) = load_dictionary(file_path, encode.as_bytes()) {
            StaticFileDict {
                file_path: String::from(file_path),
                encode: encode.to_string(),
                dictionary,
            }
        } else {
            panic!("Failed to load dictionary {}", file_path)
        }
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