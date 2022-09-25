use crate::dictionary::candidate::Candidate;
use crate::dictionary::file_dictionary::{load_dictionary, FileDictionary};
use crate::dictionary::{DictEntry, Dictionary};
use crate::error::CskkError;
use crate::error::CskkError::Error;
use encoding_rs::{Encoder, EncoderResult, Encoding};
use sequence_trie::SequenceTrie;
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
    dictionary: SequenceTrie<char, DictEntry>,
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
            dictionary,
            has_change: false,
        })
    }
}

impl Dictionary for UserDictionary {
    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry> {
        self.dictionary.get(&midashi.chars().collect::<Vec<char>>())
    }

    fn is_read_only(&self) -> bool {
        false
    }

    /// {file_path}.BAK に退避してからfile_pathに保存する
    /// TODO: 現在は他の辞書と互換性がないただのエントリの羅列なので、okuri-ari entriesとokuri-nasi entriesに分けてddskkのようにファイル上で走査する辞書互換にする。
    fn save_dictionary(&mut self) -> Result<bool, CskkError> {
        if self.has_change {
            rename(&self.file_path, &format!("{}.BAK", self.file_path))?;
            let dict_file = File::create(&self.file_path)?;

            let mut stream = BufWriter::new(dict_file);
            let mut enc = Encoding::for_label(self.encode.as_bytes())
                .expect("It should be same as encoding name succeeded when loading file.")
                .new_encoder();
            for dictentry in self.dictionary.values() {
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

    fn select_candidate(&mut self, candidate: &Candidate) -> Result<bool, CskkError> {
        let midashi = &candidate.midashi;
        log::debug!("Select midashi: {:?}", midashi);
        let midashi_chars = midashi.chars().collect::<Vec<char>>();
        let entry = self.dictionary.get_mut(&midashi_chars);
        match entry {
            Some(dict_entry) => {
                dict_entry.remove_matching_candidate(candidate);
                dict_entry.insert_as_first_candidate(candidate.clone());
            }
            None => {
                self.dictionary.insert(
                    &midashi_chars,
                    DictEntry {
                        midashi: (*candidate.midashi).clone(),
                        candidates: vec![(*candidate).clone()],
                    },
                );
            }
        }
        self.has_change = true;
        Ok(true)
    }

    fn purge_candidate(&mut self, candidate: &Candidate) -> Result<bool, CskkError> {
        let midashi = &candidate.midashi;
        let entry = self
            .dictionary
            .get_mut(&midashi.chars().collect::<Vec<char>>());
        if let Some(dict_entry) = entry {
            dict_entry.remove_matching_candidate(candidate);
        }
        self.has_change = true;
        Ok(true)
    }
}

impl FileDictionary for UserDictionary {
    fn file_path(&self) -> &str {
        &self.file_path
    }

    fn encode(&self) -> &str {
        &self.encode
    }

    fn set_dictionary(&mut self, dictionary: SequenceTrie<char, DictEntry>) {
        self.dictionary = dictionary;
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
        let candidate = Candidate::from_skk_jisyo_string("あああ", "アアア;wow").unwrap();
        user_dictionary.select_candidate(&candidate)?;
        user_dictionary.save_dictionary()?;
        user_dictionary.purge_candidate(&candidate)?;
        user_dictionary.save_dictionary()?;
        Ok(())
    }
}
