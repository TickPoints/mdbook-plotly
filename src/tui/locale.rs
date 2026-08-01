//! Documentation language selection.
//!
//! Resolution order (highest priority first):
//! 1. `MDBOOK_PLOTLY_LANG` environment variable (`zh`, `zh-CN`, `en`, …)
//! 2. `[language] doc` in the settings file
//! 3. The system locale (via `sys-locale`)
//! 4. English (default)

use crate::tui::settings;

/// Which user manual the cheat-sheet reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocLang {
    English,
    Chinese,
}

impl DocLang {
    /// The file name inside `docs/`.
    pub fn doc_file_name(self) -> &'static str {
        match self {
            DocLang::English => "USAGE.md",
            DocLang::Chinese => "USAGE-zh_CN.md",
        }
    }

    /// Stable key used in cache file names and messages.
    pub fn cache_key(self) -> &'static str {
        match self {
            DocLang::English => "en",
            DocLang::Chinese => "zh_CN",
        }
    }

    /// Short human-readable label.
    pub fn label(self) -> &'static str {
        self.cache_key()
    }
}

/// Map a locale/language name to a documentation language.
/// Anything starting with `zh` selects Chinese.
pub fn language_from_name(name: &str) -> DocLang {
    let name = name.trim().to_ascii_lowercase();
    if name.starts_with("zh") {
        DocLang::Chinese
    } else {
        DocLang::English
    }
}

/// Resolve the documentation language for this run.
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
