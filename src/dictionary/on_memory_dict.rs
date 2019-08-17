extern crate encoding_rs;
extern crate encoding_rs_io;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
// FIXME: log related use doesn't work well with current Rust2018 + clippy etc.
#[allow(unused_imports)]
use log::log;
use log::warn;

use crate::dictionary::candidate::Candidate;
use std::sync::Arc;

struct DictEntry {
    midashi: String,
    candidates: Vec<Arc<Candidate>>,
}

struct OnMemoryDict {
    okuri_nashi_dictionary: BTreeMap<String, DictEntry>,
}

#[derive(PartialEq)]
enum EntryProcessingMode {
    OkuriAri,
    OkuriNasi,
}

///
/// skkjisyo dictionary
///
impl OnMemoryDict {
    fn split_candidates(line: &str) -> Result<DictEntry, &str> {
        let mut result = Vec::new();
        let mut line = line.trim().split(' ');
        let midashi = if let Some(midashi) = line.next() { midashi } else { return Err(""); };
        let entries = if let Some(entries) = line.next() { entries } else { return Err(""); };
        let entries = entries.split('/');
        for entry in entries {
            if entry != "" {
                let mut entry = entry.split(';');
                let kouho = if let Some(text) = entry.next() { text } else { continue; };
                let annotation = entry.next().map(|entry| Arc::new(entry.to_string()));
                result.push(
                    Arc::new(
                        Candidate::new(
                            Arc::new(midashi.to_string()),
                            false,
                            Arc::new(kouho.to_string()),
                            annotation,
                            Some(kouho.to_string()),
                        )
                    )
                )
            }
        }
        Ok(DictEntry {
            midashi: midashi.to_string(),
            candidates: result,
        })
    }

    fn load(&mut self) {
        let filename = "src/data/SKK-JISYO.L";
        let dict_file = File::open(filename).expect(&format!("file {} not found", filename));
        let enc = Encoding::for_label("euc-jp".as_bytes());
        let decoder = DecodeReaderBytesBuilder::new()
            .encoding(enc)
            .build(dict_file);
        let reader = BufReader::new(decoder);
        let mut mode = EntryProcessingMode::OkuriAri;
        let mut okuri_nashi = BTreeMap::new();
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if ";; okuri-nasi entries.".eq(&line) {
                        eprintln!("Found split line. ");
                        mode = EntryProcessingMode::OkuriNasi;
                    }
                    if mode == EntryProcessingMode::OkuriNasi {
                        let parsed = OnMemoryDict::split_candidates(&line);
                        match parsed {
                            Ok(parsed) => {
                                //eprintln!("{}", parsed.midashi);
                                okuri_nashi.insert(parsed.midashi.clone(), parsed);
                            }
                            Err(_) => {
                                eprintln!("Dict is ill formatted. Ignored line {}", &line);
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
        self.okuri_nashi_dictionary = okuri_nashi;
    }

    fn reload(&mut self) {
        self.load();
    }

    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry> {
        self.okuri_nashi_dictionary.get(midashi)
    }

    ///
    /// TODO: midashi から始まるエントリを全て返す。i.e. skkserv 4 command
    /// e.g.
    /// complete('あ') -> ["あい", "あいさつ"]
    ///
    fn complete(&self, midashi: &str) /* -> Option<&Vec<&str>>?*/ {
        unimplemented!("complete")
    }

    fn purge_candidate(&self/* ,midashi, &candidate */) -> bool {
        unimplemented!("purge_candidate")
    }

    ///
    /// It is OK not to save on memory dictionary.
    /// Probably only for user dictionary?
    ///
    fn save_dict(&self) -> bool {
        false
    }

// reorder?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load() {
        let okuri_nashi_dictionary = BTreeMap::new();
        let mut dict = OnMemoryDict { okuri_nashi_dictionary };
        dict.load();

        let okuri_nashi = dict.okuri_nashi_dictionary;
        assert_ne!(0, okuri_nashi.len());
        let Candidate { kouho_text, .. } = ((okuri_nashi.get("あい").unwrap()).candidates[0]).as_ref();
        assert_eq!("", *kouho_text.as_ref());
    }

    #[test]
    fn split_candidate_okuri_nashi() {
        let result = OnMemoryDict::split_candidates(
            "あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/"
        );
        let result = result.unwrap();
        assert_eq!("あい", result.midashi);
        let Candidate { kouho_text, annotation, .. } = result.candidates[0].as_ref();
        assert_eq!("愛", *kouho_text.as_ref());
        assert_eq!(None, *annotation);
        let Candidate { kouho_text, annotation, .. } = result.candidates[5].as_ref();
        assert_eq!("亜衣", *kouho_text.as_ref());
        assert_eq!("人名", *(annotation.as_ref()).expect("亜衣 doesn't have annotation").as_ref());
    }

    #[test]
    fn split_candidate_okuri_ari() {
        let result = OnMemoryDict::split_candidates(
            "おどr /踊;dance/躍;jump/踴;「踊」の異体字/"
        );
        let result = result.unwrap();
        assert_eq!("おどr", result.midashi);
        let Candidate { kouho_text, annotation,.. } = result.candidates[0].as_ref();
        assert_eq!("踊", *kouho_text.as_ref());
        assert_eq!("dance", *(annotation.as_ref()).expect("踊 in sense of dance doesn't have annotation").as_ref());
        let Candidate { kouho_text, annotation,.. } = result.candidates[1].as_ref();
        assert_eq!("躍".to_string(), *kouho_text.as_ref());
        assert_eq!("jump".to_string(), *(annotation.as_ref()).expect("躍 in sense of jump doesn't have annotation.").as_ref());
    }
}
