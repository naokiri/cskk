use crate::error::CskkError;
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
    // Output to show candidate? Mutable?
    #[allow(dead_code)]
    output: Option<String>,
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
    pub fn new(
        midashi: Arc<String>,
        okuri: bool,
        kouho_text: Arc<String>,
        annotation: Option<Arc<String>>,
        output: Option<String>,
    ) -> Self {
        Candidate {
            midashi,
            okuri,
            kouho_text,
            annotation,
            output,
        }
    }

    pub fn from_skk_jisyo_string(midashi: &str, entry: &str) -> Result<Self, CskkError> {
        let mut chunk = entry.split(';');
        if let Some(text) = chunk.next() {
            let kouho = text;
            let annotation = chunk.next().map(|entry| Arc::new(entry.to_string()));
            Ok(Candidate::new(
                Arc::new(midashi.to_string()),
                false,
                Arc::new(kouho.to_string()),
                annotation,
                Some(kouho.to_string()),
            ))
        } else {
            Err(CskkError::Error("No candidate".to_string()))
        }
    }

    // entry string between '/'
    // {候補};アノテーション
    // {候補};*アノテーション
    // TODO: 将来的には [{優先送り仮名}/{候補}] のような優先送り仮名エントリも扱えると嬉しい
    pub fn to_skk_jisyo_string(&self) -> String {
        let mut result = "".to_string();
        result.push_str(self.kouho_text.as_str());
        if let Some(annotation) = &self.annotation {
            result.push_str(&format!(";{}", annotation.as_str()));
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
            Some("候補".to_string()),
        );
        assert_eq!("候補", candidate.to_skk_jisyo_string())
    }
}
