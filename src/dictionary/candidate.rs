use std::sync::Arc;
use std::fmt;
use std::fmt::Formatter;
use std::error::Error;

// Blind copy of libskk vala Candidate class
// TODO: Maybe enough with Rc instead of Arc, 置きかえる意味があるかどうか調べる
// TODO: pub(crate) -> pub(in foo)? Want this to be pub in cfg(test) only
#[derive(Debug)]
pub(crate) struct Candidate {
    pub (crate) midashi: Arc<String>,
    #[allow(dead_code)]
    pub (crate) okuri: bool,
    // Raw kouho_text that might include "#0回" etc
    pub (crate) kouho_text: Arc<String>,
    pub (crate) annotation: Option<Arc<String>>,
    // Output to show candidate? Mutable?
    #[allow(dead_code)]
    output: Option<String>,
}

impl Candidate {
    pub fn new(midashi: Arc<String>,
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
}


