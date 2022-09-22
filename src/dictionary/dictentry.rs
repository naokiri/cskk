use crate::dictionary::candidate::Candidate;
use crate::error::CskkError;
use anyhow::bail;
use regex::{Captures, Regex};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub(crate) struct DictEntry {
    pub(crate) midashi: String,
    pub(crate) candidates: Vec<Candidate>,
}

impl DictEntry {
    pub(crate) fn remove_matching_candidate(&mut self, candidate: &Candidate) {
        let index = self
            .candidates
            .iter()
            .position(|it| *(it.kouho_text) == *candidate.kouho_text);
        if let Some(index) = index {
            self.candidates.remove(index);
        }
    }

    pub(crate) fn insert_as_first_candidate(&mut self, candidate: Candidate) {
        if *candidate.midashi == self.midashi {
            self.candidates.insert(0, candidate);
        }
    }

    pub(crate) fn get_candidates(&self) -> &Vec<Candidate> {
        &self.candidates
    }

    pub(crate) fn from_skkjisyo_line(line: &str) -> Result<Self, CskkError> {
        let mut result = Vec::new();
        let mut line = line.trim().split_ascii_whitespace();
        let midashi = if let Some(midashi) = line.next() {
            DictEntry::process_lisp_fun(midashi)
        } else {
            return Err(CskkError::Error("No midshi".to_string()));
        };
        let entries = line.collect::<Vec<&str>>().join(" ");
        if entries.is_empty() {
            return Err(CskkError::Error("No entries".to_string()));
        }
        let entries = entries.split('/');
        for entry in entries {
            if !entry.is_empty() {
                if let Ok(candidate) = Candidate::from_skk_jisyo_string(&midashi, entry) {
                    result.push(candidate)
                }
            }
        }
        Ok(Self {
            midashi,
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

        let mut result = String::new();
        write!(
            result,
            "{} ",
            DictEntry::escape_dictionary_string(&self.midashi)
        )
        .expect("Failed to allocate jisyo string for dict midashi");
        for candidate in &self.candidates {
            write!(result, "/{}", &candidate.to_skk_jisyo_string())
                .expect("Failed to allocate jisyo string for dict entry");
        }
        result.push('/');
        result
    }

    ///
    /// 互換性のためLisp関数を適用する。
    /// 入れ子ではない単項concatのみ
    /// さらに旧辞書に含まれていたoctal形式のみ対応する。
    /// See https://www.gnu.org/software/emacs/manual/html_node/elisp/General-Escape-Syntax.html
    ///
    /// なんらかの理由で変換できなかった場合、元の文字列のまま返す。
    ///
    pub(crate) fn process_lisp_fun(entry: &str) -> String {
        if let Ok(result) = DictEntry::process_lisp_fun_inner(entry) {
            result
        } else {
            entry.to_owned()
        }
    }

    fn process_lisp_fun_inner(entry: &str) -> anyhow::Result<String> {
        lazy_static! {
            static ref CONCAT_REGEX: Regex = Regex::new(r#"\(concat .*\)"#).unwrap();
            // If subsequent string is [0-7], octal code will end with slash.
            // See https://www.gnu.org/software/emacs/manual/html_node/elisp/Non_002dASCII-in-Strings.html
            static ref OCTAL_REGEX: Regex = Regex::new(r#"\\[01234567]{1,3}\\?"#).unwrap();
            static ref ESCAPE_REGEX: Regex = Regex::new(r#"\\([^0-7])"#).unwrap();
        }
        //let CONCAT_REGEX: Regex = Regex::new(r#"\(concat .*\)"#).unwrap();
        //let OCTAL_REGEX: Regex = Regex::new(r#"\\[01234567]{1,3}\\?"#).unwrap();
        //let ESCAPE_REGEX: Regex = Regex::new(r#"\\([^0-7])"#).unwrap();

        // (being_replaced, to_replace) pair vec
        let mut concat_replacing = vec![];
        for concat_match in CONCAT_REGEX.find_iter(entry) {
            let fullmatch = concat_match
                .as_str()
                .trim_start_matches("(concat")
                .trim_start_matches(' ')
                .trim_end_matches(')')
                .trim_end_matches(' ');
            let fullmatch = fullmatch.strip_prefix('"').unwrap_or(fullmatch);
            let fullmatch = fullmatch.strip_suffix('"').unwrap_or(fullmatch);
            if fullmatch.is_empty() {
                bail!("regex matched to empty concat.");
            }
            let mut octal_replacing = vec![];
            for octal_match in OCTAL_REGEX.find_iter(fullmatch) {
                let octal_string = octal_match
                    .as_str()
                    .trim_start_matches('\\')
                    .trim_end_matches('\\');
                let num = u32::from_str_radix(octal_string, 8)?;
                if let Some(ch) = char::from_u32(num) {
                    octal_replacing.push((octal_match.as_str(), ch.to_string()))
                } else {
                    bail!("regex matched to non digit and can't parse.");
                }
            }
            let mut concat_match_replace_string = fullmatch.to_owned();
            for (be_replaced, to_replace) in octal_replacing {
                concat_match_replace_string =
                    concat_match_replace_string.replacen(be_replaced, &to_replace, 1);
            }
            concat_match_replace_string = ESCAPE_REGEX
                .replace_all(&concat_match_replace_string, |cap: &Captures| {
                    cap[1].to_string()
                })
                .to_string();
            concat_replacing.push((concat_match.as_str(), concat_match_replace_string));
        }

        let mut replace_string = entry.to_owned();
        for (be_replaced, to_replace) in concat_replacing {
            replace_string = replace_string.replacen(be_replaced, &to_replace, 1);
        }

        Ok(replace_string)
    }

    /// escape entry using (concat) if needed
    pub(crate) fn escape_dictionary_string(entry: &str) -> String {
        if entry.find(';').is_some() || entry.find('/').is_some() {
            let mut replacing_string = entry.to_owned();
            replacing_string = replacing_string.replace('/', "\\057");
            replacing_string = replacing_string.replace(';', "\\073");
            replacing_string = replacing_string.replace('"', "\\\"");
            return format!(r#"(concat "{}")"#, replacing_string);
        }

        entry.to_owned()
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

    #[test]
    fn lisp_entry_slash() {
        let result = DictEntry::process_lisp_fun(r#"(concat "DOS\057V")"#);
        assert_eq!(r#"DOS/V"#, result);
    }

    #[test]
    fn lisp_entry_semicolon() {
        let result = DictEntry::process_lisp_fun(r#"(concat "M\073tG")"#);
        assert_eq!(r#"M;tG"#, result);
    }

    #[test]
    fn lisp_entry_dquote() {
        let result = DictEntry::process_lisp_fun(r#"(concat "\"it\"")"#);
        assert_eq!(r#""it""#, result);
    }

    #[test]
    fn escape_dictionary() {
        let result = DictEntry::escape_dictionary_string("Nothing");
        assert_eq!("Nothing", result);
        let result = DictEntry::escape_dictionary_string("(;;/)");
        assert_eq!(r#"(concat "(\073\073\057)")"#, result);
    }
}
