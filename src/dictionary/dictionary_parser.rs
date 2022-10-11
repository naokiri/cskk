use nom::bytes::complete::{tag, take, take_till1, take_until, take_while1};
use nom::character::complete::char;
use nom::combinator::{opt, verify};
use nom::multi::many1;
use nom::sequence::delimited;
use nom::IResult;
use std::collections::BTreeMap;

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
    let (i, midashi) = midashi(input)?;
    let (i, _) = take_while1(|c| c == ' ')(i)?;
    let (_, candidates) = candidates(i)?;

    Ok((
        "",
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

/// '/'を含む'/'で囲われた候補リスト全体からcandidate全部
fn candidates(input: &str) -> IResult<&str, BTreeMap<&str, Vec<CandidatePrototype>>> {
    let (i, parsed_cands) = many1(candidate)(input)?;
    // Make sure ends with '/'
    let _ = char('/')(i)?;

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

/// '/'を含む'/'で囲われた部分から最初の'/'で囲われた部分を解釈し、Vec<CandidatePrototype>の厳密な送り仮名からのマップを返す。
/// 通常のcandidateだと空文字列からのマップで1要素のもの、厳密送りだと再帰的に含まれるので複数要素。
fn candidate(input: &str) -> IResult<&str, BTreeMap<&str, Vec<CandidatePrototype>>> {
    let result;
    let mut rest;
    let (i, is_strict_okuri) = opt(delimited(tag("/["), take_until("]"), char(']')))(input)?;
    rest = i;
    if let Some(delimited_str) = is_strict_okuri {
        // let taken = is_strict_okuri.unwrap();
        let (_, cand) = strict_okuri_candidates(delimited_str)?;
        result = cand;
    } else {
        let (i, cand) = non_strict_okuri_candidate(i)?;
        let mut map = BTreeMap::new();
        map.insert("", vec![cand]);
        result = map;
        rest = i;
    }

    Ok((rest, result))
}

/// []を含まない厳密な送り仮名候補列の[]の間の'かな文字列/候補/候補/'を受けてその文字列からの候補マップを返す
fn strict_okuri_candidates(input: &str) -> IResult<&str, BTreeMap<&str, Vec<CandidatePrototype>>> {
    // from U+3041 to U+3096
    let (i, okuri_kana) = take_while1(|c: char| ('ぁ'..'ゖ').contains(&c))(input)?;
    let (i, cands) = many1(non_strict_okuri_candidate)(i)?;
    let (i, _) = verify(take(1usize), |c: &str| c == "/")(i)?;
    let mut result = BTreeMap::new();
    result.insert(okuri_kana, cands);
    Ok((i, result))
}

// fn non_okuri_candidates(input: &str) -> IResult<&str, BTreeMap<String, CandidatePrototype>> {}

/// '/'を含む候補部分から次の'/'直前までの候補を解釈する、ただし厳密な送り仮名の候補は解釈できない。
fn non_strict_okuri_candidate(input: &str) -> IResult<&str, CandidatePrototype> {
    //let (i, _) = verify(take(1usize), |c: &str| c == "/")(input)?;
    let (i, _) = char('/')(input)?;
    let (i, cand) = verify(take_till1(|c: char| c == '/'), |s: &str| {
        !contains_non_srict_okuri_candidate_illegal_char(s)
    })(i)?;
    let (rest, taken) = take_till1(|c: char| c == ';')(cand)?;
    if rest.is_empty() {
        Ok((
            i,
            CandidatePrototype {
                kouho: taken,
                annotation: None,
            },
        ))
    } else {
        let (rest, _) = char(';')(rest)?;
        Ok((
            i,
            CandidatePrototype {
                kouho: taken,
                annotation: Some(rest),
            },
        ))
    }
}

// true when contains chars not good for non strict okuri candidate: '[', ']', '/'
fn contains_non_srict_okuri_candidate_illegal_char(s: &str) -> bool {
    s.contains('[') || s.contains(']') || s.contains('/')
}

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
        let (rest, result) = candidate("/愛/相/").unwrap();
        assert_eq!(rest, "/相/");
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
        let (rest, result) = candidate("/[つ/打;hit/討/]/打/").unwrap();
        assert_eq!(rest, "/打/");
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
        let (i, result) = strict_okuri_candidates("って/送/贈/").unwrap();
        assert_eq!(i, "");
        assert_eq!(result, expected);
    }

    #[test]
    fn non_strict_okuri_candidate_test() {
        let (rest, result) = non_strict_okuri_candidate("/送/贈/").unwrap();
        assert_eq!(
            result,
            CandidatePrototype {
                kouho: "送",
                annotation: None
            }
        );
        assert_eq!(rest, "/贈/");
    }
}
