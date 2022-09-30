use crate::dictionary::dictentry::DictEntry;
use crate::KanaFormChanger;

#[derive(Clone, Debug)]
pub(crate) struct CompositeKey {
    to_composite: String,
    okuri: Option<char>,
}

impl CompositeKey {
    pub(crate) fn new(to_composite: &str, okuri: Option<char>) -> Self {
        CompositeKey {
            to_composite: to_composite.to_owned(),
            okuri,
        }
    }

    pub(crate) fn get_to_composite(&self) -> &str {
        &self.to_composite
    }

    pub(crate) fn get_okuri(&self) -> &Option<char> {
        &self.okuri
    }

    pub(crate) fn has_okuri(&self) -> bool {
        self.okuri.is_some()
    }

    /// Return the string that should be used in dictionary file's midashi.
    pub(in crate::dictionary) fn get_dict_key(&self) -> String {
        if self.okuri.is_some() {
            // ローマ字ベースではない入力規則に対応するため、送り仮名の最初の文字はひらがなから対応表を引く。
            if let Some(okuri) = KanaFormChanger::kana_to_okuri_prefix(&self.okuri.unwrap()) {
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
