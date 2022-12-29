use crate::dictionary::file_dictionary::{load_dictionary, DictionaryEntries, FileDictionary};
use crate::dictionary::lru_ordered_map::LruOrderedMap;
use crate::dictionary::{CompositeKey, DictEntry, Dictionary};
use crate::CskkError;

#[derive(Debug)]
pub(crate) struct StaticFileDict {
    file_path: String,
    encode: String,
    // Midashi -> DictEntry map
    okuri_ari_dictionary: LruOrderedMap<String, DictEntry>,
    okuri_nashi_dictionary: LruOrderedMap<String, DictEntry>,
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
    /// 合致するDictEntryがあれば返す。lookupのみで、選択による副作用なし。
    fn lookup(&self, composite_key: &CompositeKey) -> Option<&DictEntry> {
        return if composite_key.has_okuri() {
            self.okuri_ari_dictionary
                .peek(&composite_key.get_dict_key())
        } else {
            self.okuri_nashi_dictionary
                .peek(&composite_key.get_dict_key())
        };
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

    fn set_dictionary(&mut self, dictionary: DictionaryEntries) {
        self.okuri_ari_dictionary = dictionary.okuri_ari;
        self.okuri_nashi_dictionary = dictionary.okuri_nashi;
    }
}
