use crate::dictionary::candidate::Candidate;
use crate::error::CskkError;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct CandidateList {
    // 現在保持している選択肢の元
    to_composite: String,
    // 現在のポインタ。
    selection_pointer: usize,
    // 変換中の選択肢
    composition_candidates: Vec<Candidate>,
}

impl CandidateList {
    pub(crate) fn new() -> Self {
        CandidateList {
            to_composite: "".to_string(),
            selection_pointer: 0,
            composition_candidates: vec![],
        }
    }

    pub(crate) fn set(&mut self, raw_to_composite: String, candidates: Vec<Candidate>) {
        self.to_composite = raw_to_composite;
        self.composition_candidates = candidates;
        self.selection_pointer = 0;
    }

    pub(crate) fn get_current_to_composite(&self) -> &str {
        &self.to_composite
    }

    pub(crate) fn get_current_candidate(&self) -> Result<&Candidate, CskkError> {
        let candidate = self.composition_candidates.get(self.selection_pointer);
        if let Some(candidate) = candidate {
            return Ok(candidate);
        }
        Err(CskkError::Error(
            "Failed to get current candidate. リスト外の候補を読もうとしました。".to_string(),
        ))
    }

    pub(crate) fn set_new_candidate(&mut self, kouho_text: &str, okuri: bool) {
        let candidate = Candidate::new(
            Arc::new(self.to_composite.to_owned()),
            okuri,
            Arc::new(kouho_text.to_string()),
            None,
            None,
        );
        self.composition_candidates.push(candidate);
        self.selection_pointer = self.composition_candidates.len() - 1;
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn forward_candidate(&mut self) -> Result<bool, CskkError> {
        // TODO: more than 1 for paging
        // TODO: boundary check
        self.selection_pointer += 1;
        Ok(true)
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn backward_candidate(&mut self) -> Result<bool, CskkError> {
        // TODO: more than 1 for paging
        // TODO: boundary check
        self.selection_pointer -= 1;
        Ok(true)
    }

    pub(crate) fn reset(&mut self) {
        // Probably doesn't harm to store to_composite and list.
        self.selection_pointer = 0;
    }
}
