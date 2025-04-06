use crate::error::CskkError;
use serde_with::DeserializeFromStr;
use std::fmt;
use std::fmt::Formatter;
use std::fmt::{Debug, Display};
#[allow(unused_imports)]
use std::ops::BitAndAssign;
use std::str::FromStr;
use xkbcommon::xkb;
use xkbcommon::xkb::{keysym_from_name, keysym_get_name, keysyms, Keysym};

bitflags! {
    ///
    /// modifier mask ported from fcitx and libskk.
    ///
    /// Most of modifieres are just reserved yet and cannot set properly.
    ///
    /// Have to keep LShift and RShift distinguishable, and represent no key typing for a while as one of key event for NICOLA (yet unimplemented in cskk)
    ///
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

        /// Mask for bits that can be actually given from ime
        const NON_DUMMY_MASK = Self::SHIFT.bits() | Self::CAPS_LOCK.bits() | Self::CONTROL.bits() | Self::ALT.bits()
        | Self::NUM_LOCK.bits() | Self::HYPER.bits() | Self::SUPER.bits() | Self::MOUSE_PRESSED.bits() | Self::SUPER2.bits() | Self::HYPER2.bits()
        | Self::META.bits() | Self::REPEAT.bits();
    }
}

pub type KeyEventSeq = Vec<CskkKeyEvent>;

///
/// In-lib structure of key event
///
///
///
/// String representation of key event is paren enclosed LongModifiers and single KeyName, or just one ShortModifier and one KeyName joined, or single KeyName.
/// LongModifier := "control" | "meta" | "alt" | "lshift" | "rshift" | "shift"
/// ShortModifier := "C-" | "A-" | "M-" | "G-" for ctrl, mod1, meta, mod5 respectively
/// KeyName := ↓
/// https://github.com/xkbcommon/libxkbcommon/blob/master/include/xkbcommon/xkbcommon-keysyms.h
/// https://xkbcommon.org/doc/current/xkbcommon_8h.html#a79e604a22703391bdfe212cfc10ea007
///
/// e.g.
/// "(control a)" "C-a" "M-Left" "l" "space"
///
#[derive(Clone, Hash, PartialEq, Eq, DeserializeFromStr)]
pub struct CskkKeyEvent {
    symbol: xkb::Keysym,
    modifiers: SkkKeyModifier,
}

impl CskkKeyEvent {
    #[cfg(test)]
    pub(crate) fn from_keysym_strict(keysym: xkb::Keysym, modifier: SkkKeyModifier) -> Self {
        Self {
            symbol: keysym,
            modifiers: modifier,
        }
    }

    ///
    /// Make KeyEvent from fcitx5's keyevent.
    /// Expecting rawKey for input not to lose lowercase/uppercase difference.
    ///
    #[cfg(feature = "capi")]
    pub(crate) fn from_fcitx_keyevent(keysym: u32, raw_modifier: u32, is_release: bool) -> Self {
        let mut modifiers: SkkKeyModifier = SkkKeyModifier::from_bits_truncate(raw_modifier);
        modifiers.bitand_assign(SkkKeyModifier::NON_DUMMY_MASK);
        if is_release {
            modifiers.set(SkkKeyModifier::RELEASE, true);
        }

        // Somehow, Tab -> Tab but Shift+Tab -> Shift + ISO_Left_Tab in raw keyevent.
        // Normalize it.
        let keysym = if keysym == keysyms::KEY_ISO_Left_Tab {
            keysyms::KEY_Tab
        } else {
            keysym
        };

        Self {
            symbol: Keysym::from(keysym),
            modifiers,
        }
    }

    ///
    /// Get a keyevent used in cskk.
    ///
    /// TODO: Switch to another interface that accepts ModMap of xkb instead of each bool when xkbcommon lib can handle them better.
    ///
    pub fn from_keysym_with_flags(
        keysym: xkb::Keysym,
        ctrl_mod: bool,
        shift_mod: bool,
        alt_mod: bool,
    ) -> Self {
        let mut modifiers = SkkKeyModifier::NONE;
        modifiers.set(SkkKeyModifier::CONTROL, ctrl_mod);
        modifiers.set(SkkKeyModifier::SHIFT, shift_mod);
        modifiers.set(SkkKeyModifier::ALT, alt_mod);

        Self {
            symbol: keysym,
            modifiers,
        }
    }

    ///
    /// space区切りで各wordがKeysymであればそのKeysymとして、そうでなければ各バイトをasciiとみなした文字のKeysymとして変換する。
    /// 変換できなかったものは無視される。
    ///
    /// "space 無視" -> KEY_space
    /// "lk Shift_L" -> KEY_l, KEY_k, KEY_Shift_L
    /// "ab" -> KEY_a, KEY_b
    /// "at" -> KEY_at
    ///
    /// "Up", "at", "mu", "ae", "oe", "ht", "ff", "cr", "lf", "nl", "IO" 等短い名称のKeysymは対応するKeysym扱いされるので、2つのKeysym扱いしたい場合はスペースを間に入れる必要がある。
    ///
    pub(crate) fn keysyms_from_str(string: &str) -> Vec<xkb::Keysym> {
        let mut result = vec![];
        let words = string.split(' ');
        for word in words {
            let word = word.trim();
            let word_keysym = keysym_from_name(word, xkb::KEYSYM_NO_FLAGS);
            if Keysym::NoSymbol == word_keysym {
                for char in word.chars() {
                    let char_keysym = keysym_from_name(&char.to_string(), xkb::KEYSYM_NO_FLAGS);
                    if Keysym::NoSymbol != char_keysym {
                        result.push(char_keysym);
                    }
                }
            } else {
                result.push(word_keysym);
            }
        }
        result
    }

    ///
    /// いわゆるAsciiの範囲で表示できる文字
    ///
    pub(crate) fn is_ascii_inputtable(&self) -> bool {
        //　ueno/libskkに倣っているが、Latin 1 全部に拡張可能？
        match self.symbol.raw() {
            keysyms::KEY_space..=keysyms::KEY_asciitilde => true,
            _ => false,
        }
    }

    ///
    /// いわゆるAsciiの大文字。
    ///
    pub(crate) fn is_upper(&self) -> bool {
        match self.symbol.raw() {
            keysyms::KEY_A..=keysyms::KEY_Z => true,
            _ => false,
        }
    }

    ///
    /// Given that this is uppercase letter,
    /// Return a new keyEvent with lowercase letter.
    ///
    pub(crate) fn to_lower(&self) -> Self {
        let mut retval = self.clone();
        if self.is_upper() {
            let lower = Keysym::from(self.symbol.raw() + 0x0020);
            retval.symbol = lower;
        }

        retval
    }

    /// 文字入力のために使えるキーイベントならば true
    // ueno/libskkでは完全にモディファイア無しのキーのみかな変換に使っているが、
    // どうもSHIFTを許容しないといけなさそうなのでSHIFT付きキー入力も明らかにコマンドではない文字入力として扱う。
    pub(crate) fn is_modifierless_input(&self) -> bool {
        self.modifiers.difference(SkkKeyModifier::SHIFT).is_empty()
    }

    ///
    /// string representation to KeyEvent.
    /// When parsing fails keysym is likely to be a voidsymbol
    ///
    /// Testing purpose and not intended to be used from library users. May delete this interface at any update.
    /// Use `from_keysym_with_flags` for now instead.
    ///
    pub fn from_string_representation(key: &str) -> Result<CskkKeyEvent, CskkError> {
        Self::from_str(key)
    }

    pub(crate) fn get_symbol_char(&self) -> Option<char> {
        xkb::keysym_to_utf8(self.symbol).chars().next()
    }

    pub(crate) fn get_modifier(&self) -> SkkKeyModifier {
        self.modifiers
    }

    pub(crate) fn get_modifier_mut(&mut self) -> &mut SkkKeyModifier {
        &mut self.modifiers
    }

    pub(crate) fn get_symbol(&self) -> xkb::Keysym {
        self.symbol
    }

    ///
    /// Mostly testing purpose. May delete this interface at any update.
    /// Use `from_keysym_with_flags` for now instead.
    ///
    pub fn deserialize_seq(from: &str) -> Result<KeyEventSeq, CskkError> {
        CskkKeyEvent::deserialize_seq_inner(from, Vec::new())
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
                match CskkKeyEvent::from_string_representation(tok) {
                    Ok(keyevent) => {
                        current.push(keyevent);
                        CskkKeyEvent::deserialize_seq_inner(left, current)
                    }
                    Err(e) => Err(e),
                }
            }
            _ => Err(CskkError::Error(format!("Syntax error. keys: {keys}"))),
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

impl FromStr for CskkKeyEvent {
    type Err = CskkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut modifier: SkkKeyModifier = SkkKeyModifier::NONE;
        let mut keysym: xkb::Keysym = Keysym::VoidSymbol;
        let key = s.trim();
        // parenで囲われているものはスペース区切りのModifierとkeysymのみ認める。
        // ddskkやlibskkテストケースをそのまま使うための措置。
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
                        modifier.set(SkkKeyModifier::SHIFT, true);
                    }
                    "rshift" => {
                        modifier.set(SkkKeyModifier::R_SHIFT, true);
                        modifier.set(SkkKeyModifier::SHIFT, true);
                    }
                    "shift" => {
                        modifier.set(SkkKeyModifier::SHIFT, true);
                    }
                    _ => {
                        keysym = keysym_from_name(word, xkb::KEYSYM_NO_FLAGS);
                    }
                }
            }
        } else {
            // 簡易な表記として[CMAG]- 接頭辞を修飾子として認める。
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
            keysym = keysym_from_name(keyname, xkb::KEYSYM_NO_FLAGS);
        }

        if keysym == Keysym::VoidSymbol {
            Err(CskkError::ParseError("No str checked".to_owned()))
        } else if keysym == Keysym::NoSymbol {
            Err(CskkError::ParseError(format!("Not a key symbol: {s}")))
        } else {
            Ok(CskkKeyEvent {
                modifiers: modifier,
                symbol: keysym,
            })
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

impl Debug for CskkKeyEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = keysym_get_name(self.symbol);
        f.debug_struct("CskkKeyEvent")
            .field("symbol", &self.symbol)
            .field("key_name", &name)
            .field("modifiers", &self.modifiers)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyevent_from_str() {
        let a = CskkKeyEvent::from_string_representation("a").unwrap();
        assert_eq!(a.symbol, Keysym::from(keysyms::KEY_a), "equals small a");
        assert_eq!(a.modifiers, SkkKeyModifier::NONE, "No modifier for a");

        let spacea = CskkKeyEvent::from_string_representation(" a").unwrap();
        assert_eq!(
            spacea.symbol,
            Keysym::from(keysyms::KEY_a),
            "equals small a"
        );
        assert_eq!(spacea.modifiers, SkkKeyModifier::NONE, "No modifier for a");

        let b = CskkKeyEvent::from_string_representation("B").unwrap();
        assert_eq!(b.symbol, Keysym::from(keysyms::KEY_B), "equals large B");
        assert_eq!(b.modifiers, SkkKeyModifier::NONE, "No modifier for B");

        let control_b = CskkKeyEvent::from_string_representation("(control b)").unwrap();
        let control_modifier: SkkKeyModifier = SkkKeyModifier::CONTROL;
        assert_eq!(
            control_b.symbol,
            Keysym::from(keysyms::KEY_b),
            "equals small b"
        );
        assert_eq!(
            control_b.modifiers, control_modifier,
            "long modifier control"
        );

        let not_u = CskkKeyEvent::from_string_representation("LATIN SMALL LETTER U WITH ACUTE");
        assert!(not_u.is_err());

        let u = CskkKeyEvent::from_string_representation("uacute").unwrap();
        assert_eq!(
            u.symbol,
            Keysym::from(keysyms::KEY_uacute),
            "latin small u acute"
        );

        let short_ctrl_a = CskkKeyEvent::from_string_representation("C-a").unwrap();
        assert_eq!(
            short_ctrl_a.symbol,
            Keysym::from(keysyms::KEY_a),
            "C-a works"
        );
        assert_eq!(short_ctrl_a.modifiers, control_modifier, "C-a works");

        let meta_left = CskkKeyEvent::from_string_representation("M-Left").unwrap();
        let meta_modifier: SkkKeyModifier = SkkKeyModifier::META;
        assert_eq!(meta_left.symbol, Keysym::from(keysyms::KEY_Left));
        assert_eq!(meta_left.modifiers, meta_modifier);

        let space = CskkKeyEvent::from_string_representation("space").unwrap();
        assert_eq!(space.symbol, Keysym::from(keysyms::KEY_space));

        let enter = CskkKeyEvent::from_string_representation("Return").unwrap();
        assert_eq!(enter.symbol, Keysym::from(keysyms::KEY_Return));

        let period = CskkKeyEvent::from_string_representation("period").unwrap();
        assert_eq!(period.symbol, Keysym::from(keysyms::KEY_period));
    }

    #[test]
    fn shift_tab() {
        let result = CskkKeyEvent::from_string_representation("(shift Tab)").unwrap();
        assert_eq!(
            result.symbol,
            Keysym::from(keysyms::KEY_Tab),
            "equals small a"
        );
        assert_eq!(result.modifiers, SkkKeyModifier::SHIFT, "No modifier for a");
    }

    #[test]
    fn keyevent_to_string() {
        let a = CskkKeyEvent::from_string_representation("a").unwrap();
        assert_eq!("a", a.to_string());
    }

    #[test]
    fn deserialize_seq() {
        let result = CskkKeyEvent::deserialize_seq("a b c").unwrap();
        assert_eq!(
            result.get(0).unwrap(),
            &CskkKeyEvent::from_string_representation("a").unwrap()
        );
        assert_eq!(
            result.get(1).unwrap(),
            &CskkKeyEvent::from_string_representation("b").unwrap()
        );
        assert_eq!(
            result.get(2).unwrap(),
            &CskkKeyEvent::from_string_representation("c").unwrap()
        );
    }

    #[test]
    fn from_keysym() {
        let modifier = SkkKeyModifier::L_SHIFT;
        let result = CskkKeyEvent::from_keysym_strict(Keysym::from(keysyms::KEY_s), modifier);
        assert_eq!(result.symbol, Keysym::from(keysyms::KEY_s));
        assert_eq!(result.modifiers, modifier);
    }

    #[test]
    fn get_symbol_char() {
        let key_event =
            CskkKeyEvent::from_keysym_strict(Keysym::from(keysyms::KEY_0), SkkKeyModifier::NONE);
        assert_eq!('0', key_event.get_symbol_char().unwrap());

        let key_event =
            CskkKeyEvent::from_keysym_strict(Keysym::from(keysyms::KEY_C), SkkKeyModifier::NONE);
        assert_eq!('C', key_event.get_symbol_char().unwrap());

        // エラーにならず、1byte目を返してしまう。
        let key_event = CskkKeyEvent::from_keysym_strict(
            Keysym::from(keysyms::KEY_BackSpace),
            SkkKeyModifier::NONE,
        );
        assert_eq!('\u{8}', key_event.get_symbol_char().unwrap());
    }

    #[test]
    fn get_symbol_char_no_display() {
        let key_event =
            CskkKeyEvent::from_keysym_strict(Keysym::from(keysyms::KEY_Home), SkkKeyModifier::NONE);
        assert_eq!(None, key_event.get_symbol_char());
    }

    #[test]
    fn keysyms_from_string() {
        assert_eq!(
            vec![Keysym::from(keysyms::KEY_space)],
            CskkKeyEvent::keysyms_from_str("space 無視")
        );
        assert_eq!(
            vec![
                Keysym::from(keysyms::KEY_l),
                Keysym::from(keysyms::KEY_k),
                Keysym::from(keysyms::KEY_Shift_L)
            ],
            CskkKeyEvent::keysyms_from_str("lk Shift_L")
        );
        assert_eq!(
            vec![Keysym::from(keysyms::KEY_a), Keysym::from(keysyms::KEY_b)],
            CskkKeyEvent::keysyms_from_str("ab")
        );
        assert_eq!(
            vec![Keysym::from(keysyms::KEY_at)],
            CskkKeyEvent::keysyms_from_str("at")
        );
        assert_eq!(
            vec![Keysym::from(keysyms::KEY_question)],
            CskkKeyEvent::keysyms_from_str("question")
        );
    }

    #[test]
    fn is_modifierless() {
        let key_event = CskkKeyEvent::from_str("C-c").unwrap();
        assert!(!key_event.is_modifierless_input())
    }
}
