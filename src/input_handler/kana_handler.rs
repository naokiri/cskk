use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use sequence_trie::SequenceTrie;

use crate::input_handler::InputHandler;
use crate::Instruction;
use crate::keyevent::KeyEvent;
use crate::keyevent::SkkKeyModifier;

type Converted = String;
type CarryOver = Vec<char>;


#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct KanaHandler {
    // Maybe change value to input-kana-* command etc?
    process_map: SequenceTrie<char, (Converted, CarryOver)>
}

impl KanaHandler {
    fn handler_from_string(contents: &str) -> Self {
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

    fn handler_from_file(filename: &str) -> Self {
        let mut file = File::open(filename).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");

        KanaHandler::handler_from_string(&contents)
    }

    pub fn default_handler() -> Self {
        KanaHandler::handler_from_file("src/rule/hiragana.json")
    }

    fn next_key(key_char: char, unprocessed: &[char]) -> Vec<char> {
        let mut next_key = vec![];
        next_key.extend_from_slice(unprocessed);
        next_key.push(key_char);
        next_key
    }

    fn get_node(&self, key_event: &KeyEvent, unprocessed: &[char]) -> Option<&SequenceTrie<char, (Converted, CarryOver)>> {
        match key_event.get_symbol_char() {
            None => { None }
            Some(key_char) => {
                self.process_map.get_node(&KanaHandler::next_key(key_char, unprocessed))
            }
        }
    }
}

impl InputHandler for KanaHandler {
    fn can_process(&self, key_event: &KeyEvent, unprocessed: &[char]) -> bool {
        match (self.get_node(key_event, unprocessed), self.get_node(key_event, &[])) {
            (None, None) => { false }
            _ => { true }
        }
    }

    fn get_instruction(&self, key_event: &KeyEvent, unprocessed: &[char]) -> Option<Instruction> {
        match self.get_node(key_event, &unprocessed) {
            Some(node) => {
                match node.value() {
                    None => {
                        Some(Instruction::FlushUnconverted { new_start: key_event.get_symbol_char().expect("Should be safe to unwrap when right after get_node is not None") })
                    }
                    Some((converted, carry_over)) => {
                        let modifier = key_event.get_modifier();
                        if modifier.contains(SkkKeyModifier::Shift) {
                            Some(Instruction::InputKana { converted, carry_over })
                        } else {
                            Some(Instruction::StartComposition { converted, carry_over })
                        }
                    }
                }
            }
            None => {
                None
            }
        }
    }
}

#[cfg(test)]
impl KanaHandler {
    fn test_handler() -> KanaHandler {
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

        KanaHandler {
            process_map: process_list,
        }
    }
}

#[cfg(test)]
mod tests {
    use xkbcommon::xkb::keysyms;

    use super::*;

    #[test]
    fn next_key_with_unprocessed() {
        let next_key = KanaHandler::next_key(KeyEvent::from_str("a").unwrap().get_symbol_char().expect("no symbol char"), &vec!['b']);
        assert_eq!(vec!['b', 'a'], next_key);
    }

    #[test]
    fn next_key_no_unprocessed() {
        let next_key = KanaHandler::next_key(KeyEvent::from_str("k").unwrap().get_symbol_char().unwrap(), &vec![]);
        assert_eq!(vec!['k'], next_key);
    }

    #[test]
    fn can_process_single() {
        let handler = KanaHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec![]);
        assert!(result);
    }

    #[test]
    fn can_process_intermediate() {
        let handler = KanaHandler::test_handler();
        let result = handler.can_process(&KeyEvent::from_str("k").unwrap(), &vec![]);
        assert!(result);
    }

    #[test]
    fn handler_from_string() {
        let content = r#"
        {
            "a": ["", "あ" ],
            "bb": ["b", "っ" ],
            "ba": ["", "ば" ],
            "be": ["", "べ" ]
        }
        "#.to_string();
        let handler = KanaHandler::handler_from_string(&content);
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec![]);
        assert!(result);
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec!['b']);
        assert!(result);
    }

    #[test]
    fn default_handler_works() {
        let handler = KanaHandler::default_handler();

        let result = handler.can_process(&KeyEvent::from_keysym(keysyms::KEY_apostrophe, SkkKeyModifier::None), &vec!['n']);
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("b").unwrap(), &vec![]);
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("y").unwrap(), &vec!['b']);
        assert!(result);

        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec!['b', 'y']);
        assert!(result);
    }

    #[test]
    fn get_instruction() {
        let handler = KanaHandler::default_handler();

        let result = handler.get_instruction(&KeyEvent::from_str("b").unwrap(), &vec![]);
        match result {
            Some(Instruction::FlushUnconverted { new_start: _ }) => {}
            _ => assert!(false)
        }

        let result = handler.get_instruction(&KeyEvent::from_str("y").unwrap(), &vec!['b']);
        match result {
            Some(Instruction::FlushUnconverted { new_start: _ }) => {}
            _ => assert!(false)
        }

        let result = handler.get_instruction(&KeyEvent::from_str("y").unwrap(), &vec!['n']);
        match result {
            Some(Instruction::FlushUnconverted { new_start: _ }) => {}
            _ => assert!(false)
        }


        let result = handler.get_instruction(&KeyEvent::from_str("a").unwrap(), &vec!['b', 'y']);
        match result {
            Some(Instruction::InputKana { converted, carry_over: _ }) => {
                assert_eq!("びゃ", converted);
            }
            _ => assert!(false)
        }
    }
}