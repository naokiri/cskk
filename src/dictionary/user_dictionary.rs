use crate::dictionary::candidate::Candidate;
use crate::dictionary::file_dictionary::{load_dictionary, DictionaryEntriesPair, FileDictionary};
use crate::dictionary::{CompositeKey, DictEntry, Dictionary};
use crate::error::CskkError;
use crate::error::CskkError::Error;
use encoding_rs::{Encoder, EncoderResult, Encoding};
use log::*;
use std::collections::BTreeMap;
use std::fs::{rename, File};
use std::io::{BufWriter, Write};

///
/// User dictionary that can load from file and save entries to file.
///
///
#[derive(Debug)]
pub(crate) struct UserDictionary {
    file_path: String,
    encode: String,
    // Midashi -> DictEntry map
    okuri_ari_dictionary: BTreeMap<String, DictEntry>,
    okuri_nashi_dictionary: BTreeMap<String, DictEntry>,
    // Just bool, because we know this is under mutex.
    has_change: bool,
}

const BUF_SIZE: usize = 1024;

impl UserDictionary {
    pub(crate) fn new(file_path: &str, encode: &str) -> Result<Self, CskkError> {
        let dictionary = load_dictionary(file_path, encode.as_bytes())?;
        Ok(UserDictionary {
            file_path: String::from(file_path),
            encode: encode.to_string(),
            okuri_ari_dictionary: dictionary.okuri_ari,
            okuri_nashi_dictionary: dictionary.okuri_nashi,
            has_change: false,
        })
    }
}

impl Dictionary for UserDictionary {
    fn lookup(&self, composite_key: &CompositeKey) -> Option<&DictEntry> {
        FileDictionary::lookup(self, composite_key)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    /// {file_path}.BAK に退避してからfile_pathに保存する
    /// 辞書ファイルのフォーマットは SKK 16.2 user manual 5.10.7 辞書の書式 に依る
    fn save_dictionary(&mut self) -> Result<bool, CskkError> {
        if self.has_change {
            rename(&self.file_path, &format!("{}.BAK", self.file_path))?;
            let dict_file = File::create(&self.file_path)?;

            let mut stream = BufWriter::new(dict_file);
            let mut enc = Encoding::for_label(self.encode.as_bytes())
                .expect("It should be same as encoding name succeeded when loading file.")
                .new_encoder();

            // Not using. Can't compile on mac.
            // let encoded = encode_string(
            //     &mut enc,
            //     &format!(
            //         ";; Save on {} \n",
            //         chrono::offset::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
            //     ),
            // )?;
            // stream.write_all(encoded.as_slice())?;

            let encoded = encode_string(&mut enc, ";; okuri-ari entries.\n")?;
            stream.write_all(encoded.as_slice())?;
            for dictentry in self.okuri_ari_dictionary.values() {
                let mut source = dictentry.to_skk_jisyo_string();
                source += "\n";
                if let Ok(encoded) = encode_string(&mut enc, source.as_mut_str()) {
                    stream.write_all(encoded.as_slice())?;
                }
            }
            let encoded = encode_string(&mut enc, ";; okuri-nasi entries.\n")?;
            stream.write_all(encoded.as_slice())?;
            for dictentry in self.okuri_nashi_dictionary.values() {
                let mut source = dictentry.to_skk_jisyo_string();
                source += "\n";
                if let Ok(encoded) = encode_string(&mut enc, source.as_mut_str()) {
                    stream.write_all(encoded.as_slice())?;
                }
            }
            stream.flush()?;
            self.has_change = false;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn select_candidate(
        &mut self,
        composite_key: &CompositeKey,
        candidate: &Candidate,
    ) -> Result<bool, CskkError> {
        let midashi = &candidate.midashi;
        debug!("Select midashi: {:?}", midashi);
        let dictionary = if candidate.okuri {
            &mut self.okuri_ari_dictionary
        } else {
            &mut self.okuri_nashi_dictionary
        };

        let entry = dictionary.get_mut(midashi.as_str());
        match entry {
            Some(dict_entry) => {
                dict_entry.prioritize_candidate(composite_key, candidate);
            }
            None => {
                dictionary.insert(
                    candidate.midashi.to_owned(),
                    DictEntry::new(&candidate.midashi, composite_key, candidate),
                );
            }
        }
        self.has_change = true;
        Ok(true)
    }

    fn purge_candidate(
        &mut self,
        composite_key: &CompositeKey,
        candidate: &Candidate,
    ) -> Result<bool, CskkError> {
        let dictionary = if candidate.okuri {
            &mut self.okuri_ari_dictionary
        } else {
            &mut self.okuri_nashi_dictionary
        };
        let midashi = &candidate.midashi;
        let entry = dictionary.get_mut(midashi.as_str());
        if let Some(dict_entry) = entry {
            dict_entry.remove_matching_candidate(composite_key, candidate);
        }
        self.has_change = true;
        Ok(true)
    }

    fn reload(&mut self) -> Result<(), CskkError> {
        FileDictionary::reload(self)
    }
}

impl FileDictionary for UserDictionary {
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

fn encode_string(encoder: &mut Encoder, to_encode: &str) -> Result<Vec<u8>, CskkError> {
    let mut encoded_vec = Vec::with_capacity(BUF_SIZE);
    let mut source = to_encode;
    let mut tmp_buf = Vec::with_capacity(BUF_SIZE);
    loop {
        let (result, read) =
            encoder.encode_from_utf8_to_vec_without_replacement(source, &mut tmp_buf, true);
        if read == 0 {
            return Err(Error(
                "Cannot read on encoding. Give up whole string.".to_string(),
            ));
        }
        match result {
            EncoderResult::Unmappable(_char) => {
                return Err(Error("Encoding failed. Give up whole string.".to_string()));
            }
            EncoderResult::InputEmpty => {
                encoded_vec.append(&mut tmp_buf);
                break;
            }
            EncoderResult::OutputFull => {
                encoded_vec.append(&mut tmp_buf);
                source = &source[read..];
            }
        }
    }
    Ok(encoded_vec)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn userdict() -> Result<(), CskkError> {
        File::create("tests/data/dictionaries/empty.dat")?;
        let mut user_dictionary =
            UserDictionary::new("tests/data/dictionaries/empty.dat", "utf-8")?;
        let candidate = Candidate::new(
            "あああ".to_string(),
            false,
            "アアア".to_string(),
            Some("wow".to_string()),
            "アアア".to_string(),
        );
        let composite_key = CompositeKey::new("あああ", None);
        user_dictionary.select_candidate(&composite_key, &candidate)?;
        user_dictionary.save_dictionary()?;
        user_dictionary.purge_candidate(&composite_key, &candidate)?;
        user_dictionary.save_dictionary()?;
        Ok(())
    }
}
