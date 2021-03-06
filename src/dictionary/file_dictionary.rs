use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::Dictionary;
use crate::error::CskkError;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::warn;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub trait FileDictionary: Dictionary {
    fn file_path(&self) -> &str;

    fn encode(&self) -> &str;

    fn set_dictionary(&mut self, dictionary: BTreeMap<String, DictEntry>);

    fn reload(&mut self) -> Result<(), CskkError> {
        let dictionary = load_dictionary(&self.file_path(), self.encode().as_bytes())?;
        self.set_dictionary(dictionary);
        Ok(())
    }
}

pub(crate) fn load_dictionary(
    file_path: &str,
    encode: &[u8],
) -> Result<BTreeMap<String, DictEntry>, CskkError> {
    let dict_file = File::open(file_path)?;
    let enc = Encoding::for_label(encode);
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(enc)
        .build(dict_file);
    let reader = BufReader::new(decoder);
    let mut dictionary = BTreeMap::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(';') {
                    // Skip
                } else {
                    let parsed = DictEntry::from_skkjisyo_line(&line);
                    match parsed {
                        Ok(parsed) => {
                            dictionary.insert(parsed.midashi.clone(), parsed);
                        }
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
    Ok(dictionary)
}
