//! Tests for the settings file parsing and language detection.

#![cfg(feature = "tui")]

use mdbook_plotly::tui::locale::{DocLang, language_from_name};
use mdbook_plotly::tui::settings::parse_config;

#[test]
fn parses_config_file_ignoring_unknown_keys() {
    let text = r#"
        [language]
        doc = "zh_CN"
        [github]
        proxy = "https://ghproxy.com/"
        download = "https://mirror.example.com"
        future-key = 42
    "#;
    let config = parse_config(text);
    assert_eq!(config.language.as_deref(), Some("zh_CN"));
    assert_eq!(config.github.proxy.as_deref(), Some("https://ghproxy.com/"));
    assert_eq!(
        config.github.download.as_deref(),
        Some("https://mirror.example.com")
    );
    assert!(config.github.api.is_none());
}

#[test]
fn empty_or_invalid_config_is_defaults() {
    assert_eq!(parse_config("").github, Default::default());
    assert_eq!(parse_config("").language, None);
    assert_eq!(parse_config("not toml [").github, Default::default());
}

#[test]
fn config_with_only_language() {
    let config = parse_config("[language]\ndoc = \"en\"\n");
    assert_eq!(config.language.as_deref(), Some("en"));
    assert_eq!(config.github, Default::default());
}

#[test]
fn language_from_name_selects_chinese_for_zh() {
    for name in ["zh", "zh-CN", "zh_CN", "ZH_CN", "zh-Hans-CN", " zh "] {
        assert_eq!(language_from_name(name), DocLang::Chinese, "for '{name}'");
    }
    for name in ["en", "en-US", "ja", "de", ""] {
        assert_eq!(language_from_name(name), DocLang::English, "for '{name}'");
    }
}

#[test]
fn doc_lang_maps_to_files_and_cache_keys() {
    assert_eq!(DocLang::English.doc_file_name(), "USAGE.md");
    assert_eq!(DocLang::Chinese.doc_file_name(), "USAGE-zh_CN.md");
    assert_eq!(DocLang::English.cache_key(), "en");
    assert_eq!(DocLang::Chinese.cache_key(), "zh_CN");
}
