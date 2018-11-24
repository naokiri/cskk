use sequence_trie::SequenceTrie;

use Instruction;
use keyevent::KeyEvent;

type Converted = String;
type CarryOver = Vec<char>;

pub(crate) struct Handler {
    // Maybe change value to input-kana-* command etc?
    process_map: SequenceTrie<char, (Converted, CarryOver)>
}

impl Handler {
    pub(crate) fn defalt_handler() -> Handler {
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

        Handler {
            process_map: process_list,
        }
    }

    fn next_key(key_char: char, unprocessed: &Vec<char>) -> Vec<char> {
        let mut val = vec![];
        val.extend_from_slice(unprocessed);
        val.push(key_char);
        return val;
    }

    fn get_node(&self, key_event: &KeyEvent, unprocessed: &Vec<char>) -> Option<&SequenceTrie<char, (Converted, CarryOver)>> {
        match key_event.get_symbol_char() {
            None => { None }
            Some(key_char) => {
                self.process_map.get_node(&Handler::next_key(key_char, unprocessed))
            }
        }
    }

    pub(crate) fn can_process(&self, key_event: &KeyEvent, unprocessed: &Vec<char>) -> bool {
        match self.get_node(key_event, unprocessed) {
            None => { false }
            Some(_) => { true }
        }
    }

    pub(crate) fn get_instruction(&self, key_event: &KeyEvent, unprocessed: &Vec<char>) -> Option<Instruction> {
        match self.get_node(key_event, &unprocessed) {
            Some(node) => {
                match node.value() {
                    None => None,
                    Some((converted, carry_over)) => {
                        Some(Instruction::Input { converted, unconverted: carry_over })
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
mod tests {
    use super::*;

    #[test]
    fn next_key_with_unprocessed() {
        let next_key = Handler::next_key(KeyEvent::from_str("a").unwrap().get_symbol_char().expect("no symbol char"), &vec!['b']);
        assert_eq!(vec!['b', 'a'], next_key);
    }

    #[test]
    fn next_key_no_unprocessed() {
        let next_key = Handler::next_key(KeyEvent::from_str("k").unwrap().get_symbol_char().unwrap(), &vec![]);
        assert_eq!(vec!['k'], next_key);
    }

    #[test]
    fn can_process_single() {
        let handler = Handler::defalt_handler();
        let result = handler.can_process(&KeyEvent::from_str("a").unwrap(), &vec![]);
        assert!(result);
    }

    #[test]
    fn can_process_intermediate() {
        let handler = Handler::defalt_handler();
        let result = handler.can_process(&KeyEvent::from_str("k").unwrap(), &vec![]);
        assert!(result);
    }
}