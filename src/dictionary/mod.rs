use std::sync::Arc;

use crate::dictionary::candidate::Candidate;
use crate::dictionary::static_dict::StaticFileDict;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use log::warn;

pub mod candidate;
pub mod static_dict;

#[derive(Debug)]
pub struct DictEntry {
    midashi: String,
    candidates: Vec<Arc<Candidate>>,
}

impl DictEntry {
    pub fn get_candidates(&self) -> &Vec<Arc<Candidate>> {
        &self.candidates
    }
}

// C側に出す時にSizedである必要があり、enumでラップする。
#[derive(Debug)]
pub enum CskkDictionary {
    StaticFile(StaticFileDict),
    // UserFile(UserFileDict)
}

pub trait Dictionary {
    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry>;
    fn is_read_only(&self) -> bool;
    //
    // TODO: midashi から始まるエントリを全て返す。i.e. skkserv 4 command
    // e.g.
    // complete('あ') -> ["あい", "あいさつ"]
    //
    // fn complete(&self, _midashi: &str) /* -> Option<&Vec<&str>>?*/

    // fn purge_candidate(&self/* ,midashi, &candidate */) -> bool
    // fn save_dictionary(&self) -> bool {
}

fn split_candidates(line: &str) -> Result<DictEntry, &str> {
    let mut result = Vec::new();
    let mut line = line.trim().split(' ');
    let midashi = if let Some(midashi) = line.next() { midashi } else { return Err(""); };
    let entries = if let Some(entries) = line.next() { entries } else { return Err(""); };
    let entries = entries.split('/');
    for entry in entries {
        if !entry.is_empty() {
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

fn load_dictionary(file_path: &str, encode: &[u8]) -> BTreeMap<String, DictEntry> {
    let dict_file = File::open(file_path).unwrap_or_else(|_| panic!("file {} not found", file_path));
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
                    let parsed = split_candidates(&line);
                    match parsed {
                        Ok(parsed) => {
                            //eprintln!("{}", parsed.midashi);
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
    dictionary
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split_candidate_okuri_nashi() {
        let result = split_candidates(
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
        let result = split_candidates(
            "おどr /踊;dance/躍;jump/踴;「踊」の異体字/"
        );
        let result = result.unwrap();
        assert_eq!("おどr", result.midashi);
        let Candidate { kouho_text, annotation, .. } = result.candidates[0].as_ref();
        assert_eq!("踊", *kouho_text.as_ref());
        assert_eq!("dance", *(annotation.as_ref()).expect("踊 in sense of dance doesn't have annotation").as_ref());
        let Candidate { kouho_text, annotation, .. } = result.candidates[1].as_ref();
        assert_eq!("躍".to_string(), *kouho_text.as_ref());
        assert_eq!("jump".to_string(), *(annotation.as_ref()).expect("躍 in sense of jump doesn't have annotation.").as_ref());
    }

}