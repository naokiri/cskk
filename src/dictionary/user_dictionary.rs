use std::collections::BTreeMap;
use anyhow::Result;

use crate::dictionary::{DictEntry, Dictionary};
use crate::dictionary::file_dictionary::{FileDictionary, load_dictionary};
use std::fs::{rename, File};
use encoding_rs::{Encoding, EncoderResult, Encoder};
use std::io::{BufWriter, Write};
use crate::error::CskkError::Error;

///
/// User dictionary that can load from file and save entries to file.
///
///
#[derive(Debug)]
pub struct UserDictionary {
    file_path: String,
    encode: String,
    // Midashi -> DictEntry map
    dictionary: BTreeMap<String, DictEntry>,
}

const BUF_SIZE: usize = 1024;

impl UserDictionary {
    pub fn new(file_path: &str, encode: &str) -> Self {
        if let Ok(dictionary) = load_dictionary(file_path, encode.as_bytes()) {
            UserDictionary {
                file_path: String::from(file_path),
                encode: encode.to_string(),
                dictionary,
            }
        } else {
            panic!("Failed to load dictionary {}", file_path)
        }
    }
}

impl Dictionary for UserDictionary {
    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry> {
        self.dictionary.get(midashi)
    }

    fn is_read_only(&self) -> bool {
        false
    }

    /// {file_path}.BAK に退避してからfile_pathに保存する
    fn save_dictionary(&self) -> Result<bool> {
        rename(&self.file_path, &format!("{}.BAK", self.file_path))?;
        let dict_file = File::create(&self.file_path)?;

        let mut stream = BufWriter::new(dict_file);
        let mut enc = Encoding::for_label(&self.encode.as_bytes()).expect("It should be same as encoding name succeeded when loading file.").new_encoder();
        for (_, dictentry) in &self.dictionary {
            let mut source = dictentry.to_skk_jisyo_string();
            if let Ok(encoded) = encode_string(&mut enc, source.as_mut_str()) {
                stream.write_all(encoded.as_slice())?;
            }
        }
        stream.flush()?;
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

    fn set_dictionary(&mut self, dictionary: BTreeMap<String, DictEntry>) {
        self.dictionary = dictionary;
    }
}

fn encode_string(encoder: &mut Encoder, to_encode: &str) -> Result<Vec<u8>> {
    let mut encoded_vec = Vec::with_capacity(BUF_SIZE);
    let mut source = to_encode;
    let mut tmp_buf = Vec::with_capacity(BUF_SIZE);
    loop {
        let (result, read) = encoder.encode_from_utf8_to_vec_without_replacement(&source, &mut tmp_buf, true);
        if read == 0 {
            Err(Error("Cannot read on encoding. Give up whole string.".to_string()))?
        }
        match result {
            EncoderResult::Unmappable(_char) => {
                Err(Error("Encoding failed. Give up whole string.".to_string()))?
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