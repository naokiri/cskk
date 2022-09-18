use crate::keyevent::CskkKeyEvent;
use crate::rule::CskkRule;
use crate::skk_modes::{CommaStyle, PeriodStyle};
use sequence_trie::SequenceTrie;
use std::collections::HashMap;
use xkbcommon::xkb::keysyms;
#[cfg(test)]
use xkbcommon::xkb::keysyms::{
    KEY_a, KEY_at, KEY_b, KEY_bracketleft, KEY_e, KEY_i, KEY_k, KEY_n, KEY_o, KEY_t, KEY_u, KEY_7,
    KEY_A, KEY_B,
};
use xkbcommon::xkb::Keysym;

pub(crate) type Converted = String;
pub(crate) type CarryOver = Vec<Keysym>;

///
/// 単一あるいは複数のkeysymから文字への変換(convert)を担う部分。主にローマ字からひらがなへの変換に使われるが、ひらがなに限定しない。
/// keysymには[A-Z]が[a-z]とは別に存在するが、[a-z]と同じとみなす。
/// この構造体自体は状態を持たない。
///
/// 例として一般的なローマ字変換の設定では[KEY_k KEY_k]の入力をconvertすると"っ"に変換され、CarryOverとして[KEY_k]が残る。
///
#[derive(Clone, Debug)]
pub(crate) struct KanaBuilder {
    process_map: SequenceTrie<Keysym, (Converted, CarryOver)>,
}

impl KanaBuilder {
    //!
    //! 未決時にもconvertすると確定してしまうので、ddskkのskk-kana-input実装と違う作りになっている。要再検討
    //!

    /// このunprocessedの状態を持っている時にkey_eventを入力した'直後'、実際の変換を行う前のunprocessed状態を返す。
    ///
    /// 例えば一般的なローマ字変換の設定の場合、[Key_l] に Key_kを入力してそのまま続く変換状態は存在しないため、[Key_k]のみが返る。
    /// [Key_k]にKey_aを入力すると[Key_k, Key_a]が返る。この返り値をconvertに通すことで変換後の文字列"な"と変換後の次のunprocessed状態である[] (空ベクタ)を得ることができる。
    /// [Key_n]にKey_yを入力すると"にゃ(nya)"等の変換に続きうるので以前の状態に続いた状態として[Key_n, Key_y]が返るが、convertに渡してもこの時点では変換不能なためNoneが返る。
    /// そもそもハンドラーで対応できないKey_spaceなどが入力されると空ベクタを返す。
    pub(crate) fn next_unprocessed_state(
        &self,
        key_event: &CskkKeyEvent,
        unprocessed: &[Keysym],
    ) -> Vec<Keysym> {
        let mut combined = vec![];
        combined.extend_from_slice(unprocessed);

        combined.push(key_event.get_symbol());
        if self.can_continue(key_event, unprocessed) {
            Self::combine_raw(key_event, unprocessed)
        } else if self.can_continue_lower(key_event, unprocessed) {
            Self::combine_lower(key_event, unprocessed)
        } else if self.can_continue(key_event, &[]) {
            vec![key_event.get_symbol()]
        } else {
            vec![]
        }
    }

    /// convert the unprocessed vector into String and the remaining carryover if matching String exists
    pub(crate) fn convert(&self, kana: &[Keysym]) -> Option<&(Converted, CarryOver)> {
        self.process_map.get(kana)
    }

    ///
    /// Not in the normal convert function because caller should know ",." to treat this specially for composition mode changes.
    ///
    pub(crate) fn convert_periods(
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

    ///
    /// 接続可能かどうか確認せずにunprocessedにkey_eventのKeysymを足したものを返す。
    /// ただし、[A-Z]は[a-z]とみなされる。
    /// 通常はnext_unprocessed_stateで入力の結果を見る。
    ///
    pub(crate) fn combine_lower(key_event: &CskkKeyEvent, unprocessed: &[Keysym]) -> Vec<Keysym> {
        let mut combined = vec![];
        combined.extend_from_slice(unprocessed);
        combined.push(Self::uncapitalize(key_event.get_symbol()));
        combined
    }

    ///
    /// 接続可能かどうか確認せずにunprocessedにkey_eventのKeysymを足したものを返す。
    /// 大文字もそのまま扱う。
    /// 通常はnext_unprocessed_stateで入力の結果を見る。
    ///
    pub(crate) fn combine_raw(key_event: &CskkKeyEvent, unprocessed: &[Keysym]) -> Vec<Keysym> {
        let mut combined = vec![];
        combined.extend_from_slice(unprocessed);
        combined.push(key_event.get_symbol());
        combined
    }

    /// 大文字を対応する小文字に変換する。
    fn uncapitalize(keysym: Keysym) -> Keysym {
        if (keysyms::KEY_A..=keysyms::KEY_Z).contains(&keysym) {
            keysym + 0x0020
        } else {
            keysym
        }
    }

    /// 今のunprocessedに続いて次のkey_eventが来た時にかな変換を続けられるか。
    /// 一般的なローマ字変換での例
    /// k j -> false
    /// t t -> true ('っt' として続けられるため)
    pub(crate) fn can_continue(&self, key_event: &CskkKeyEvent, unprocessed: &[Keysym]) -> bool {
        self.get_node_raw(key_event, unprocessed).is_some()
    }

    /// 今のunprocessedに続いて次のkey_eventが来た時に、それを小文字化すればかな変換を続けられるか。
    /// 一般的なローマ字変換での例
    /// k j -> false
    /// t t -> true ('っt' として続けられるため)
    pub(crate) fn can_continue_lower(
        &self,
        key_event: &CskkKeyEvent,
        unprocessed: &[Keysym],
    ) -> bool {
        self.get_node_lower(key_event, unprocessed).is_some()
    }

    fn get_node_lower(
        &self,
        key_event: &CskkKeyEvent,
        unprocessed: &[Keysym],
    ) -> Option<&SequenceTrie<Keysym, (Converted, CarryOver)>> {
        let key = KanaBuilder::combine_lower(key_event, unprocessed);
        self.process_map.get_node(&key)
    }

    fn get_node_raw(
        &self,
        key_event: &CskkKeyEvent,
        unprocessed: &[Keysym],
    ) -> Option<&SequenceTrie<Keysym, (Converted, CarryOver)>> {
        let key = KanaBuilder::combine_raw(key_event, unprocessed);
        self.process_map.get_node(&key)
    }

    fn converter_from_hashmap(map: &HashMap<String, (String, String)>) -> Self {
        let mut process_map = SequenceTrie::new();
        for (k, (carry, conv)) in map {
            let key = CskkKeyEvent::keysyms_from_str(k);
            let carry_over = CskkKeyEvent::keysyms_from_str(carry);

            let converted = conv.to_owned();

            process_map.insert(&key, (converted, carry_over));
        }
        Self { process_map }
    }

    pub(crate) fn new(rule: &CskkRule) -> Self {
        KanaBuilder::converter_from_hashmap(rule.get_conversion_rule())
    }

    ///
    /// Returns KanaBuilder that can convert nothing.
    ///
    pub(crate) fn new_empty() -> Self {
        Self {
            process_map: SequenceTrie::new(),
        }
    }
}

#[cfg(test)]
impl KanaBuilder {
    pub fn test_converter() -> Self {
        let mut process_list = SequenceTrie::new();

        process_list.insert(&[KEY_a], ("あ".to_string(), vec![]));
        process_list.insert(&[KEY_i], ("い".to_string(), vec![]));
        process_list.insert(&[KEY_u], ("う".to_string(), vec![]));
        process_list.insert(&[KEY_e], ("え".to_string(), vec![]));
        process_list.insert(&[KEY_o], ("お".to_string(), vec![]));

        process_list.insert(&[KEY_k, KEY_a], ("か".to_string(), vec![]));
        process_list.insert(&[KEY_k, KEY_i], ("き".to_string(), vec![]));
        process_list.insert(&[KEY_k, KEY_u], ("く".to_string(), vec![]));
        process_list.insert(&[KEY_k, KEY_e], ("け".to_string(), vec![]));
        process_list.insert(&[KEY_k, KEY_o], ("こ".to_string(), vec![]));

        process_list.insert(
            &[keysyms::KEY_t, keysyms::KEY_s, keysyms::KEY_u],
            ("つ".to_string(), vec![]),
        );

        KanaBuilder {
            process_map: process_list,
        }
    }

    // Example from ddskk 16.2 skk-kana-input
    fn test_ant_converter() -> Self {
        let mut process_list = SequenceTrie::new();

        process_list.insert(&[KEY_a], ("あ".to_string(), vec![]));
        process_list.insert(&[KEY_n], ("ん".to_string(), vec![]));
        process_list.insert(&[KEY_n, KEY_n], ("ん".to_string(), vec![]));
        process_list.insert(&[KEY_n, KEY_n], ("な".to_string(), vec![]));
        process_list.insert(&[KEY_t, KEY_a], ("た".to_string(), vec![]));
        process_list.insert(&[KEY_t, KEY_t], ("っ".to_string(), vec![KEY_t]));

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
        let combined = KanaBuilder::combine_lower(
            &CskkKeyEvent::from_string_representation("a").unwrap(),
            &[KEY_b],
        );
        assert_eq!(vec![KEY_b, KEY_a], combined);
    }

    #[test]
    fn combine_no_unprocessed() {
        let combined = KanaBuilder::combine_lower(
            &CskkKeyEvent::from_string_representation("k").unwrap(),
            &[],
        );
        assert_eq!(vec![KEY_k], combined);
    }

    #[test]
    fn combine_capital() {
        let combined = KanaBuilder::combine_lower(
            &CskkKeyEvent::from_string_representation("B").unwrap(),
            &[],
        );
        assert_eq!(vec![KEY_b], combined);
    }

    #[test]
    fn uncapitalize() {
        // 変換する
        assert_eq!(KEY_a, KanaBuilder::uncapitalize(KEY_A));
        assert_eq!(KEY_b, KanaBuilder::uncapitalize(KEY_B));
        // 変換しない
        assert_eq!(KEY_7, KanaBuilder::uncapitalize(KEY_7));
        // 大文字の境界
        assert_eq!(KEY_at, KanaBuilder::uncapitalize(KEY_at));
        assert_eq!(KEY_bracketleft, KanaBuilder::uncapitalize(KEY_bracketleft));
    }

    #[test]
    fn convert() {
        let converter = KanaBuilder::test_converter();

        let result = converter.convert(&[KEY_k]);
        assert_eq!(result, None);
    }

    #[test]
    fn ant_tree_convert() {
        let converter = KanaBuilder::test_ant_converter();
        let result = converter.convert(&[KEY_t]);
        assert_eq!(result, None);

        let (kana, carry_over) = converter.convert(&[KEY_t, KEY_t]).unwrap();
        assert_eq!("っ", kana);
        assert_eq!(*carry_over, vec![KEY_t])
    }

    #[test]
    fn can_continue() {
        let converter = KanaBuilder::test_converter();
        let unprocessed = vec![];
        let actual = converter.can_continue(
            &CskkKeyEvent::from_string_representation("Q").unwrap(),
            &unprocessed,
        );
        assert!(!actual);
    }

    #[test]
    fn can_continue_2of3letter() {
        let converter = KanaBuilder::test_converter();
        let unprocessed = vec![keysyms::KEY_t];
        let actual = converter.can_continue(
            &CskkKeyEvent::from_string_representation("s").unwrap(),
            &unprocessed,
        );
        assert!(actual);
    }

    #[test]
    fn can_continue_na() {
        let converter = KanaBuilder::test_ant_converter();
        let unprocessed = vec![keysyms::KEY_n];
        let actual = converter.can_continue(
            &CskkKeyEvent::from_string_representation("a").unwrap(),
            &unprocessed,
        );
        assert!(!actual);
    }

    #[test]
    fn can_not_continue() {
        let converter = KanaBuilder::test_ant_converter();
        let unprocessed = vec![];
        let actual = converter.can_continue(
            &CskkKeyEvent::from_string_representation("space").unwrap(),
            &unprocessed,
        );
        assert!(!actual);
    }

    #[test]
    fn next_unprocessed_state() {
        let converter = KanaBuilder::test_ant_converter();
        let unprocessed = vec![];
        let actual = converter.next_unprocessed_state(
            &CskkKeyEvent::from_string_representation("space").unwrap(),
            &unprocessed,
        );
        assert_eq!(actual, Vec::<Keysym>::new());
    }
}
