use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::Dictionary;
use crate::error::CskkError;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::warn;
use sequence_trie::SequenceTrie;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub(crate) trait FileDictionary: Dictionary {
    fn file_path(&self) -> &str;

    fn encode(&self) -> &str;

    fn set_dictionary(&mut self, dictionary: SequenceTrie<char, DictEntry>);

    fn get_dictionary(&self) -> &SequenceTrie<char, DictEntry>;

    fn reload(&mut self) -> Result<(), CskkError> {
        let dictionary = load_dictionary(self.file_path(), self.encode().as_bytes())?;
        self.set_dictionary(dictionary);
        Ok(())
    }

    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry> {
        self.get_dictionary()
            .get(&midashi.chars().collect::<Vec<char>>())
    }

    fn complement(&self, midashi: &str) -> Result<Vec<&DictEntry>, CskkError> {
        return if let Some(entry) = self
            .get_dictionary()
            .get_node(&midashi.chars().collect::<Vec<char>>())
        {
            let mut entries = entry.children_with_keys();
            entries.sort_unstable_by(|(key_a, _node_a), (key_b, _node_b)| key_a.cmp(key_b));
            let result = entries
                .iter()
                .filter(|(_key, node)| node.value().is_some())
                .map(|(_key, node)| node.value().unwrap())
                .collect();
            Ok(result)
        } else {
            Ok(vec![])
        };
    }
}

pub(crate) fn load_dictionary(
    file_path: &str,
    encode: &[u8],
) -> Result<SequenceTrie<char, DictEntry>, CskkError> {
    let dict_file = File::open(file_path)?;
    let enc = Encoding::for_label_no_replacement(encode);
    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(enc)
        .build(dict_file);
    let reader = BufReader::new(decoder);
    let mut dictionary = SequenceTrie::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with(';') {
                    // Skip
                } else {
                    let parsed = DictEntry::from_skkjisyo_line(&line);
                    match parsed {
                        Ok(parsed) => {
                            dictionary
                                .insert(&parsed.midashi.chars().collect::<Vec<char>>(), parsed);
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
