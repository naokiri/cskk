use crate::env::filepath_from_xdg_data_dir;
use crate::skk_modes::InputMode;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

pub(crate) struct KanaFormChanger {
    maps: KanaFormMap,
    /// max len in chars. 'う゛' -> 2
    katakana_key_maxlen: usize,
    jisx0201_key_maxlen: usize,
}

#[derive(Deserialize)]
struct KanaFormMap {
    katakana: BTreeMap<String, String>,
    jisx0201: BTreeMap<String, String>,
}

macro_rules! btreemap {
        ($([$key:expr,$val:expr]),*) => {
            {
            let mut map = BTreeMap::new();
            $(
                map.insert($key,$val);
            )*

            map
            }
        };
    }
lazy_static! {
    static ref KANA_ROM_MAP: BTreeMap<&'static str, &'static str> = btreemap![
        ["あ", "a"],
        ["い", "i"],
        ["う", "u"],
        ["え", "e"],
        ["お", "o"],
        ["か", "k"],
        ["き", "k"],
        ["く", "k"],
        ["け", "k"],
        ["こ", "k"],
        ["さ", "s"],
        ["し", "s"],
        ["す", "s"],
        ["せ", "s"],
        ["そ", "s"],
        ["た", "t"],
        ["ち", "t"],
        ["つ", "t"],
        ["て", "t"],
        ["と", "t"],
        ["な", "n"],
        ["に", "n"],
        ["ぬ", "n"],
        ["ね", "n"],
        ["の", "n"],
        ["は", "h"],
        ["ひ", "h"],
        ["ふ", "h"],
        ["へ", "h"],
        ["ほ", "h"],
        ["ま", "m"],
        ["み", "m"],
        ["む", "m"],
        ["め", "m"],
        ["も", "m"],
        ["や", "y"],
        ["ゆ", "y"],
        ["よ", "y"],
        ["ら", "r"],
        ["り", "r"],
        ["る", "r"],
        ["れ", "r"],
        ["ろ", "r"],
        ["わ", "w"],
        ["ゐ", "x"],
        ["ゑ", "x"],
        ["を", "w"],
        ["ん", "n"],
        ["が", "g"],
        ["ぎ", "g"],
        ["ぐ", "g"],
        ["げ", "g"],
        ["ご", "g"],
        ["ざ", "z"],
        ["じ", "z"], // ddskkでは"じ"が送り仮名の場合"j"として処理するのがデフォルト値だが、SKK-JISYO.S等ではzの送り仮名を用いることが多いのでこちらを用いる。
        ["ず", "z"],
        ["ぜ", "z"],
        ["ぞ", "z"],
        ["だ", "d"],
        ["ぢ", "d"],
        ["づ", "d"],
        ["で", "d"],
        ["ど", "d"],
        ["ば", "b"],
        ["び", "b"],
        ["ぶ", "b"],
        ["べ", "b"],
        ["ぼ", "b"],
        ["ぱ", "p"],
        ["ぴ", "p"],
        ["ぷ", "p"],
        ["ぺ", "p"],
        ["ぽ", "p"],
        ["ぁ", "x"],
        ["ぃ", "x"],
        ["ぅ", "x"],
        ["ぇ", "x"],
        ["ぉ", "x"],
        ["っ", "t"], // ddskk 16.2ではxがデフォルトだが、SKK-JISYO.Lなどでは撥音便の用語はtで収録されているため。'いt'等。
        ["ゃ", "x"],
        ["ゅ", "x"],
        ["ょ", "x"],
        ["ゎ", "x"]
    ];
}
impl KanaFormChanger {
    pub fn default_kanaform_changer() -> Self {
        let filepath = filepath_from_xdg_data_dir("libcskk/rule/kana_form.toml");
        if let Ok(filepath) = filepath {
            KanaFormChanger::from_file(&filepath)
        } else {
            KanaFormChanger::from_string("")
        }
    }

    /// pub for e2e test. Use default_kanaform_changer instead.
    pub fn from_file(filename: &str) -> Self {
        let mut file =
            File::open(filename).unwrap_or_else(|_| panic!("file {} not found", filename));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");
        KanaFormChanger::from_string(&contents)
    }

    fn from_string(contents: &str) -> Self {
        let kana_form_map: KanaFormMap =
            toml::from_str(contents).expect("source data file for kana form is broken");
        let katakana_key_maxlen = kana_form_map
            .katakana
            .keys()
            .map(|x| x.chars().count())
            .max()
            .unwrap();
        let jisx0201_key_maxlen = kana_form_map
            .jisx0201
            .keys()
            .map(|x| x.chars().count())
            .max()
            .unwrap();
        KanaFormChanger {
            maps: kana_form_map,
            katakana_key_maxlen,
            jisx0201_key_maxlen,
        }
    }

    ///
    /// 'kana' が最小置き換え単位と仮定して、input modeに合わせた置換をする。
    ///  'う゛' -> 'ヴ', 'ぽ' -> 'ﾎﾟ' 等文字数も変わる可能性がある。
    ///
    #[allow(dead_code)]
    fn adjust_one_kana(&self, input_mode: &InputMode, kana: &str) -> String {
        match input_mode {
            InputMode::Katakana => self
                .maps
                .katakana
                .get(kana)
                .unwrap_or(&kana.to_string())
                .to_owned(),
            InputMode::HankakuKatakana => self
                .maps
                .jisx0201
                .get(kana)
                .unwrap_or(&kana.to_string())
                .to_owned(),
            InputMode::Hiragana => kana.to_string(),
            _ => kana.to_string(),
        }
    }

    ///
    ///  kanaに対してinput modeに合わせた置換をする。
    ///  'う゛' -> 'ヴ', 'ぽ' -> 'ﾎﾟ' 等文字数も変わる可能性がある。
    ///
    pub fn adjust_kana_string(&self, input_mode: InputMode, kana: &str) -> String {
        if input_mode == InputMode::Katakana || input_mode == InputMode::HankakuKatakana {
            let replace_map = if input_mode == InputMode::Katakana {
                &self.maps.katakana
            } else {
                &self.maps.jisx0201
            };
            let maxlen = if input_mode == InputMode::Katakana {
                self.katakana_key_maxlen
            } else {
                self.jisx0201_key_maxlen
            };
            let mut result = "".to_string();
            KanaFormChanger::adjust_kana_string_inner_recur(replace_map, maxlen, kana, &mut result);
            result
        } else {
            kana.to_string()
        }
    }

    /// Greedy match and replace recursion.
    fn adjust_kana_string_inner_recur(
        map: &BTreeMap<String, String>,
        max_len: usize,
        to_adjust: &str,
        adjusted: &mut String,
    ) {
        if to_adjust.is_empty() {
            return;
        };

        for i in (1..max_len + 1).rev() {
            if let Some(replace) = map.get(&to_adjust.chars().take(i).collect::<String>()) {
                adjusted.push_str(replace);
                return KanaFormChanger::adjust_kana_string_inner_recur(
                    map,
                    max_len,
                    &to_adjust.chars().skip(i).collect::<String>(),
                    adjusted,
                );
            }
        }
        adjusted.push(to_adjust.chars().next().unwrap());
        KanaFormChanger::adjust_kana_string_inner_recur(
            map,
            max_len,
            &to_adjust.chars().skip(1).collect::<String>(),
            adjusted,
        )
    }

    ///
    /// ひらがな一文字からローマ字の最初のアルファベット一文字を返す。
    /// ddskkのskk-rom-kana-vectorの対応。
    ///
    pub(crate) fn kana_to_okuri_prefix(kana: &str) -> Option<&str> {
        KANA_ROM_MAP.get(kana).copied()
    }
}

#[cfg(test)]
impl KanaFormChanger {
    pub fn test_kana_form_changer() -> Self {
        KanaFormChanger::from_string(
            "\
[katakana]
\"あ\" = \"ア\"
\"ぁ\" = \"ァ\"
\"い\" = \"イ\"
\"き\" = \"キ\"
\"ん\" = \"ン\"
\"う゛\" = \"ヴ\"
\"ぐ\" = \"グ\"
\"っ\" = \"ッ\"
[jisx0201]
\"あ\" = \"ｱ\"
",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_check() {
        let changer = KanaFormChanger::test_kana_form_changer();
        assert_eq!(changer.maps.jisx0201.get("あ").unwrap(), "ｱ");
        assert_eq!(changer.katakana_key_maxlen, 2);
    }

    #[test]
    fn adjust_kana_string() {
        let changer = KanaFormChanger::test_kana_form_changer();
        let actual = changer.adjust_kana_string(InputMode::Katakana, "う゛ぁいきんぐ");
        assert_eq!("ヴァイキング", actual);
    }

    #[test]
    fn adjust_kana_string_small_tu() {
        let changer = KanaFormChanger::test_kana_form_changer();
        let actual = changer.adjust_kana_string(InputMode::Hiragana, "っ");
        assert_eq!("っ", actual);
    }

    #[test]
    fn kana_to_okuri_prefix() {
        assert_eq!(Some("r"), KanaFormChanger::kana_to_okuri_prefix("り"));
        assert_eq!(Some("s"), KanaFormChanger::kana_to_okuri_prefix("す"));
        assert_eq!(Some("w"), KanaFormChanger::kana_to_okuri_prefix("わ"));
    }
}
