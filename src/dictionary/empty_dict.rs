use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::Dictionary;

///
/// Empty dictionary
///
#[derive(Debug)]
pub struct EmptyDictionary {}

impl Dictionary for EmptyDictionary {
    fn lookup(&self, _midashi: &str, _okuri: bool) -> Option<&DictEntry> {
        None
    }
}
