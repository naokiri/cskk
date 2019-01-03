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

#[derive(Debug)]
pub(crate) struct Candidate {
    kouho: String,
    // TODO: okuri of personal dictionary?
    // okuri: Option<String>,
    annotation: Option<String>,
}

struct DictEntry {
    midashi: String,
    candidates: Vec<Candidate>,
}

struct OnMemoryDict {
    okuri_nashi_dictionary: BTreeMap<String, Vec<Candidate>>,
}

#[derive(PartialEq)]
enum EntryProcessingMode {
    OkuriAri,
    OkuriNasi,
}

///
/// skkjisyo dict
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
                let annotation = entry.next().map(|entry| entry.to_string());
                result.push(Candidate {
                    kouho: kouho.to_string(),
                    annotation,
                });
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
                                okuri_nashi.insert(parsed.midashi, parsed.candidates);
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

    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&Vec<Candidate>> {
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
    /// It is OK not to save on memory dict.
    /// Probably only for user dict?
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
        assert_eq!("", (*okuri_nashi.get("あい").unwrap())[0].kouho);
    }

    #[test]
    fn split_candidate() {
        let result = OnMemoryDict::split_candidates("あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/");
        let result = result.unwrap();
        assert_eq!("あい", result.midashi);
        assert_eq!("愛", result.candidates[0].kouho);
        assert_eq!(None, result.candidates[0].annotation);
        assert_eq!("亜衣", result.candidates[5].kouho);
        assert_eq!(Some("人名".to_string()), result.candidates[5].annotation);
    }
}