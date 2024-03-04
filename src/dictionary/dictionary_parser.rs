use nom::branch::{alt, permutation};
use nom::bytes::complete::{take_till1, take_while1};
use nom::character::complete::char;
use nom::combinator::{all_consuming, map, opt};
use nom::multi::many1;
use nom::IResult;
use std::collections::BTreeMap;

//!
//! BNF風に表現
//!
//! <entry> ::= <midashi> " "+ <candidates>
//! <midashi> ::= (<char> except ' ')+
//! <candidates> ::= "/" (<candidate>"/")+
//! <candidate> ::= <no-strict-okuri-candidate> | <strict-okuri-candidates>
//! <no-strict-okuri-candidate> ::= <kouho> (";"<annotation>)?
//! <strict-okuri-candidates> ::= "[" <okuri> "/" (<no-strict-okuri-candidate> "/")+ "]"
//! <kouho> ::= (<char> except '/',';','[',']' )+
//! <annotation> ::= (<char> except '/',';')+
//! <okuri> ::= <hiragana>+
//! <hiragana> ::= U+3041..U+3096
//!

#[derive(PartialEq, Debug, Clone)]
pub(in crate::dictionary) struct CandidatePrototype<'a> {
    pub(in crate::dictionary) kouho: &'a str,
    pub(in crate::dictionary) annotation: Option<&'a str>,
}

#[derive(PartialEq, Debug, Clone)]
pub(in crate::dictionary) struct DictEntryPrototype<'a> {
    pub(in crate::dictionary) midashi: &'a str,
    pub(in crate::dictionary) candidates: BTreeMap<&'a str, Vec<CandidatePrototype<'a>>>,
}

/// 辞書のエントリを読む
pub(in crate::dictionary) fn entry(input: &str) -> IResult<&str, DictEntryPrototype> {
    let (rest, (midashi, _, candidates)) = all_consuming(permutation((
        midashi,
        take_while1(|c| c == ' '),
        candidates,
    )))(input)?;

    Ok((
        rest,
        DictEntryPrototype {
            midashi,
            candidates,
        },
    ))
}

fn midashi(input: &str) -> IResult<&str, &str> {
    let (i, midashi) = take_till1(|c: char| c == ' ')(input)?;
    Ok((i, midashi))
}

/// 先頭の'/'を含む'/'で囲われた候補リスト全体からcandidate全部
fn candidates(input: &str) -> IResult<&str, BTreeMap<&str, Vec<CandidatePrototype>>> {
    // Make sure starts with '/'
    //let (_, (_, parsed_cands)) = all_consuming(permutation((char('/'), many1(candidate))))(input)?;
    let (_, (_, parsed_cands)) = permutation((char('/'), many1(candidate)))(input)?;

    let mut result = BTreeMap::<&str, Vec<CandidatePrototype>>::new();
    for mut cand_map in parsed_cands {
        for (okuri, value) in cand_map.iter_mut() {
            if let Some(candidates) = result.get_mut(*okuri) {
                candidates.append(value);
            } else {
                let mut new_candidates = vec![];
                new_candidates.append(value);
                result.insert(okuri, new_candidates);
            }
        }
    }
    Ok(("", result))
}

/// 先頭の'/'を含まない部分から、Vec<CandidatePrototype>の厳密な送り仮名からのマップを返す。
/// 通常のcandidateだと空文字列からのマップで1要素のもの、厳密送りだと再帰的に含まれるので複数要素。
fn candidate(input: &str) -> IResult<&str, BTreeMap<&str, Vec<CandidatePrototype>>> {
    let (i, (result, _)) = permutation((
        alt((
            strict_okuri_candidates,
            map(no_strict_okuri_candidate, |cand: CandidatePrototype| {
                let mut map = BTreeMap::new();
                map.insert("", vec![cand]);
                map
            }),
        )),
        char('/'),
    ))(input)?;
    Ok((i, result))
}

/// 先頭の\[と末尾の\]を含む厳密な送り仮名候補列('かな文字列/候補/候補/')を受けてその文字列からの候補マップを返す
fn strict_okuri_candidates(input: &str) -> IResult<&str, BTreeMap<&str, Vec<CandidatePrototype>>> {
    let (i, (_, okuri_kana, _, cands, _)) = permutation((
        char('['),
        take_while1(|c: char| ('ぁ'..'ゖ').contains(&c)),
        char('/'),
        many1(map(
            permutation((no_strict_okuri_candidate, char('/'))),
            |(cand, _)| cand,
        )),
        char(']'),
    ))(input)?;
    let mut result = BTreeMap::new();

    result.insert(okuri_kana, cands);
    Ok((i, result))
}

/// 先頭の'/'を含まない候補部分から候補と存在するならばアノテーションを解釈する。厳密な送り仮名の候補は解釈できない。
/// 候補と次の'/'から始まる残りの部分を返す。
fn no_strict_okuri_candidate(input: &str) -> IResult<&str, CandidatePrototype> {
    //let (i, _) = verify(take(1usize), |c: &str| c == "/")(input)?;
    // let (i, _) = char('/')(input)?;
    let (i, cand) = take_till1(|c: char| is_no_strict_okuri_candidate_illegal_char(&c))(input)?;

    let (i, annotation_opt) = opt(permutation((
        char(';'),
        take_till1(|c: char| is_annotation_illegal_char(&c)),
    )))(i)?;

    if let Some((_, a)) = annotation_opt {
        Ok((
            i,
            CandidatePrototype {
                kouho: cand,
                annotation: Some(a),
            },
        ))
    } else {
        Ok((
            i,
            CandidatePrototype {
                kouho: cand,
                annotation: None,
            },
        ))
    }
}

// true when contains chars not good for no strict okuri candidate: '[', ']', '/', ';'
fn is_no_strict_okuri_candidate_illegal_char(c: &char) -> bool {
    ['/', ';', '[', ']'].contains(c)
}

fn is_annotation_illegal_char(c: &char) -> bool {
    ['/', ';'].contains(c)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_midashi() {
        let (_i, result) = midashi("ほげr ////").unwrap();
        assert_eq!(result, "ほげr");
    }

    #[test]
    fn basic_candidates() {
        let (rest, result) = candidates("/愛;love/相/[す/愛/]/").unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(
            "",
            vec![
                CandidatePrototype {
                    kouho: "愛",
                    annotation: Some("love"),
                },
                CandidatePrototype {
                    kouho: "相",
                    annotation: None,
                },
            ],
        );
        expected.insert(
            "す",
            vec![CandidatePrototype {
                kouho: "愛",
                annotation: None,
            }],
        );

        assert_eq!(rest, "");
        assert_eq!(result, expected)
    }

    #[test]
    fn basic_candidate() {
        let (rest, result) = candidate("愛/相/").unwrap();
        assert_eq!(rest, "相/");
        let mut expected = BTreeMap::new();
        expected.insert(
            "",
            vec![CandidatePrototype {
                kouho: "愛",
                annotation: None,
            }],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn strict_okuri_candidate_in_candidates() {
        let (rest, result) = candidate("[つ/打;hit/討/]/打/").unwrap();
        assert_eq!(rest, "打/");
        let mut expected = BTreeMap::new();
        expected.insert(
            "つ",
            vec![
                CandidatePrototype {
                    kouho: "打",
                    annotation: Some("hit"),
                },
                CandidatePrototype {
                    kouho: "討",
                    annotation: None,
                },
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn basic_strict_okuri_candidate() {
        let mut expected = BTreeMap::new();
        expected.insert(
            "って",
            vec![
                CandidatePrototype {
                    kouho: "送",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "贈",
                    annotation: None,
                },
            ],
        );
        let (i, result) = strict_okuri_candidates("[って/送/贈/]").unwrap();
        assert_eq!(i, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn no_strict_okuri_candidate_test() {
        let (rest, result) = no_strict_okuri_candidate("送/贈/").unwrap();
        assert_eq!(
            result,
            CandidatePrototype {
                kouho: "送",
                annotation: None
            }
        );
        assert_eq!(rest, "/贈/");
    }

    #[test]
    fn no_strict_okuri_candidate_with_annotation() {
        let (rest, result) = no_strict_okuri_candidate("送;アノテーション/贈/").unwrap();
        assert_eq!(
            result,
            CandidatePrototype {
                kouho: "送",
                annotation: Some("アノテーション")
            }
        );
        assert_eq!(rest, "/贈/");
    }

    #[test]
    fn strict_okuri_candidate_with_annotation() {
        let mut expected = BTreeMap::new();
        expected.insert(
            "う",
            vec![
                CandidatePrototype {
                    kouho: "合",
                    annotation: Some("[match]"),
                },
                CandidatePrototype {
                    kouho: "会",
                    annotation: Some("[meet]"),
                },
            ],
        );
        let (rest, result) = strict_okuri_candidates("[う/合;[match]/会;[meet]/]/").unwrap();
        assert_eq!(result, expected);
        assert_eq!(rest, "/");
    }

    #[test]
    fn annotation_with_bracket() {
        let (rest, result) = candidates("/愛;love/藍;color[004c71]/[す/愛;[love]/]/").unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(
            "",
            vec![
                CandidatePrototype {
                    kouho: "愛",
                    annotation: Some("love"),
                },
                CandidatePrototype {
                    kouho: "藍",
                    annotation: Some("color[004c71]"),
                },
            ],
        );
        expected.insert(
            "す",
            vec![CandidatePrototype {
                kouho: "愛",
                annotation: Some("[love]"),
            }],
        );

        assert_eq!(rest, "");
        assert_eq!(result, expected)
    }

    #[test]
    fn github_issue244() {
        let (rest, result) = entry("よし /由/葦/葭/葭/余資/余矢;[数学]versed cosine/好/良/美/吉/純/義/喜/善/佳/圭/慶/祥/芳/嘉/克/宜/淑/禎/禧/譱/縦/").unwrap();
        let mut candidates = BTreeMap::new();
        candidates.insert(
            "",
            vec![
                CandidatePrototype {
                    kouho: "由",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "葦",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "葭",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "葭",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "余資",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "余矢",
                    annotation: Some("[数学]versed cosine"),
                },
                CandidatePrototype {
                    kouho: "好",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "良",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "美",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "吉",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "純",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "義",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "喜",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "善",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "佳",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "圭",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "慶",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "祥",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "芳",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "嘉",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "克",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "宜",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "淑",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "禎",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "禧",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "譱",
                    annotation: None,
                },
                CandidatePrototype {
                    kouho: "縦",
                    annotation: None,
                },
            ],
        );
        let expected = DictEntryPrototype {
            midashi: "よし",
            candidates,
        };

        assert_eq!(rest, "");
        assert_eq!(result, expected)
    }
}
