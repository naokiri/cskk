use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::{CompositeKey, Dictionary};
use crate::error::CskkError;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::warn;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub(in crate::dictionary) struct DictionaryEntriesPair {
    pub(in crate::dictionary) okuri_ari: BTreeMap<String, DictEntry>,
    pub(in crate::dictionary) okuri_nashi: BTreeMap<String, DictEntry>,
}

pub(in crate::dictionary) trait FileDictionary: Dictionary {
    fn file_path(&self) -> &str;

    fn encode(&self) -> &str;

    fn set_dictionary(&mut self, dictionary: DictionaryEntriesPair);

    fn get_okuri_ari_dictionary(&self) -> &BTreeMap<String, DictEntry>;

    fn get_okuri_nashi_dictionary(&self) -> &BTreeMap<String, DictEntry>;

    fn reload(&mut self) -> Result<(), CskkError> {
        let dictionary = load_dictionary(self.file_path(), self.encode().as_bytes())?;
        self.set_dictionary(dictionary);
        Ok(())
    }

    fn lookup(&self, composite_key: &CompositeKey) -> Option<&DictEntry> {
        return if composite_key.has_okuri() {
            self.get_okuri_ari_dictionary()
                .get(&composite_key.get_dict_key())
        } else {
            self.get_okuri_nashi_dictionary()
                .get(&composite_key.get_dict_key())
        };
    }
}

enum DictionaryLoadMode {
    OkuriAri,
    OkuriNashi,
}

pub(in crate::dictionary) fn load_dictionary(
    file_path: &str,
    encode: &[u8],
) -> Result<DictionaryEntriesPair, CskkError> {
    let dict_file = File::open(file_path)?;
    let enc = Encoding::for_label_no_replacement(encode);
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(enc)
        .build(dict_file);
    let reader = BufReader::new(decoder);
    let mut okuri_ari_dictionary = BTreeMap::new();
    let mut okuri_nashi_dictionary = BTreeMap::new();

    // 後の送り仮名再確認の時にabbrevエントリを読み間違えないため、デフォルトはOkuriAri
    let mut mode = DictionaryLoadMode::OkuriAri;
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(';') {
                    if line.contains(";; okuri-ari entries") {
                        mode = DictionaryLoadMode::OkuriAri;
                    } else if line.eq(";; okuri-nasi entries") {
                        mode = DictionaryLoadMode::OkuriNashi
                    }
                } else {
                    let parsed = DictEntry::from_skkjisyo_line(&line);
                    match parsed {
                        Ok(parsed) => match mode {
                            DictionaryLoadMode::OkuriAri => {
                                // 過去の辞書でokuri-ari,nasiを無視して保存していた互換性のため、行をparseした内容で確認しなおす。
                                if parsed.is_okuri_ari_entry() {
                                    okuri_ari_dictionary.insert(parsed.midashi.clone(), parsed);
                                } else {
                                    okuri_nashi_dictionary.insert(parsed.midashi.clone(), parsed);
                                }
                            }
                            DictionaryLoadMode::OkuriNashi => {
                                okuri_nashi_dictionary.insert(parsed.midashi.clone(), parsed);
                            }
                        },
                        Err(_) => {
                            warn!("Dict is ill formatted. Ignored line {}", &line);
                        }
                    }
                }
            }
            Err(_) => {
                warn!("Dict is ill encoded. Ignored one line.");
            }
        }
    }
    Ok(DictionaryEntriesPair {
        okuri_nashi: okuri_nashi_dictionary,
        okuri_ari: okuri_ari_dictionary,
    })
}
