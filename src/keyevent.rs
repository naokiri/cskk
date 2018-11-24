use serde::{Deserialize, Deserializer};
use serde::de::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use xkbcommon::xkb;
use xkbcommon::xkb::keysyms;

type KeyEventError = String; // TODO: Make better error structure?

bitmask! {
    ///
    /// modifier mask ported from fcitx-skk and libskk.
    /// Have to keep LShift and RShift distinguishable, and represent no key typing for a while as one of key event for NICOLA (yet unimplemented in cskk)
    ///
    pub mask SkkKeyModifier: u32 where flags SkkKeyModifierFlag {
        None = 0,
        Shift = 1 << 0,
        Lock = 1 << 1,
        Control = 1 << 2,
        Mod1 = 1 << 3,
        Mod2 = 1 << 4,
        Mod3 = 1 << 5,
        Mod4 = 1 << 6,
        Mod5 = 1 << 7,

        /// Reserved for nicola
        LShift = 1 << 22,
        /// Reserved for nicola
        RShift = 1 << 23,
        /// Reserved for nicola
        /// works specially that sleeps (int)keysym usec for simulating non-double key press event.
        USleep = 1 << 24,

        Super = 1 << 26,
        Hyper = 1 << 27,
        Meta = 1 << 28,
        Release = 1 << 30
    }
}

///
/// Just a Vec of KeyEvents, but makes string representation a space separated single string that can be a key for toml table.
///
type KeyEventSeq = Vec<KeyEvent>;

///
/// In-lib structure of key event
///
/// String representation of key event is paren enclosed LongModifiers and single KeyName, or just one ShortModifier and one KeyName joined, or single KeyName.
/// LongModifier := "control" | "meta" | "alt" | "lshift" | "rshift"
/// ShortModifier := "C-" | "A-" | "M-" | "G-" for ctrl, mod1, meta, mod5 respectively
/// KeyName := ↓
/// https://github.com/xkbcommon/libxkbcommon/blob/master/xkbcommon/xkbcommon-keysyms.h
/// https://xkbcommon.org/doc/current/xkbcommon_8h.html#a79e604a22703391bdfe212cfc10ea007
///
/// e.g.
/// "(control a)" "C-a" "M-Left" "l"
///
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct KeyEvent {
    symbol: xkb::Keysym,
    modifiers: SkkKeyModifier,
}

impl KeyEvent {
    pub fn from_keysym(keysym: xkb::Keysym,
                       modifier: SkkKeyModifier) -> KeyEvent {
        KeyEvent { symbol: keysym, modifiers: modifier }
    }

    ///
    /// string representation to KeyEvent.
    /// When parsing fails keysym is likely to be a voidsymbol
    ///
    pub fn from_str(key: &str) -> Result<KeyEvent, KeyEventError> {
        let mut modifier: SkkKeyModifier = SkkKeyModifier::none();
        let mut keysym: xkb::Keysym = keysyms::KEY_VoidSymbol;
        let key = key.trim();
        if key.starts_with("(") & &key.ends_with(")") {
            let mut words = key.trim_left_matches("(").trim_right_matches(")").split(' ');
            loop {
                match words.next() {
                    Some(word) => {
                        match word {
                            "control" => {
                                modifier.set(SkkKeyModifierFlag::Control);
                            }
                            "meta" => {
                                modifier.set(SkkKeyModifierFlag::Meta);
                            }
                            "alt" => {
                                modifier.set(SkkKeyModifierFlag::Mod1);
                            }
                            "lshift" => {
                                modifier.set(SkkKeyModifierFlag::LShift);
                            }
                            "rshift" => {
                                modifier.set(SkkKeyModifierFlag::RShift);
                            }
                            _ => {
                                keysym = xkb::keysym_from_name(word, xkb::KEYSYM_NO_FLAGS);
                            }
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        } else {
            let keyname: &str = if key.len() > 2 {
                match &key[0..2] {
                    "C-" => {
                        modifier.set(SkkKeyModifierFlag::Control);
                        &key[2..]
                    }
                    "M-" => {
                        modifier.set(SkkKeyModifierFlag::Meta);
                        &key[2..]
                    }
                    "A-" => {
                        modifier.set(SkkKeyModifierFlag::Mod1);
                        &key[2..]
                    }
                    "G-" => {
                        modifier.set(SkkKeyModifierFlag::Mod5);
                        &key[2..]
                    }
                    _ => {
                        key
                    }
                }
            } else {
                key
            };
            keysym = xkb::keysym_from_name(keyname, xkb::KEYSYM_NO_FLAGS);
        }

        if keysym == xkb::keysyms::KEY_VoidSymbol {
            Err("No str checked".to_owned())
        } else if keysym == xkb::keysyms::KEY_NoSymbol {
            Err("Not a key symbol: {}".to_owned())
        } else {
            Ok(KeyEvent {
                modifiers: modifier,
                symbol: keysym,
            })
        }
    }


    pub fn deserialize_seq(from: &str) -> Result<KeyEventSeq, KeyEventError> {
        match KeyEvent::deserialize_seq_inner(from, Vec::new()) {
            Ok(result) => {
                Ok(result)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn deserialize_seq_inner(keys: &str, mut current: Vec<KeyEvent>) -> Result<KeyEventSeq, KeyEventError> {
        let keys = keys.trim();
        if keys.len() == 0 {
            return Ok(current);
        }
        match KeyEvent::next_tok(keys) {
            Some(tok) => {
                let left = &keys[tok.len()..];
                match KeyEvent::from_str(tok) {
                    Ok(keyevent) => {
                        current.push(keyevent);
                        KeyEvent::deserialize_seq_inner(left, current)
                    }
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            _ => {
                Err(format!("Syntax error. keys: {}", keys))
            }
        }
    }

    /// '''
    /// let str = "(foo bar) other string"
    /// let result = KeyEventSeq::next_tok(str)
    /// assert_eq!(result, Ok("(foo bar)")
    /// '''
    /// '''
    /// let str = "foo bar baz"
    /// let result = KeyEventSeq::next_tok(str)
    /// assert_eq!(result, Ok("foo")
    /// '''
    fn next_tok(keys: &str) -> Option<&str> {
        if keys.starts_with('(') {
            let len = keys.find(')');
            match len {
                Some(x) => {
                    Some(&keys[0..x + 1])
                }
                _ => {
                    None
                }
            }
        } else {
            let len = keys.find(' ');
            match len {
                Some(x) => {
                    Some(&keys[0..x])
                }
                _ => {
                    Some(keys)
                }
            }
        }
    }
}

impl Display for KeyEvent {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", xkb::keysym_to_utf8(self.symbol).trim_right_matches("\u{0}"))
    }
}

impl<'de> Deserialize<'de> for KeyEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        KeyEvent::from_str(s).map_err(D::Error::custom)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    extern crate env_logger;

    #[test]
    fn keyevent_from_str() {
        let a = KeyEvent::from_str("a").unwrap();
        assert_eq!(a.symbol, keysyms::KEY_a, "equals small a");
        assert_eq!(a.modifiers, SkkKeyModifier::none(), "No modifier for a");

        let spacea = KeyEvent::from_str(" a").unwrap();
        assert_eq!(spacea.symbol, keysyms::KEY_a, "equals small a");
        assert_eq!(spacea.modifiers, SkkKeyModifier::none(), "No modifier for a");

        let b = KeyEvent::from_str("B").unwrap();
        assert_eq!(b.symbol, keysyms::KEY_B, "equals large B");
        assert_eq!(b.modifiers, SkkKeyModifier::none(), "No modifier for B");

        let shift_b = KeyEvent::from_str("(control b)").unwrap();
        let mut control_modifier: SkkKeyModifier = SkkKeyModifier::none();
        control_modifier.set(SkkKeyModifierFlag::Control);
        assert_eq!(shift_b.symbol, keysyms::KEY_b, "equals small b");
        assert_eq!(shift_b.modifiers, control_modifier, "long modifier control");


        let notu = KeyEvent::from_str("LATIN SMALL LETTER U WITH ACUTE");
        assert!(notu.is_err());

        let u = KeyEvent::from_str("uacute").unwrap();
        assert_eq!(u.symbol, keysyms::KEY_uacute, "latin small u acute");

        let short_ctrl_a = KeyEvent::from_str("C-a").unwrap();
        assert_eq!(short_ctrl_a.symbol, keysyms::KEY_a, "C-a works");
        assert_eq!(short_ctrl_a.modifiers, control_modifier, "C-a works");

        let meta_left = KeyEvent::from_str("M-Left").unwrap();
        let mut meta_modifier: SkkKeyModifier = SkkKeyModifier::none();
        meta_modifier.set(SkkKeyModifierFlag::Meta);
        assert_eq!(meta_left.symbol, keysyms::KEY_Left);
        assert_eq!(meta_left.modifiers, meta_modifier);
    }

    #[test]
    fn keyevent_to_string() {
        let a = KeyEvent::from_str("a").unwrap();
        assert_eq!("a", a.to_string());
    }

    #[test]
    fn deserialize_seq() {
        let result = KeyEvent::deserialize_seq("a b c").unwrap();
        assert_eq!(result.get(0).unwrap(), &KeyEvent::from_str("a").unwrap());
        assert_eq!(result.get(1).unwrap(), &KeyEvent::from_str("b").unwrap());
        assert_eq!(result.get(2).unwrap(), &KeyEvent::from_str("c").unwrap());
    }

    #[test]
    fn from_keysym() {
        let mut modifier = SkkKeyModifier::none();
        modifier.set(SkkKeyModifierFlag::LShift);
        let result = KeyEvent::from_keysym(keysyms::KEY_s, modifier);
        assert_eq!(result.symbol, keysyms::KEY_s);
        assert_eq!(result.modifiers, modifier);
    }
}