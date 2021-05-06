use crate::dictionary::candidate::Candidate;
use crate::error::CskkError;

#[derive(Debug, Clone)]
pub struct DictEntry {
    pub midashi: String,
    pub candidates: Vec<Candidate>,
}

impl DictEntry {
    pub fn remove_matching_candidate(&mut self, candidate: &Candidate) {
        let index = self
            .candidates
            .iter()
            .position(|it| *(it.kouho_text) == *candidate.kouho_text);
        if let Some(index) = index {
            (*self).candidates.remove(index);
        }
    }

    pub fn insert_as_first_candidate(&mut self, candidate: Candidate) {
        if *candidate.midashi == self.midashi {
            self.candidates.insert(0, candidate);
        }
    }

    pub fn get_candidates(&self) -> &Vec<Candidate> {
        &self.candidates
    }

    pub(crate) fn from_skkjisyo_line(line: &str) -> Result<Self, CskkError> {
        let mut result = Vec::new();
        let mut line = line.trim().split_ascii_whitespace();
        let midashi = if let Some(midashi) = line.next() {
            midashi
        } else {
            return Err(CskkError::Error("No midshi".to_string()));
        };
        let entries = line.fold("".to_string(), |a, b| a + b);
        if entries.is_empty() {
            return Err(CskkError::Error("No entries".to_string()));
        }
        let entries = entries.split('/');
        for entry in entries {
            if !entry.is_empty() {
                if let Ok(candidate) = Candidate::from_skk_jisyo_string(midashi, entry) {
                    result.push(candidate)
                }
            }
        }
        Ok(Self {
            midashi: midashi.to_string(),
            candidates: result,
        })
    }

    // one line of dictionary.
    // e.g.
    // こうほ /候補/好捕/
    pub fn to_skk_jisyo_string(&self) -> String {
        if self.candidates.is_empty() {
            return "".to_string();
        }

        let mut result = "".to_string();
        result.push_str(&format!("{} ", self.midashi));
        for candidate in &self.candidates {
            result.push_str(&format!("/{}", &candidate.to_skk_jisyo_string()));
        }
        result.push('/');
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testhelper::init_test_logger;
    use std::sync::Arc;

    #[test]
    fn split_candidate_okuri_nashi() {
        let result = DictEntry::from_skkjisyo_line(
            "あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/"
        );
        let result = result.unwrap();
        assert_eq!("あい", result.midashi);
        let Candidate {
            kouho_text,
            annotation,
            ..
        } = &result.candidates[0];
        assert_eq!("愛", *kouho_text.as_ref());
        assert_eq!(None, *annotation);
        let Candidate {
            kouho_text,
            annotation,
            ..
        } = &result.candidates[5];
        assert_eq!("亜衣", *kouho_text.as_ref());
        assert_eq!(
            "人名",
            *(annotation.as_ref())
                .expect("亜衣 doesn't have annotation")
                .as_ref()
        );
    }

    #[test]
    fn split_candidate_okuri_ari() {
        let result = DictEntry::from_skkjisyo_line("おどr /踊;dance/躍;jump/踴;「踊」の異体字/");
        let result = result.unwrap();
        assert_eq!("おどr", result.midashi);
        let Candidate {
            kouho_text,
            annotation,
            ..
        } = &result.candidates[0];
        assert_eq!("踊", *kouho_text.as_ref());
        assert_eq!(
            "dance",
            *(annotation.as_ref())
                .expect("踊 in sense of dance doesn't have annotation")
                .as_ref()
        );
        let Candidate {
            kouho_text,
            annotation,
            ..
        } = &result.candidates[1];
        assert_eq!("躍".to_string(), *kouho_text.as_ref());
        assert_eq!(
            "jump".to_string(),
            *(annotation.as_ref())
                .expect("躍 in sense of jump doesn't have annotation.")
                .as_ref()
        );
    }

    #[test]
    fn split_candidate_with_space_in_annotation() {
        init_test_logger();
        let jisyo = "おくr /送;(send)/贈;(present) 賞を贈る/遅/後;気後れ/遲;「遅」の旧字/";
        let result = DictEntry::from_skkjisyo_line(jisyo).unwrap();
        assert_eq!("送", *result.candidates[0].kouho_text);
        assert_eq!("遅", *result.candidates[2].kouho_text);
    }

    #[test]
    fn to_string() {
        let jisyo = "あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/";
        let dict_entry = DictEntry::from_skkjisyo_line(jisyo).unwrap();
        assert_eq!(jisyo, &dict_entry.to_skk_jisyo_string());
    }

    #[test]
    fn remove() {
        let jisyo = "あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/";
        let mut dict_entry = DictEntry::from_skkjisyo_line(jisyo).unwrap();
        let candidate = Candidate::from_skk_jisyo_string("あい", "愛").unwrap();
        dict_entry.remove_matching_candidate(&candidate);
        let Candidate {
            kouho_text,
            annotation,
            ..
        } = &dict_entry.candidates[0];
        assert_eq!("相", *kouho_text.as_ref());
        assert_eq!(None, *annotation);
    }

    #[test]
    fn insert() {
        let jisyo = "あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/";
        let mut dict_entry = DictEntry::from_skkjisyo_line(jisyo).unwrap();
        let candidate = Candidate::from_skk_jisyo_string("あい", "アイ;foo").unwrap();
        dict_entry.insert_as_first_candidate(candidate);
        let Candidate {
            kouho_text,
            annotation,
            ..
        } = &dict_entry.candidates[0];
        assert_eq!("アイ", *kouho_text.as_ref());
        assert_eq!(Some(Arc::new("foo".to_string())), *annotation);
    }
}
