// string that is not processed yet. e.g. Single consonant input in Hiragana mode.
type Unprocessed = String;
// string that is processed in mode
type Converted = String;

type InputConverterError = String;

#[derive(Deserialize)]
struct Rule {
    meta: RuleMeta,
    convert: RuleConvert,
}

#[derive(Deserialize)]
struct RuleConvert {
    command: HashMap<KeyEvent, String>,
    kana: HashMap<KeyEventSeq, (Unprocessed, Converted)>,
}

/// TBD 'Rule' applier
struct InputConverter {
    rule: Rule,
    mode: InputMode,
}

impl InputConverter {
    pub fn convert() {}
    pub fn reset() {}

    ///
    /// Consume one KeyEvent and make the next unconverted string and converted string.
    /// make the next unconverted and converted string, and also change the copositionstate?
    ///
    pub fn process_key_event(
        self,
        key_event: KeyEvent,
        mut unprocessed: &Unprocessed,
    ) -> Result<Instruction, InputConverterError> {
        let convert = &self.rule.convert;

        let command = InputConverter::keyevent_to_command(key_event, convert);

        match command {
            Some(command) => {
                return Ok(Instruction::Operation { operation: command });
            }
            None => { // fall thru
            }
        }

        //        InputConverter::keyevent_to_

        return Err(String.from(""));
    }

    fn keyevent_to_command(key_event: KeyEvent, convert: &RuleConvert) -> Option<Command> {
        match convert.command.get(&key_event) {
            Some(x) => {
                match x.as_ref() {
                    "abort" => Option::from(Command::Abort),
                    _ => {
                        None // Do nothing
                    }
                }
            }
            None => {
                None // Do nothing
            }
        }
    }

    fn convert_keyevent(mut unconverted: Unprocessed, key_event: KeyEvent, convert: &RuleConvert) {
        let unconverted = unconverted + key_event.to_string();
        match convert.kana.get(unconverted) {
            Some(unprocessed, unconverted) => {}
        }
    }

    pub fn create_from_file(filename: &str) -> InputConverter {
        let mut file = File::open(filename).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("file read error");

        return InputConverter::create_from_string(contents);
    }

    fn create_from_string(contents: String) -> InputConverter {
        let config: Rule = toml::from_str(&contents).expect("toml content error");

        InputConverter {
            rule: config,
            mode: InputMode::Ascii,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyevent::KeyEvent;

    #[test]
    fn load_rule() {
        let converter = InputConverter::create_from_file("src/rule/default.toml");
    }

    #[test]
    fn convert_ctrl_g() {
        let converter = InputConverter::create_from_file("src/rule/default.toml");
        let ctrl_g_event = KeyEvent::from_str("C-g").unwrap();
        let str = "".to_string();
        let instruction = converter.process_key_event(ctrl_g_event, &str).unwrap();
        match instruction {
            Instruction::Operation { operation } => {
                assert_eq!(operation, Command::Abort);
            }
            _ => {
                panic!();
            }
        }
    }

    #[test]
    fn convert_a() {
        let converter = InputConverter::create_from_file("src/rule/default.toml");
        let a_event = KeyEvent::from_str("a").unwrap();
        let str = "".to_string();
        let instruction = converter.process_key_event(a_event, &str).unwrap();
        match instruction {
            Instruction::Input {
                converted,
                unconverted,
            } => {
                assert_eq!("あ", converted);
                assert_eq!("", unconverted)
            }
            _ => {
                panic!();
            }
        }
    }
}
