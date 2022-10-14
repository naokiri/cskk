use crate::dictionary::dictionary_parser::{entry, CandidatePrototype, DictEntryPrototype};
use crate::dictionary::DictionaryCandidate;
use crate::dictionary::{Candidate, CompositeKey};
use crate::error::CskkError;
use anyhow::bail;
use nom::Finish;
use regex::{Captures, Regex};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub(crate) struct DictEntry {
    pub(in crate::dictionary) midashi: String,
    // 本来はエントリ自体が持つものではないが、
    // 過去に送りありエントリと無しエントリを混ぜて扱っていたため、互換性のために区別をここにも持っている。
    has_okuri: bool,
    // 厳密な送り仮名がない場合や送りなしエントリは空文字列からのマップ
    strict_okuri_candidate_map: BTreeMap<String, Vec<DictionaryCandidate>>,
}

impl DictEntry {
    /// Usually, DictEntry should be created from [from_skkjisyo_line]
    ///
    /// Create new DictEntry that has single candidate
    /// This is for registration of new composition.
    ///
    pub(in crate::dictionary) fn new(
        midashi: &str,
        composite_key: &CompositeKey,
        candidate: &Candidate,
    ) -> Self {
        let mut new_map = BTreeMap::new();
        if let Some(strict_okuri) = composite_key.get_okuri() {
            new_map.insert(
                strict_okuri.to_owned(),
                vec![DictionaryCandidate::from_candidate(candidate)],
            );
        }
        // even for okuri-ari key, register a non strict okuri entry
        new_map.insert(
            "".to_string(),
            vec![DictionaryCandidate::from_candidate(candidate)],
        );

        Self {
            midashi: midashi.to_string(),
            has_okuri: composite_key.has_okuri(),
            strict_okuri_candidate_map: new_map,
        }
    }

    /// candidateが含まれなかった場合はこのdictentryの先頭に追加する。
    /// candidateがこのdictentryに含まれる場合は与えられたcandidateを先頭にする。
    /// composite_keyが送り仮名を含む場合、厳密な送り仮名なしのエントリと有りのエントリの両方について先頭にする。
    pub(in crate::dictionary) fn prioritize_candidate(
        &mut self,
        composite_key: &CompositeKey,
        candidate: &Candidate,
    ) {
        if let Some(okuri) = composite_key.get_okuri() {
            self.prioritize_candidate_for_okuri(okuri, candidate);
        }

        self.prioritize_candidate_for_okuri("", candidate);
    }

    /// strict_okuriの候補の中でcandidateを優先する。
    fn prioritize_candidate_for_okuri(&mut self, strict_okuri: &str, candidate: &Candidate) {
        // 長さもたいしたことがないのでVecを手作業でRecent used 更新している。LRUCacheを用いるべきか検討の余地あり。
        let mut done = false;
        if let Some(cands) = self.strict_okuri_candidate_map.get_mut(strict_okuri) {
            let index = cands
                .iter()
                .position(|it| it.kouho_text == candidate.kouho_text);
            if let Some(i) = index {
                cands.swap(0, i);
                // done by swap
                done = true;
            }

            if !done {
                cands.insert(0, DictionaryCandidate::from_candidate(candidate));
                // done by insert on top
                done = true;
            }
        }

        if !done {
            // create new mapping for okuri
            self.strict_okuri_candidate_map.insert(
                strict_okuri.to_string(),
                vec![DictionaryCandidate::from_candidate(candidate)],
            );
        }
    }

    ///
    /// composite_keyが送りなしの場合、エントリからcandidateに合うものを削除する。合うものがなかったら何もしない。
    ///
    /// composite_keyが送りありの場合、厳密な送り仮名マッチのエントリと厳密な送り仮名のないエントリの両方からcandidateにあうものを削除する。合うものがなかったら何もしない。
    ///
    pub(in crate::dictionary) fn remove_matching_candidate(
        &mut self,
        composite_key: &CompositeKey,
        candidate: &Candidate,
    ) {
        if let Some(okuri) = composite_key.get_okuri() {
            self.remove_candidate_for_okuri(okuri, candidate);
        }

        self.remove_candidate_for_okuri("", candidate);
    }

    fn remove_candidate_for_okuri(&mut self, strict_okuri: &str, candidate: &Candidate) {
        if let Some(cands) = self.strict_okuri_candidate_map.get_mut(strict_okuri) {
            let index = cands
                .iter()
                .position(|it| *(it.kouho_text) == *candidate.kouho_text);
            if let Some(index) = index {
                cands.remove(index);
            }
        }
    }

    /// strict_okuriのマッチするエントリを返す。
    ///
    pub(in crate::dictionary) fn get_candidates(
        &self,
        strict_okuri: &Option<String>,
    ) -> Option<&Vec<DictionaryCandidate>> {
        return if let Some(okuri) = strict_okuri {
            self.strict_okuri_candidate_map.get(okuri)
        } else {
            self.strict_okuri_candidate_map.get("")
        };
    }

    ///
    /// 過去に送り有無エントリを混ぜていた実装のため、ファイル読み込み側ではデフォルトでは送りありエントリと推定し、
    /// 行処理では見出し語先頭がアルファベットではない(abbrevエントリではないと推定) かつ 末尾にアルファベットが付かないものを送りなしエントリとして扱っている。
    ///
    pub(crate) fn from_skkjisyo_line(line: &str) -> Result<Self, CskkError> {
        lazy_static! {}
        let parsed = entry(line).finish();
        if let Ok((_, dict_entry_prototype)) = parsed {
            Ok(DictEntry::from_dict_entry_prototype(dict_entry_prototype))
        } else {
            Err(CskkError::ParseError(format!("falied to parse {}", line)))
        }
    }

    fn from_dict_entry_prototype(dict_entry_prototype: DictEntryPrototype) -> Self {
        let midashi = DictEntry::process_lisp_fun(dict_entry_prototype.midashi);
        let alphabet = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        ];
        let has_okuri = !midashi.starts_with(alphabet) && midashi.ends_with(alphabet);

        let strict_okuri_candidate_map =
            DictEntry::candidates_from_prototype(dict_entry_prototype.candidates);

        Self {
            midashi,
            has_okuri,
            strict_okuri_candidate_map,
        }
    }

    fn candidates_from_prototype(
        candidates_prototype: BTreeMap<&str, Vec<CandidatePrototype>>,
    ) -> BTreeMap<String, Vec<DictionaryCandidate>> {
        let mut result = BTreeMap::new();
        for (key, val) in candidates_prototype {
            result.insert(
                key.to_string(),
                val.iter()
                    .map(DictionaryCandidate::from_candidate_prototype)
                    .collect(),
            );
        }

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

    /// true if this is likely okuri ari entry
    pub(crate) fn is_okuri_ari_entry(&self) -> bool {
        self.has_okuri
    }
}

impl Display for DictEntry {
    ///
    /// skk辞書内の一行
    ///
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", DictEntry::escape_dictionary_string(&self.midashi))?;
        for (strict_okuri, cands) in &self.strict_okuri_candidate_map {
            if !strict_okuri.is_empty() {
                write!(f, "/[{}", strict_okuri)?;
            }
            for cand in cands {
                write!(f, "/")?;
                write!(f, "{}", cand)?;
            }
            if !strict_okuri.is_empty() {
                write!(f, "/]")?;
            }
        }
        write!(f, "/")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testhelper::init_test_logger;

    #[test]
    fn split_candidate_okuri_nashi() {
        let result = DictEntry::from_skkjisyo_line(
            "あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/"
        ).unwrap();
        assert_eq!("あい", result.midashi);
        let DictionaryCandidate {
            kouho_text,
            annotation,
            ..
        } = &result.strict_okuri_candidate_map.get("").unwrap()[0];
        assert_eq!("愛", *kouho_text);
        assert_eq!(None, *annotation);
        let DictionaryCandidate {
            kouho_text,
            annotation,
            ..
        } = &result.strict_okuri_candidate_map.get("").unwrap()[5];
        assert_eq!("亜衣", *kouho_text);
        assert_eq!(
            "人名",
            annotation.as_ref().expect("亜衣 doesn't have annotation")
        );
    }

    #[test]
    fn split_candidate_okuri_ari() {
        let result = DictEntry::from_skkjisyo_line("おどr /踊;dance/躍;jump/踴;「踊」の異体字/");
        let result = result.unwrap();
        assert_eq!("おどr", result.midashi);
        let DictionaryCandidate {
            kouho_text,
            annotation,
            ..
        } = &result.strict_okuri_candidate_map.get("").unwrap()[0];
        assert_eq!("踊", kouho_text);
        assert_eq!(
            "dance",
            annotation
                .as_ref()
                .expect("踊 in sense of dance doesn't have annotation")
        );
        let DictionaryCandidate {
            kouho_text,
            annotation,
            ..
        } = &result.strict_okuri_candidate_map.get("").unwrap()[1];
        assert_eq!("躍", kouho_text);
        assert_eq!(
            "jump",
            annotation
                .as_ref()
                .expect("躍 in sense of jump doesn't have annotation.")
        );
    }

    #[test]
    fn split_candidate_with_space_in_annotation() {
        init_test_logger();
        let jisyo = "おくr /送;(send)/贈;(present) 賞を贈る/遅/後;気後れ/遲;「遅」の旧字/";
        let result = DictEntry::from_skkjisyo_line(jisyo).unwrap();
        assert_eq!(
            "送",
            &result.strict_okuri_candidate_map.get("").unwrap()[0].kouho_text
        );
        assert_eq!(
            "遅",
            &result.strict_okuri_candidate_map.get("").unwrap()[2].kouho_text
        );
    }

    #[test]
    fn to_string() {
        let jisyo = "あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/";
        let dict_entry = DictEntry::from_skkjisyo_line(jisyo).unwrap();
        assert_eq!(jisyo, &dict_entry.to_string());
    }

    #[test]
    fn to_string_with_strict_okuri() {
        let jisyo = "あいs /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/[さ/ダミー1/ダミー2/]/[せ/ダミー/]/";
        let dict_entry = DictEntry::from_skkjisyo_line(jisyo).unwrap();
        assert_eq!(jisyo, &dict_entry.to_string());
    }

    #[test]
    fn remove() {
        let jisyo = "あい /愛/相/藍/間/合/亜衣;人名/哀;悲哀/埃;(ほこり)塵埃/挨;挨拶/曖;曖昧/瞹;「曖」の異体字/靉/噫;ああ/欸/隘;狭隘/娃/藹;和気藹々/阨;≒隘/穢;(慣用音)/姶;姶良町/会;?/饗;?/";
        let mut dict_entry = DictEntry::from_skkjisyo_line(jisyo).unwrap();
        let candidate = Candidate::new(
            "あい".to_string(),
            false,
            "愛".to_string(),
            None,
            "愛".to_string(),
        );
        let composite_key = CompositeKey::new("あい", None);
        dict_entry.remove_matching_candidate(&composite_key, &candidate);
        let DictionaryCandidate {
            kouho_text,
            annotation,
            ..
        } = &dict_entry.strict_okuri_candidate_map.get("").unwrap()[0];
        assert_eq!("相", kouho_text);
        assert_eq!(None, *annotation);
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

    #[test]
    fn is_okuri_ari() {
        let entry = DictEntry::from_skkjisyo_line("おくr /送;(send)/").unwrap();
        assert!(entry.is_okuri_ari_entry());
    }
}
