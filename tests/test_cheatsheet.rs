//! Tests for the cheat-sheet document loading and view filtering.

#![cfg(feature = "tui")]

use mdbook_plotly::docs_parser::parse_doc;
use mdbook_plotly::tui::cheatsheet::{CACHE_VERSION, cache_file};
use mdbook_plotly::tui::cheatsheet_view::CheatSheetView;
use mdbook_plotly::tui::locale::DocLang;
use std::path::Path;

fn read_usage_en() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/USAGE.md");
    std::fs::read_to_string(&path).unwrap()
}

#[test]
fn cache_file_is_versioned_and_language_scoped() {
    for (lang, key) in [(DocLang::English, "en"), (DocLang::Chinese, "zh_CN")] {
        let name = cache_file(lang)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        assert!(
            name.starts_with(&format!("usage-v{CACHE_VERSION}-{key}")),
            "cache file must be schema-versioned and language-scoped, got {name}"
        );
    }
    assert_ne!(cache_file(DocLang::English), cache_file(DocLang::Chinese));
}

#[test]
fn filter_and_select() {
    let mut view = CheatSheetView::new();
    view.doc = Some(parse_doc(&read_usage_en()));
    view.loading = false;
    let before = view.filtered().len();
    assert!(before > 0);
    // Search narrows the list.
    for c in "line".chars() {
        view.search.insert(c);
    }
    let narrowed = view.filtered();
    assert!(!narrowed.is_empty());
    assert!(narrowed.len() <= before);
}

#[test]
fn doc_language_picks_the_right_file() {
    assert_eq!(DocLang::English.doc_file_name(), "USAGE.md");
    assert_eq!(DocLang::Chinese.doc_file_name(), "USAGE-zh_CN.md");
}
