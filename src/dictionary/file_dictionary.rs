use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::Dictionary;
use crate::error::CskkError;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::warn;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub(in crate::dictionary) struct DictionaryEntries {
    pub(in crate::dictionary) okuri_ari: Vec<(String, DictEntry)>,
    pub(in crate::dictionary) okuri_nashi: Vec<(String, DictEntry)>,
}

pub(in crate::dictionary) trait FileDictionary: Dictionary {
    fn file_path(&self) -> &str;

    fn encode(&self) -> &str;

    fn set_dictionary(&mut self, dictionary: DictionaryEntries);

    fn reload(&mut self) -> Result<(), CskkError> {
        let dictionary = load_dictionary(self.file_path(), self.encode().as_bytes())?;
        self.set_dictionary(dictionary);
        Ok(())
    }
}

enum DictionaryLoadMode {
    OkuriAri,
    OkuriNashi,
}

/// 順序付きで辞書を読む
pub(in crate::dictionary) fn load_dictionary(
    file_path: &str,
    encode: &[u8],
) -> Result<DictionaryEntries, CskkError> {
    let dict_file = File::open(file_path)?;
    let enc = Encoding::for_label_no_replacement(encode);
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(enc)
        .build(dict_file);
    let reader = BufReader::new(decoder);
    let mut okuri_ari_dictionary = Vec::new();
    let mut okuri_nashi_dictionary = Vec::new();

    // 後の送り仮名再確認の時にabbrevエントリを読み間違えないため、デフォルトはOkuriAri
    let mut mode = DictionaryLoadMode::OkuriAri;
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(';') {
                    if line.contains(";; okuri-ari entries") {
                        mode = DictionaryLoadMode::OkuriAri;
                    } else if line.contains(";; okuri-nasi entries") {
                        mode = DictionaryLoadMode::OkuriNashi
                    }
                } else {
                    let parsed = DictEntry::from_skkjisyo_line(&line);
                    match parsed {
                        Ok(parsed) => match mode {
                            DictionaryLoadMode::OkuriAri => {
                                // 過去の辞書でokuri-ari,nasiを無視して保存していた互換性のため、行をparseした内容で確認しなおす。
                                if parsed.is_okuri_ari_entry() {
                                    okuri_ari_dictionary.push((parsed.midashi.clone(), parsed));
                                } else {
                                    okuri_nashi_dictionary.push((parsed.midashi.clone(), parsed));
                                }
                            }
                            DictionaryLoadMode::OkuriNashi => {
                                okuri_nashi_dictionary.push((parsed.midashi.clone(), parsed));
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
    Ok(DictionaryEntries {
        okuri_nashi: okuri_nashi_dictionary,
        okuri_ari: okuri_ari_dictionary,
    })
}
