use crate::dictionary::dictentry::DictEntry;
use crate::error::CskkError;
use log::*;
use std::fmt::Write;
use std::sync::Arc;

// Blind copy of libskk vala Candidate class
#[derive(Debug, Clone)]
pub struct Candidate {
    pub(crate) midashi: Arc<String>,
    #[allow(dead_code)]
    pub(crate) okuri: bool,
    // Raw kouho_text that might include "#0回" etc
    pub(crate) kouho_text: Arc<String>,
    pub(crate) annotation: Option<Arc<String>>,
    // Output to show the candidate.
    pub(crate) output: String,
}

impl Default for Candidate {
    fn default() -> Self {
        Candidate {
            midashi: Arc::new("エラー".to_owned()),
            okuri: false,
            kouho_text: Arc::new("エラー".to_owned()),
            annotation: None,
            output: "エラー".to_string(),
        }
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        if self.midashi.eq(&other.midashi) && self.kouho_text.eq(&other.kouho_text) {
            return true;
        }
        false
    }
}

impl Candidate {
    pub(crate) fn new(
        midashi: Arc<String>,
        okuri: bool,
        kouho_text: Arc<String>,
        annotation: Option<Arc<String>>,
        output: String,
    ) -> Self {
        Candidate {
            midashi,
            okuri,
            kouho_text,
            annotation,
            output,
        }
    }

    pub(crate) fn from_skk_jisyo_string(midashi: &str, raw_entry: &str) -> Result<Self, CskkError> {
        let mut chunk = raw_entry.split(';');
        if let Some(text) = chunk.next() {
            let kouho = DictEntry::process_lisp_fun(text);
            let annotation = chunk
                .next()
                .map(|entry| Arc::new(DictEntry::process_lisp_fun(entry)));
            Ok(Candidate::new(
                Arc::new(midashi.to_string()),
                false,
                Arc::new(kouho.to_string()),
                annotation,
                kouho.to_string(),
            ))
        } else {
            debug!("Failed to parse candidate from: {:?}", raw_entry);
            Err(CskkError::Error("No candidate".to_string()))
        }
    }

    // entry string between '/'
    // {候補};アノテーション
    // {候補};*アノテーション
    // TODO: 将来的には [{優先送り仮名}/{候補}] のような優先送り仮名エントリも扱えると嬉しい
    pub(crate) fn to_skk_jisyo_string(&self) -> String {
        let mut result = String::new();
        result.push_str(&DictEntry::escape_dictionary_string(
            self.kouho_text.as_str(),
        ));
        if let Some(annotation) = &self.annotation {
            write!(
                result,
                ";{}",
                &DictEntry::escape_dictionary_string(annotation.as_str())
            )
            .expect("Failed to allocate jisyo string for candidate.");
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn skk_jisyo_string_no_annotation() {
        let candidate = Candidate::new(
            Arc::new("みだし".to_string()),
            false,
            Arc::new("候補".to_string()),
            None,
            "候補".to_string(),
        );
        assert_eq!("候補", candidate.to_skk_jisyo_string())
    }
}
