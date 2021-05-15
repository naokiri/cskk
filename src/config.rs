pub(crate) struct CskkConfig {
    pub(crate) auto_start_henkan_keywords: Vec<String>,
}

impl Default for CskkConfig {
    fn default() -> Self {
        Self {
            auto_start_henkan_keywords: vec![
                "。".to_string(),
                "、".to_string(),
                "」".to_string(),
                "』".to_string(),
            ],
        }
    }
}

impl CskkConfig {
    pub(crate) fn set_auto_start_henkan_keywords(&mut self, new_auto_start_henkan: Vec<String>) {
        self.auto_start_henkan_keywords = new_auto_start_henkan
    }
}
