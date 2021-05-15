pub(crate) struct CskkConfig {
    pub(crate) auto_start_henkan_strings: Vec<String>,
}

impl Default for CskkConfig {
    fn default() -> Self {
        Self {
            auto_start_henkan_strings: vec![
                "。".to_string(),
                "、".to_string(),
                "」".to_string(),
                "』".to_string(),
            ],
        }
    }
}
