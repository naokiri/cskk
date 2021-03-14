use std::sync::Arc;

// Blind copy of libskk vala Candidate class
#[derive(Debug)]
pub struct Candidate {
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


