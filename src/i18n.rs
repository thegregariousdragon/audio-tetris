use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Language {
    #[default]
    EnUs,
    EnGb,
    EsEs,
    EsLa,
    FrFr,
    FrCa,
    ItIt,
    DeDe,
    ZhCn,
    ZhTw,
    JaJp,
    KoKr,
}

impl Language {
    pub const ALL: &'static [Language] = &[
        Language::EnUs,
        Language::EnGb,
        Language::EsEs,
        Language::EsLa,
        Language::FrFr,
        Language::FrCa,
        Language::ItIt,
        Language::DeDe,
        Language::ZhCn,
        Language::ZhTw,
        Language::JaJp,
        Language::KoKr,
    ];

    pub fn code(&self) -> &'static str {
        match self {
            Language::EnUs => "en-US",
            Language::EnGb => "en-GB",
            Language::EsEs => "es-ES",
            Language::EsLa => "es-419",
            Language::FrFr => "fr-FR",
            Language::FrCa => "fr-CA",
            Language::ItIt => "it-IT",
            Language::DeDe => "de-DE",
            Language::ZhCn => "zh-CN",
            Language::ZhTw => "zh-TW",
            Language::JaJp => "ja-JP",
            Language::KoKr => "ko-KR",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::EnUs => "English (US)",
            Language::EnGb => "English (UK)",
            Language::EsEs => "Español (España)",
            Language::EsLa => "Español (Latinoamérica)",
            Language::FrFr => "Français (France)",
            Language::FrCa => "Français (Canada)",
            Language::ItIt => "Italiano",
            Language::DeDe => "Deutsch",
            Language::ZhCn => "简体中文 (Simplified Chinese)",
            Language::ZhTw => "繁體中文 (Traditional Chinese)",
            Language::JaJp => "日本語 (Japanese)",
            Language::KoKr => "한국어 (Korean)",
        }
    }

    pub fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|l| l == self).unwrap_or(0);
        let next_idx = (idx + 1) % Self::ALL.len();
        Self::ALL[next_idx]
    }

    pub fn prev(&self) -> Self {
        let idx = Self::ALL.iter().position(|l| l == self).unwrap_or(0);
        let prev_idx = if idx == 0 {
            Self::ALL.len() - 1
        } else {
            idx - 1
        };
        Self::ALL[prev_idx]
    }

    pub fn from_code(code: &str) -> Self {
        let clean = code.trim().replace('_', "-").to_lowercase();
        if clean.starts_with("en-gb") || clean.starts_with("en-uk") {
            Language::EnGb
        } else if clean.starts_with("es-es") {
            Language::EsEs
        } else if clean.starts_with("es") {
            Language::EsLa
        } else if clean.starts_with("fr-ca") {
            Language::FrCa
        } else if clean.starts_with("fr") {
            Language::FrFr
        } else if clean.starts_with("it") {
            Language::ItIt
        } else if clean.starts_with("de") {
            Language::DeDe
        } else if clean.starts_with("zh-tw")
            || clean.starts_with("zh-hk")
            || clean.starts_with("zh-hant")
        {
            Language::ZhTw
        } else if clean.starts_with("zh") {
            Language::ZhCn
        } else if clean.starts_with("ja") {
            Language::JaJp
        } else if clean.starts_with("ko") {
            Language::KoKr
        } else {
            Language::EnUs
        }
    }

    pub fn from_system_locale() -> Self {
        if let Some(locale) = sys_locale::get_locale() {
            Self::from_code(&locale)
        } else {
            Language::EnUs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_cycling() {
        let mut lang = Language::EnUs;
        lang = lang.next();
        assert_eq!(lang, Language::EnGb);
        lang = lang.prev();
        assert_eq!(lang, Language::EnUs);
        lang = lang.prev();
        assert_eq!(lang, Language::KoKr);
    }

    #[test]
    fn test_from_code_parsing() {
        assert_eq!(Language::from_code("en-US"), Language::EnUs);
        assert_eq!(Language::from_code("en_GB"), Language::EnGb);
        assert_eq!(Language::from_code("es-ES"), Language::EsEs);
        assert_eq!(Language::from_code("es-MX"), Language::EsLa);
        assert_eq!(Language::from_code("fr-CA"), Language::FrCa);
        assert_eq!(Language::from_code("fr-FR"), Language::FrFr);
        assert_eq!(Language::from_code("it-IT"), Language::ItIt);
        assert_eq!(Language::from_code("de-DE"), Language::DeDe);
        assert_eq!(Language::from_code("zh-CN"), Language::ZhCn);
        assert_eq!(Language::from_code("zh-TW"), Language::ZhTw);
        assert_eq!(Language::from_code("ja-JP"), Language::JaJp);
        assert_eq!(Language::from_code("ko-KR"), Language::KoKr);
        assert_eq!(Language::from_code("unknown"), Language::EnUs);
    }

    #[test]
    fn test_locale_files_exist_and_match_keys() {
        let en_us_json = include_str!("../locales/en-US.json");
        let en_gb_json = include_str!("../locales/en-GB.json");

        let us_val: serde_json::Value =
            serde_json::from_str(en_us_json).expect("en-US json invalid");
        let gb_val: serde_json::Value =
            serde_json::from_str(en_gb_json).expect("en-GB json invalid");

        // Verify top-level keys parity
        let us_obj = us_val.as_object().unwrap();
        let gb_obj = gb_val.as_object().unwrap();

        for (k, _) in us_obj {
            assert!(gb_obj.contains_key(k), "en-GB missing section: {}", k);
            if let Some(sub_us) = us_obj[k].as_object() {
                let sub_gb = gb_obj[k].as_object().unwrap();
                for (sub_k, _) in sub_us {
                    assert!(
                        sub_gb.contains_key(sub_k),
                        "en-GB missing key in {}: {}",
                        k,
                        sub_k
                    );
                }
            }
        }
    }

    #[test]
    fn test_runtime_locale_switching() {
        rust_i18n::set_locale("en-US");
        assert_eq!(t!("settings.title"), "Settings");
        assert_eq!(
            t!("settings.language", name = "English (US)"),
            "Language: English (US)"
        );
        assert!(t!("how_to_play.line_7").contains("centered"));

        rust_i18n::set_locale("en-GB");
        assert_eq!(t!("settings.title"), "Settings");
        assert!(t!("how_to_play.line_7").contains("centred"));

        // Reset to en-US
        rust_i18n::set_locale("en-US");
        assert_eq!(t!("settings.title"), "Settings");
    }
}
