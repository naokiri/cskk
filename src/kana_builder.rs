use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use sequence_trie::SequenceTrie;

use crate::env::filepath_from_xdg_data_dir;
use crate::keyevent::CskkKeyEvent;
use crate::skk_modes::{CommaStyle, PeriodStyle};

pub(crate) type Converted = String;
pub(crate) type CarryOver = Vec<char>;

#[derive(Clone, Debug)]
pub(crate) struct KanaBuilder {
    process_map: SequenceTrie<char, (Converted, CarryOver)>,
}

impl KanaBuilder {
    //!
    //! 未決時にもconvertすると確定してしまうので、ddskkのskk-kana-input実装と違う作りになっている。要再検討
    //!

    /// returns unprocessed vector appending the key_event
    pub fn combined_key(key_event: &CskkKeyEvent, unprocessed: &[char]) -> Vec<char> {
        let mut combined = vec![];
        combined.extend_from_slice(unprocessed);

        match key_event.get_symbol_char() {
            None => combined,
            Some(key_char) => {
                combined.push(key_char.to_ascii_lowercase());
                combined
            }
        }
    }

    /// convert the unprocessed vector into kana and the remaining carryover if matching kana exists
    pub fn convert(&self, kana: &[char]) -> Option<&(Converted, CarryOver)> {
        self.process_map.get(kana)
    }

    ///
    /// Not in the normal convert function because caller should know ",." to treat this specially for composition mode changes.
    ///
    pub fn convert_periods(
        &self,
        kana: &char,
        period_style: PeriodStyle,
        comma_style: CommaStyle,
    ) -> Option<Converted> {
        if *kana == '.' {
            match period_style {
                PeriodStyle::PeriodJa => Some("。".to_string()),
                PeriodStyle::PeriodEn => Some("．".to_string()),
            }
        } else if *kana == ',' {
            match comma_style {
                CommaStyle::CommaJa => Some("、".to_string()),
                CommaStyle::CommaEn => Some("，".to_string()),
            }
        } else {
            None
        }
    }

    // 今のunprocessedに続いて次のkey_eventが来た時にかな変換を続けられるか。
    // e.g.
    // k j -> false
    // t t -> true ('っt' として続けられるため)
    pub fn can_continue(&self, key_event: &CskkKeyEvent, unprocessed: &[char]) -> bool {
        self.get_node(key_event, unprocessed).is_some()
    }

    fn get_node(
        &self,
        key_event: &CskkKeyEvent,
        unprocessed: &[char],
    ) -> Option<&SequenceTrie<char, (Converted, CarryOver)>> {
        let key = KanaBuilder::combined_key(key_event, unprocessed);
        self.process_map.get_node(&key)
    }

    fn converter_from_string(contents: &str) -> Self {
        let mut process_map = SequenceTrie::new();
        let map: HashMap<String, (String, String)> =
            serde_json::from_str(contents).expect("content error");
        for (k, (carry, conv)) in &map {
            let mut key = vec![];
            for c in k.chars() {
                key.push(c);
            }

            let mut carry_over = vec![];
            for c in carry.chars() {
                carry_over.push(c);
            }

            let converted = conv.to_owned();

            process_map.insert(&key, (converted, carry_over));
        }

        Self { process_map }
    }

    /// For e2e test purpose. Use default_converter instead.
    pub fn converter_from_file(filename: &str) -> Self {
        let mut file = File::open(filename).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");

        KanaBuilder::converter_from_string(&contents)
    }

    pub fn default_converter() -> Self {
        let filepath = filepath_from_xdg_data_dir("libcskk/rule/hiragana.json");
        if let Ok(filepath) = filepath {
            KanaBuilder::converter_from_file(&filepath)
        } else {
            KanaBuilder::converter_from_string("")
        }
    }
}

#[cfg(test)]
impl KanaBuilder {
    pub fn test_converter() -> Self {
        let mut process_list = SequenceTrie::new();

        process_list.insert(&['a'], ("あ".to_string(), vec![]));
        process_list.insert(&['i'], ("い".to_string(), vec![]));
        process_list.insert(&['u'], ("う".to_string(), vec![]));
        process_list.insert(&['e'], ("え".to_string(), vec![]));
        process_list.insert(&['o'], ("お".to_string(), vec![]));

        process_list.insert(&['k', 'a'], ("か".to_string(), vec![]));
        process_list.insert(&['k', 'i'], ("き".to_string(), vec![]));
        process_list.insert(&['k', 'u'], ("く".to_string(), vec![]));
        process_list.insert(&['k', 'e'], ("け".to_string(), vec![]));
        process_list.insert(&['k', 'o'], ("こ".to_string(), vec![]));

        KanaBuilder {
            process_map: process_list,
        }
    }

    // Example from ddskk 16.2 skk-kana-input
    fn test_ant_converter() -> Self {
        let mut process_list = SequenceTrie::new();

        process_list.insert(&['a'], ("あ".to_string(), vec![]));
        process_list.insert(&['n'], ("ん".to_string(), vec![]));
        process_list.insert(&['n', 'n'], ("ん".to_string(), vec![]));
        process_list.insert(&['n', 'a'], ("な".to_string(), vec![]));
        process_list.insert(&['t', 'a'], ("た".to_string(), vec![]));
        process_list.insert(&['t', 't'], ("っ".to_string(), vec!['t']));

        KanaBuilder {
            process_map: process_list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_with_unprocessed() {
        let next_key = KanaBuilder::combined_key(&CskkKeyEvent::from_string_representation("a").unwrap(), &['b']);
        assert_eq!(vec!['b', 'a'], next_key);
    }

    #[test]
    fn combine_no_unprocessed() {
        let next_key = KanaBuilder::combined_key(&CskkKeyEvent::from_string_representation("k").unwrap(), &[]);
        assert_eq!(vec!['k'], next_key);
    }

    #[test]
    fn combine_capital() {
        let next_key = KanaBuilder::combined_key(&CskkKeyEvent::from_string_representation("B").unwrap(), &[]);
        assert_eq!(vec!['b'], next_key);
    }

    #[test]
    fn converter_from_string() {
        let content = r#"
        {
            "a": ["", "あ" ],
            "bb": ["b", "っ" ],
            "ba": ["", "ば" ],
            "be": ["", "べ" ]
        }
        "#
        .to_string();
        let converter = KanaBuilder::converter_from_string(&content);

        let (converted, carry_over) = converter.convert(&['a']).unwrap();
        assert_eq!(converted, "あ");
        assert_eq!(Vec::<char>::with_capacity(0), *carry_over);
    }

    #[test]
    fn convert() {
        let converter = KanaBuilder::test_converter();

        let result = converter.convert(&['k']);
        assert_eq!(result, None);
    }

    #[test]
    fn ant_tree_convert() {
        let converter = KanaBuilder::test_ant_converter();
        let result = converter.convert(&['t']);
        assert_eq!(result, None);

        let (kana, carry_over) = converter.convert(&['t', 't']).unwrap();
        assert_eq!("っ", kana);
        assert_eq!(*carry_over, vec!['t'])
    }

    #[test]
    fn can_continue() {
        let converter = KanaBuilder::test_converter();
        let unprocessed = vec![];
        let actual = converter.can_continue(&CskkKeyEvent::from_string_representation("Q").unwrap(), &unprocessed);
        assert!(!actual);
    }
}
