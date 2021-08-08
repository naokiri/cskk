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
        let mut file = File::open(filename).expect("file not found");
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
}
