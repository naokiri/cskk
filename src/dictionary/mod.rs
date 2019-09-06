use std::sync::Arc;

use crate::dictionary::candidate::Candidate;

pub mod on_memory_dict;
pub mod candidate;

#[derive(Debug)]
pub(crate) struct DictEntry {
    midashi: String,
    candidates: Vec<Arc<Candidate>>,
}


impl DictEntry {
    pub fn get_candidates(&self) -> &Vec<Arc<Candidate>> {
        &self.candidates
    }
}

pub(crate) trait Dictionary {
    fn lookup(&self, midashi: &str, _okuri: bool) -> Option<&DictEntry>;
    fn new() -> Self;
}