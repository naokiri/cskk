use crate::skk_modes::{CommaStyle, PeriodStyle};

pub(crate) struct CskkConfig {
    pub(crate) auto_start_henkan_keywords: Vec<String>,
    // Easy override only for period and comma for libskk compatibility.
    pub(crate) period_style: PeriodStyle,
    pub(crate) comma_style: CommaStyle,
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
            period_style: PeriodStyle::PeriodJa,
            comma_style: CommaStyle::CommaJa,
        }
    }
}

impl CskkConfig {
    pub(crate) fn set_auto_start_henkan_keywords(&mut self, new_auto_start_henkan: Vec<String>) {
        self.auto_start_henkan_keywords = new_auto_start_henkan
    }

    pub fn set_period_style(&mut self, period_style: PeriodStyle) {
        self.period_style = period_style;
    }

    pub fn set_comma_style(&mut self, comma_style: CommaStyle) {
        self.comma_style = comma_style;
    }
}
