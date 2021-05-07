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

    /// get selection pointer if possible
    pub(crate) fn get_selection_pointer(&self) -> usize {
        self.selection_pointer
    }

    /// set selection pointer if possible
    pub(crate) fn set_selection_pointer(&mut self, i: usize) -> bool {
        return if self.composition_candidates.get(i).is_some() {
            self.selection_pointer = i;
            true
        } else {
            false
        };
    }

    pub(crate) fn get_current_to_composite(&self) -> &str {
        &self.to_composite
    }

    pub(crate) fn get_all_candidates(&self) -> &Vec<Candidate> {
        &self.composition_candidates
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

    pub(crate) fn is_empty(&self) -> bool {
        self.composition_candidates.is_empty()
    }

    pub(crate) fn has_previous(&self) -> bool {
        self.selection_pointer != 0
    }

    pub(crate) fn has_next(&self) -> bool {
        self.selection_pointer != self.composition_candidates.len() - 1
    }

    pub(crate) fn forward_candidate(&mut self) -> bool {
        // TODO: more than 1 for paging
        if self.selection_pointer < self.composition_candidates.len() - 1 {
            self.selection_pointer += 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn backward_candidate(&mut self) -> bool {
        // TODO: more than 1 for paging
        if self.selection_pointer > 0 {
            self.selection_pointer -= 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn clear(&mut self) {
        self.selection_pointer = 0;
        self.composition_candidates.clear();
        self.to_composite.clear();
    }
}
