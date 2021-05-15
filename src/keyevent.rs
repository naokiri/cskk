use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

use serde::de::Error;
use serde::{Deserialize, Deserializer};
use xkbcommon::xkb;
use xkbcommon::xkb::keysyms;
// Hidden by bitflags macro
use crate::error::CskkError;
#[allow(unused_imports)]
use std::ops::BitAndAssign;

bitflags! {
    ///
    /// modifier mask ported from fcitx and libskk.
    /// Have to keep LShift and RShift distinguishable, and represent no key typing for a while as one of key event for NICOLA (yet unimplemented in cskk)
    ///
    pub(crate) struct SkkKeyModifier: u32 {
        const NONE = 0;
        const SHIFT = 1;
        const CAPS_LOCK = 1 << 1;
        const CONTROL = 1 << 2;
        const MOD1 = 1 << 3;
        const ALT = 1 << 3; // == MOD1
        const MOD2 = 1 << 4;
        const NUM_LOCK = 1 << 4; // == MOD2
        const MOD3 = 1 << 5;
        const HYPER = 1 << 5; // == MOD3
        const MOD4 = 1 << 6;
        const SUPER = 1 << 6; // == MOD4
        const MOD5 = 1 << 7;
        const MOUSE_PRESSED = 1 << 8;

        /// Reserved for nicola
        const L_SHIFT = 1 << 22;
        /// Reserved for nicola
        const R_SHIFT = 1 << 23;
        /// Reserved for nicola
        /// works specially that sleeps (int)keysym usec for simulating non-double key press event.
        const USLEEP = 1 << 24;

        const SUPER2 = 1 << 26;
        const HYPER2 = 1 << 27;
        const META = 1 << 28;

        const RELEASE = 1 << 30;

        const REPEAT = 1 << 31;

        /// Mask for bits that can be actually given from the fcitx ime
        const NON_DUMMY_MASK = Self::SHIFT.bits | Self::CAPS_LOCK.bits | Self::CONTROL.bits | Self::ALT.bits
        | Self::NUM_LOCK.bits | Self::HYPER.bits | Self::SUPER.bits | Self::MOUSE_PRESSED.bits | Self::SUPER2.bits | Self::HYPER2.bits
        | Self::META.bits | Self::REPEAT.bits;
    }
}

pub type KeyEventSeq = Vec<CskkKeyEvent>;

///
/// In-lib structure of key event
///
/// String representation of key event is paren enclosed LongModifiers and single KeyName, or just one ShortModifier and one KeyName joined, or single KeyName.
/// LongModifier := "control" | "meta" | "alt" | "lshift" | "rshift"
/// ShortModifier := "C-" | "A-" | "M-" | "G-" for ctrl, mod1, meta, mod5 respectively
/// KeyName := ↓
/// https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-keysyms.h
/// https://xkbcommon.org/doc/current/xkbcommon_8h.html#a79e604a22703391bdfe212cfc10ea007
///
/// e.g.
/// "(control a)" "C-a" "M-Left" "l" "space"
///
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct CskkKeyEvent {
    symbol: xkb::Keysym,
    modifiers: SkkKeyModifier,
}

impl CskkKeyEvent {
    #[cfg(test)]
    pub(crate) fn from_keysym(keysym: xkb::Keysym, modifier: SkkKeyModifier) -> Self {
        Self {
            symbol: keysym,
            modifiers: modifier,
        }
    }

    #[cfg(feature = "capi")]
    pub(crate) fn from_fcitx_keyevent(keysym: u32, raw_modifier: u32, is_release: bool) -> Self {
        let mut modifiers: SkkKeyModifier = SkkKeyModifier::from_bits_truncate(raw_modifier);
        modifiers.bitand_assign(SkkKeyModifier::NON_DUMMY_MASK);
        if is_release {
            modifiers.set(SkkKeyModifier::RELEASE, true);
        }
        Self {
            symbol: keysym,
            modifiers,
        }
    }

    /// wrapper of keysym_from_name to pretend some words as a known key name.
    fn keysym_from_name(word: &str) -> xkb::Keysym {
        match word {
            "." => xkb::keysym_from_name(&"period", xkb::KEYSYM_NO_FLAGS),
            "-" => xkb::keysym_from_name(&"minus", xkb::KEYSYM_NO_FLAGS),
            _ => xkb::keysym_from_name(word, xkb::KEYSYM_NO_FLAGS),
        }
    }

    ///
    /// いわゆるAsciiの範囲で表示できる文字
    ///
    pub fn is_ascii_inputtable(&self) -> bool {
        //　ueno/libskkに倣っているが、Latin 1 全部に拡張可能？
        xkb::keysyms::KEY_space <= self.symbol && self.symbol <= xkb::keysyms::KEY_asciitilde
    }

    ///
    /// string representation to KeyEvent.
    /// When parsing fails keysym is likely to be a voidsymbol
    ///
    pub fn from_str(key: &str) -> Result<CskkKeyEvent, CskkError> {
        let mut modifier: SkkKeyModifier = SkkKeyModifier::NONE;
        let mut keysym: xkb::Keysym = keysyms::KEY_VoidSymbol;
        let key = key.trim();
        if key.starts_with('(') && key.ends_with(')') {
            let words = key.trim_start_matches('(').trim_end_matches(')').split(' ');
            for word in words {
                match word {
                    "control" => {
                        modifier.set(SkkKeyModifier::CONTROL, true);
                    }
                    "meta" => {
                        modifier.set(SkkKeyModifier::META, true);
                    }
                    "alt" => {
                        modifier.set(SkkKeyModifier::MOD1, true);
                    }
                    "lshift" => {
                        modifier.set(SkkKeyModifier::L_SHIFT, true);
                    }
                    "rshift" => {
                        modifier.set(SkkKeyModifier::R_SHIFT, true);
                    }
                    _ => {
                        keysym = CskkKeyEvent::keysym_from_name(word);
                    }
                }
            }
        } else {
            let keyname: &str = if key.len() > 2 {
                match &key[0..2] {
                    "C-" => {
                        modifier.set(SkkKeyModifier::CONTROL, true);
                        &key[2..]
                    }
                    "M-" => {
                        modifier.set(SkkKeyModifier::META, true);
                        &key[2..]
                    }
                    "A-" => {
                        modifier.set(SkkKeyModifier::MOD1, true);
                        &key[2..]
                    }
                    "G-" => {
                        modifier.set(SkkKeyModifier::MOD5, true);
                        &key[2..]
                    }
                    _ => key,
                }
            } else {
                key
            };
            keysym = CskkKeyEvent::keysym_from_name(keyname);
        }

        if keysym == xkb::keysyms::KEY_VoidSymbol {
            Err(CskkError::Error("No str checked".to_owned()))
        } else if keysym == xkb::keysyms::KEY_NoSymbol {
            Err(CskkError::Error("Not a key symbol: {}".to_owned()))
        } else {
            Ok(CskkKeyEvent {
                modifiers: modifier,
                symbol: keysym,
            })
        }
    }

    pub fn get_symbol_char(&self) -> Option<char> {
        xkb::keysym_to_utf8(self.symbol).chars().next()
    }

    pub(crate) fn get_modifier(&self) -> SkkKeyModifier {
        self.modifiers
    }

    pub(crate) fn get_symbol(&self) -> xkb::Keysym {
        self.symbol
    }

    pub fn deserialize_seq(from: &str) -> Result<KeyEventSeq, CskkError> {
        match CskkKeyEvent::deserialize_seq_inner(from, Vec::new()) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    fn deserialize_seq_inner(
        keys: &str,
        mut current: Vec<CskkKeyEvent>,
    ) -> Result<KeyEventSeq, CskkError> {
        let keys = keys.trim();
        if keys.is_empty() {
            return Ok(current);
        }
        match CskkKeyEvent::next_tok(keys) {
            Some(tok) => {
                let left = &keys[tok.len()..];
                match CskkKeyEvent::from_str(tok) {
                    Ok(keyevent) => {
                        current.push(keyevent);
                        CskkKeyEvent::deserialize_seq_inner(left, current)
                    }
                    Err(e) => Err(e),
                }
            }
            _ => Err(CskkError::Error(format!("Syntax error. keys: {}", keys))),
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
            len.map(|x| &keys[0..=x])
        } else {
            let len = keys.find(' ');
            match len {
                Some(x) => Some(&keys[0..x]),
                _ => Some(keys),
            }
        }
    }
}

impl Display for CskkKeyEvent {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            xkb::keysym_to_utf8(self.symbol).trim_end_matches('\u{0}')
        )
    }
}

impl<'de> Deserialize<'de> for CskkKeyEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        CskkKeyEvent::from_str(s).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyevent_from_str() {
        let a = CskkKeyEvent::from_str("a").unwrap();
        assert_eq!(a.symbol, keysyms::KEY_a, "equals small a");
        assert_eq!(a.modifiers, SkkKeyModifier::NONE, "No modifier for a");

        let spacea = CskkKeyEvent::from_str(" a").unwrap();
        assert_eq!(spacea.symbol, keysyms::KEY_a, "equals small a");
        assert_eq!(spacea.modifiers, SkkKeyModifier::NONE, "No modifier for a");

        let b = CskkKeyEvent::from_str("B").unwrap();
        assert_eq!(b.symbol, keysyms::KEY_B, "equals large B");
        assert_eq!(b.modifiers, SkkKeyModifier::NONE, "No modifier for B");

        let control_b = CskkKeyEvent::from_str("(control b)").unwrap();
        let control_modifier: SkkKeyModifier = SkkKeyModifier::CONTROL;
        assert_eq!(control_b.symbol, keysyms::KEY_b, "equals small b");
        assert_eq!(
            control_b.modifiers, control_modifier,
            "long modifier control"
        );

        let not_u = CskkKeyEvent::from_str("LATIN SMALL LETTER U WITH ACUTE");
        assert!(not_u.is_err());

        let u = CskkKeyEvent::from_str("uacute").unwrap();
        assert_eq!(u.symbol, keysyms::KEY_uacute, "latin small u acute");

        let short_ctrl_a = CskkKeyEvent::from_str("C-a").unwrap();
        assert_eq!(short_ctrl_a.symbol, keysyms::KEY_a, "C-a works");
        assert_eq!(short_ctrl_a.modifiers, control_modifier, "C-a works");

        let meta_left = CskkKeyEvent::from_str("M-Left").unwrap();
        let meta_modifier: SkkKeyModifier = SkkKeyModifier::META;
        assert_eq!(meta_left.symbol, keysyms::KEY_Left);
        assert_eq!(meta_left.modifiers, meta_modifier);

        let space = CskkKeyEvent::from_str("space").unwrap();
        assert_eq!(space.symbol, keysyms::KEY_space);

        let enter = CskkKeyEvent::from_str("Return").unwrap();
        assert_eq!(enter.symbol, keysyms::KEY_Return);

        let period = CskkKeyEvent::from_str(".").unwrap();
        assert_eq!(period.symbol, keysyms::KEY_period);
    }

    #[test]
    fn keyevent_to_string() {
        let a = CskkKeyEvent::from_str("a").unwrap();
        assert_eq!("a", a.to_string());
    }

    #[test]
    fn deserialize_seq() {
        let result = CskkKeyEvent::deserialize_seq("a b c").unwrap();
        assert_eq!(
            result.get(0).unwrap(),
            &CskkKeyEvent::from_str("a").unwrap()
        );
        assert_eq!(
            result.get(1).unwrap(),
            &CskkKeyEvent::from_str("b").unwrap()
        );
        assert_eq!(
            result.get(2).unwrap(),
            &CskkKeyEvent::from_str("c").unwrap()
        );
    }

    #[test]
    fn from_keysym() {
        let modifier = SkkKeyModifier::L_SHIFT;
        let result = CskkKeyEvent::from_keysym(keysyms::KEY_s, modifier);
        assert_eq!(result.symbol, keysyms::KEY_s);
        assert_eq!(result.modifiers, modifier);
    }

    #[test]
    fn get_symbol_char() {
        let key_event = CskkKeyEvent::from_keysym(keysyms::KEY_0, SkkKeyModifier::NONE);
        assert_eq!('0', key_event.get_symbol_char().unwrap());

        let key_event = CskkKeyEvent::from_keysym(keysyms::KEY_C, SkkKeyModifier::NONE);
        assert_eq!('C', key_event.get_symbol_char().unwrap());

        // エラーにならず、1byte目を返してしまう。
        let key_event = CskkKeyEvent::from_keysym(keysyms::KEY_BackSpace, SkkKeyModifier::NONE);
        assert_eq!('\u{8}', key_event.get_symbol_char().unwrap());
    }

    #[test]
    fn get_symbol_char_no_display() {
        let key_event = CskkKeyEvent::from_keysym(keysyms::KEY_Home, SkkKeyModifier::NONE);
        assert_eq!(None, key_event.get_symbol_char());
    }
}
