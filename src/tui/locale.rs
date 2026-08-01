//! Generator schema language selection.
//!
//! Resolution order (highest priority first):
//! 1. `MDBOOK_PLOTLY_LANG` environment variable (`zh`, `zh-CN`, `en`, …)
//! 2. `[language] doc` in the settings file
//! 3. The system locale (via `sys-locale`)
//! 4. English (default)

use crate::plot_schema::{EMBEDDED_EN, EMBEDDED_ZH_CN};
use crate::tui::settings;

/// Which localized schema the plot generator reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocLang {
    English,
    Chinese,
}

impl DocLang {
    /// The schema file name inside `docs/`.
    pub fn schema_file_name(self) -> &'static str {
        match self {
            DocLang::English => "PLOT-SCHEMA.json",
            DocLang::Chinese => "PLOT-SCHEMA-zh_CN.json",
        }
    }

    /// Embedded schema source for this language.
    pub fn schema_source(self) -> &'static str {
        match self {
            DocLang::English => EMBEDDED_EN,
            DocLang::Chinese => EMBEDDED_ZH_CN,
        }
    }

    /// Short human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            DocLang::English => "en",
            DocLang::Chinese => "zh_CN",
        }
    }
}

/// Map a locale/language name to a schema language.
/// Anything starting with `zh` selects Chinese.
pub fn language_from_name(name: &str) -> DocLang {
    let name = name.trim().to_ascii_lowercase();
    if name.starts_with("zh") {
        DocLang::Chinese
    } else {
        DocLang::English
    }
}

/// Resolve the schema language for this run.
pub fn resolve_language() -> DocLang {
    if let Some(name) = settings::env_language_override() {
        return language_from_name(&name);
    }
    if let Some(name) = settings::language_override() {
        return language_from_name(&name);
    }
    if let Some(locale) = sys_locale::get_locale() {
        return language_from_name(&locale);
    }
    DocLang::English
}
