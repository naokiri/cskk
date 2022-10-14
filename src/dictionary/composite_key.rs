use crate::KanaFormChanger;

/// 辞書を引くための情報
/// 厳密な送り仮名マッチのため、送り仮名を複数文字含みうる。
#[derive(Clone, Debug)]
pub(crate) struct CompositeKey {
    to_composite: String,
    // When Some(), should never be empty string.
    okuri: Option<String>,
}

impl CompositeKey {
    pub(crate) fn new(to_composite: &str, okuri: Option<String>) -> Self {
        CompositeKey {
            to_composite: to_composite.to_owned(),
            okuri,
        }
    }

    pub(crate) fn get_to_composite(&self) -> &str {
        &self.to_composite
    }

    pub(crate) fn get_okuri(&self) -> &Option<String> {
        &self.okuri
    }

    pub(crate) fn has_okuri(&self) -> bool {
        self.okuri.is_some()
    }

    /// Return the string that should be used in dictionary file's midashi.
    pub(in crate::dictionary) fn get_dict_key(&self) -> String {
        if self.okuri.is_some() {
            // ローマ字ベースではない入力規則に対応するため、送り仮名の最初の文字はひらがなから対応表を引く。
            if let Some(okuri) = KanaFormChanger::kana_to_okuri_prefix(
                &self.okuri.as_ref().unwrap().chars().next().unwrap(),
            )
            //KanaFormChanger::kana_to_okuri_prefix(&self.okuri.unwrap())
            {
                let mut result = self.get_to_composite().to_string();
                result.push(okuri);
                return result;
            }
        }

        self.to_composite.to_owned()
    }

    pub(crate) fn clear(&mut self) {
        self.to_composite.clear();
        self.okuri = None;
    }
}
