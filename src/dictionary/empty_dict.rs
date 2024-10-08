use crate::dictionary::dictentry::DictEntry;
use crate::dictionary::{CompositeKey, Dictionary};

///
/// Empty dictionary
///
#[derive(Debug, Default)]
pub(crate) struct EmptyDictionary {}

impl Dictionary for EmptyDictionary {
    fn lookup(&self, _composite_key: &CompositeKey) -> Option<&DictEntry> {
        None
    }

    fn complete<'a>(
        &'a self,
        _midashi_head: &CompositeKey,
    ) -> Box<dyn Iterator<Item = &'a DictEntry> + 'a> {
        let a = std::iter::empty();
        Box::new(a)
    }
}
