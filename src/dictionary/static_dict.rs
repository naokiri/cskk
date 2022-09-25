use crate::dictionary::file_dictionary::{load_dictionary, FileDictionary};
use crate::dictionary::{DictEntry, Dictionary};
use crate::CskkError;
use sequence_trie::SequenceTrie;

#[derive(Debug)]
pub(crate) struct StaticFileDict {
    file_path: String,
    encode: String,
    // Midashi -> DictEntry map
    dictionary: SequenceTrie<char, DictEntry>,
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
    fn lookup(&self, midashi: &str, okuri: bool) -> Option<&DictEntry> {
        FileDictionary::lookup(self, midashi, okuri)
    }

    fn complement(&self, midashi: &str) -> Result<Vec<&DictEntry>, CskkError> {
        FileDictionary::complement(self, midashi)
    }
}

impl FileDictionary for StaticFileDict {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn encode(&self) -> &str {
        &self.encode
    }

    fn set_dictionary(&mut self, dictionary: SequenceTrie<char, DictEntry>) {
        self.dictionary = dictionary
    }

    fn get_dictionary(&self) -> &SequenceTrie<char, DictEntry> {
        &self.dictionary
    }
}
