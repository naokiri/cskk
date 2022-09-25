use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::Dictionary;
use crate::CskkError;

///
/// Empty dictionary
///
#[derive(Debug, Default)]
pub(crate) struct EmptyDictionary {}

impl Dictionary for EmptyDictionary {
    fn lookup(&self, _midashi: &str, _okuri: bool) -> Option<&DictEntry> {
        None
    }

    fn complement(&self, _midashi: &str) -> Result<Vec<&DictEntry>, CskkError> {
        Ok(Vec::new())
    }
}
