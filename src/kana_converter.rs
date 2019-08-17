use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use sequence_trie::SequenceTrie;

use crate::keyevent::KeyEvent;

pub(crate) type Converted = String;
pub(crate) type CarryOver = Vec<char>;

#[derive(Clone, Deserialize, Debug, Serialize)]
pub(crate) struct KanaConverter {
    // Maybe change value to input-kana-* command etc?
    process_map: SequenceTrie<char, (Converted, CarryOver)>
}

impl KanaConverter {
    pub fn combined_key(key_event: &KeyEvent, unprocessed: &[char]) -> Vec<char> {
        let mut combined = vec![];
        combined.extend_from_slice(unprocessed);

        match key_event.get_symbol_char() {
            None => {
                combined
            }
            Some(key_char) => {
                combined.push(key_char.to_ascii_lowercase());
                combined
            }
        }
    }

    pub fn convert<'a>(&'a self, kana: &[char]) -> Option<&'a (Converted, CarryOver)> {
        self.process_map.get(kana)
    }

    pub fn can_continue(&self, key_event: &KeyEvent, unprocessed: &[char]) -> bool {
        match self.get_node(key_event, unprocessed) {
            None => {
                false
            }
            Some(_) => {
                true
            }
        }
    }

    fn get_node(&self, key_event: &KeyEvent, unprocessed: &[char]) -> Option<&SequenceTrie<char, (Converted, CarryOver)>> {
        let key = KanaConverter::combined_key(key_event, unprocessed);
        self.process_map.get_node(&key)
    }

    fn converter_from_string(contents: &str) -> Self {
        let mut process_map = SequenceTrie::new();
        let map: HashMap<String, (String, String)> = serde_json::from_str(&contents).expect("content error");
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

        Self {
            process_map
        }
    }

    fn converter_from_file(filename: &str) -> Self {
        let mut file = File::open(filename).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");

        KanaConverter::converter_from_string(&contents)
    }

    pub fn default_converter() -> Self {
        KanaConverter::converter_from_file("src/rule/hiragana.json")
    }
}


#[cfg(test)]
impl KanaConverter {
    fn test_converter() -> Self {
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

        KanaConverter {
            process_map: process_list,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_with_unprocessed() {
        let next_key = KanaConverter::combined_key(&KeyEvent::from_str("a").unwrap(), &vec!['b']);
        assert_eq!(vec!['b', 'a'], next_key);
    }

    #[test]
    fn combine_no_unprocessed() {
        let next_key = KanaConverter::combined_key(&KeyEvent::from_str("k").unwrap(), &vec![]);
        assert_eq!(vec!['k'], next_key);
    }

    #[test]
    fn combine_capital() {
        let next_key = KanaConverter::combined_key(&KeyEvent::from_str("B").unwrap(), &vec![]);
        assert_eq!(vec!['b'], next_key);
    }

//    #[test]
//    fn na() {
//        let next_key = KanaConverter::combined_key(&KeyEvent::from_str("n").unwrap(), &vec![]);
//        assert_eq!(vec!['n'], next_key);
//    }

    #[test]
    fn converter_from_string() {
        let content = r#"
        {
            "a": ["", "あ" ],
            "bb": ["b", "っ" ],
            "ba": ["", "ば" ],
            "be": ["", "べ" ]
        }
        "#.to_string();
        let converter = KanaConverter::converter_from_string(&content);

        let (converted, carry_over) = converter.convert(&['a']).unwrap();
        assert_eq!("あ", converted);
        assert_eq!(Vec::<char>::with_capacity(0), *carry_over);
    }

    #[test]
    fn convert() {
        let converter = KanaConverter::test_converter();

        let result = converter.convert(&['k']);
        assert_eq!(None, result);
    }
}