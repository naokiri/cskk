use regex::Regex;

lazy_static! {
    static ref NUMERIC_REGEX: Regex = Regex::new(r"\d+").unwrap();
}

static DIGIT_CHARACTER_STRINGS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
static DIGIT_CHARACTER_ZENKAKU_STRINGS: [&str; 10] =
    ["０", "１", "２", "３", "４", "５", "６", "７", "８", "９"];
static DIGIT_CHARACTER_KANJI_STRINGS: [&str; 10] =
    ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"];

static NUMERIC_KURAI_KANJI_STRINGS: [&str; 5] = ["", "万", "億", "兆", "京"];

static NUMERIC_CHARACTER_KANJI_STRINGS: [&str; 17] = [
    "〇", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十", "百", "千", "万", "億", "兆",
    "京",
];
// ddskkで元々使われていた大字
static NUMERIC_DAIJI_STRINGS: [&str; 17] = [
    "零", "壱", "弐", "参", "四", "伍", "六", "七", "八", "九", "拾", "百", "阡", "萬", "億", "兆",
    "京",
];
// 商業登記法に基く法務省令 商業登記規則 や 戸籍法等による大字
static NUMERIC_LEGAL_DAIJI_STRINGS: [&str; 17] = [
    "〇", "壱", "弐", "参", "四", "五", "六", "七", "八", "九", "拾", "百", "千", "万", "億", "兆",
    "京",
];

pub(crate) fn numeric_to_kanji_each(original: &str) -> String {
    let mut result = original.to_owned();
    for (idx, num_str) in DIGIT_CHARACTER_STRINGS.iter().enumerate() {
        result = result.replace(*num_str, DIGIT_CHARACTER_KANJI_STRINGS[idx]);
    }
    result
}

///
/// 数字入りの文字列の数字部分を位取りされた漢字にして返す。
///
pub(crate) fn numeric_to_simple_kanji_as_number(original: &str) -> String {
    let mut result = original.to_owned();
    if let Some(numeric_match) = NUMERIC_REGEX.find(original) {
        let num_str = numeric_match.as_str().to_string();
        result.replace_range(
            numeric_match.range(),
            &format_number_kuraidori_nashi_kanji(num_str.parse().unwrap(), false),
        );
    }

    result
}

///
/// 数字入りの文字列の数字部分を大字にして返す。legal_daijiがtrueの時は現代の日本の法令で定められた大字のみ使用する。
///
pub(crate) fn numeric_to_daiji_as_number(original: &str, legal_daiji: bool) -> String {
    let replacement_strings;
    if legal_daiji {
        replacement_strings = NUMERIC_LEGAL_DAIJI_STRINGS;
    } else {
        replacement_strings = NUMERIC_DAIJI_STRINGS;
    }
    let mut result = original.to_owned();
    if let Some(numeric_match) = NUMERIC_REGEX.find(original) {
        let num_str = numeric_match.as_str().to_string();
        let mut replacing_kanji_str =
            format_number_kuraidori_nashi_kanji(num_str.parse().unwrap(), true);
        let mut tmp_kanji_str;
        for (i, kanji) in NUMERIC_CHARACTER_KANJI_STRINGS.iter().enumerate() {
            tmp_kanji_str = replacing_kanji_str.replace(kanji, replacement_strings[i]);
            replacing_kanji_str = tmp_kanji_str;
        }
        result.replace_range(numeric_match.range(), &replacing_kanji_str);
    }

    result
}

const MAX_KETA: usize = 20;
// pub ではないのでDoc testでimportできない
///
/// 数字を漢字表記する。
/// explict_oneがtrueの時は一千,一百,のように1を省略しない。
///
/// assert_eq!(numeric_form_changer::format_number_kuraidori_nashi_kanji(111, false), "百十一");
/// assert_eq!(format_number_kuraidori_nashi_kanji(111, true), "一百一十一");
fn format_number_kuraidori_nashi_kanji(num: u64, explicit_one: bool) -> String {
    if num >= 10000000000000000000 {
        return "数字が大きすぎます".to_string();
    }

    if num == 0 {
        return DIGIT_CHARACTER_KANJI_STRINGS[0].to_owned();
    }

    let mut current = num;
    let mut keta: [u8; MAX_KETA] = [0; MAX_KETA];
    for i in keta.iter_mut() {
        *i = (current % 10) as u8;
        current /= 10;
    }

    let mut result = String::new();
    for i in (0..MAX_KETA / 4).rev() {
        if !(keta[i * 4] == 0
            && keta[i * 4 + 1] == 0
            && keta[i * 4 + 2] == 0
            && keta[i * 4 + 3] == 0)
        {
            let n = keta[i * 4 + 3];
            if n != 0 {
                if n != 1 || explicit_one {
                    result.push_str(DIGIT_CHARACTER_KANJI_STRINGS[n as usize]);
                }
                result.push('千');
            }
            let n = keta[i * 4 + 2];
            if n != 0 {
                if n != 1 || explicit_one {
                    result.push_str(DIGIT_CHARACTER_KANJI_STRINGS[n as usize]);
                }
                result.push('百');
            }
            let n = keta[i * 4 + 1];
            if n != 0 {
                if n != 1 || explicit_one {
                    result.push_str(DIGIT_CHARACTER_KANJI_STRINGS[n as usize]);
                }
                result.push('十');
            }
            let n = keta[i * 4];
            if n != 0 {
                result.push_str(DIGIT_CHARACTER_KANJI_STRINGS[n as usize]);
            }
            result.push_str(NUMERIC_KURAI_KANJI_STRINGS[i])
        }
    }
    result
}

pub(crate) fn numeric_to_zenkaku(original: &str) -> String {
    let mut result = original.to_owned();
    for (idx, num_str) in DIGIT_CHARACTER_STRINGS.iter().enumerate() {
        result = result.replace(*num_str, DIGIT_CHARACTER_ZENKAKU_STRINGS[idx]);
    }
    result
}

///
/// numeric string to thousand separator as defined in the 文化庁 公用文作成の要領 3.3 (or I-4 イ. in the new version suggested in 2021)
///
pub(crate) fn numeric_to_thousand_separator(original: &str) -> String {
    let mut result = String::new();
    if let Some(numeric_match) = NUMERIC_REGEX.find(original) {
        let num_char = numeric_match.as_str();
        let num_len = num_char.len();
        if num_len < 4 {
            return num_char.to_string();
        }
        let mut cnt = 3 - (num_len % 3);

        for c in num_char.chars() {
            result.push(c);
            if cnt % 3 == 2 && cnt < num_len {
                result.push(',');
            }
            cnt += 1;
        }
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn to_zenkaku() {
        assert_eq!(numeric_to_zenkaku(&"foobar123"), "foobar１２３");
    }

    #[test]
    fn to_each_kanji() {
        assert_eq!(numeric_to_kanji_each(&"foobar123"), "foobar一二三");
    }

    #[test]
    fn to_kanji_as_num() {
        assert_eq!(
            numeric_to_simple_kanji_as_number(&"foobar1234"),
            "foobar千二百三十四"
        );
    }

    #[test]
    fn format_kuraidori() {
        assert_eq!(
            format_number_kuraidori_nashi_kanji(123456789987654321, false),
            "十二京三千四百五十六兆七千八百九十九億八千七百六十五万四千三百二十一"
        )
    }

    #[test]
    fn thousand_separator() {
        assert_eq!(
            numeric_to_thousand_separator(&"123"),
            "123"
        );
        assert_eq!(
        numeric_to_thousand_separator(&"12345"),
            "12,345"
        );
        assert_eq!(
            numeric_to_thousand_separator(&"123456"),
            "123,456"
        );
    }
}
