//! Tests for the book.toml editing logic and the shared text `Input`.

#![cfg(feature = "tui")]

use mdbook_plotly::tui::book_toml::{
    ConfigItem, DiffLine, ItemKind, apply_item, atomic_write, build_items, diff, diff_to_string,
    find_book_toml, split_list,
};
use mdbook_plotly::tui::widget::Input;
use toml_edit::DocumentMut;

const SAMPLE: &str = r#"# My Book
[book]
title = "Original"

# Plotly preprocessor
[preprocessor.plotly]
after = ["links"]
output-type = "plotly-html"
input-type = "json-input"

[preprocessor.plotly.map-eval]
enabled = true
namespace-scope = "full-map"
"#;

#[test]
fn finds_book_toml_upwards() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a/b/c");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(dir.path().join("book.toml"), "").unwrap();
    let found = find_book_toml(&nested).unwrap();
    assert_eq!(found, dir.path().join("book.toml"));
}

#[test]
fn does_not_find_missing_book_toml() {
    let dir = tempfile::tempdir().unwrap();
    assert!(find_book_toml(dir.path()).is_none());
}

#[test]
fn build_items_reads_values_and_defaults() {
    let doc: DocumentMut = SAMPLE.parse().unwrap();
    let items = build_items(&doc);
    assert_eq!(items.len(), 8);
    assert_eq!(items[0].kind.display_value(), "Original");
    assert_eq!(items[1].kind.display_value(), "[\"links\"]");
    assert_eq!(items[2].kind.display_value(), "\"plotly-html\"");
    assert_eq!(items[4].kind.display_value(), "true");
    assert_eq!(items[7].kind.display_value(), "\"full-map\"");
}

#[test]
fn apply_item_creates_missing_tables() {
    let mut doc: DocumentMut = r#"[book]
title = "T"
"#
    .parse()
    .unwrap();
    let mut item = ConfigItem {
        path: "preprocessor.plotly.input-type".into(),
        description: String::new(),
        valid: String::new(),
        default: String::new(),
        kind: ItemKind::Enum(vec!["json-input".into(), "toml-input".into()], 1),
    };
    apply_item(&mut doc, &item);
    assert_eq!(
        doc["preprocessor"]["plotly"]["input-type"].as_str(),
        Some("toml-input")
    );

    // Writing again preserves comments/formatting elsewhere.
    item.kind = ItemKind::Text("Changed".into());
    item.path = "book.title".into();
    apply_item(&mut doc, &item);
    assert!(doc.to_string().contains("Changed"));
    assert!(doc.to_string().contains("[book]"));
}

#[test]
fn diff_shows_add_and_remove() {
    let old = "a\nb\nc\n";
    let new = "a\nx\nc\n";
    let d = diff(old, new);
    assert!(d.contains(&DiffLine::Same("a".into())));
    assert!(d.contains(&DiffLine::Removed("b".into())));
    assert!(d.contains(&DiffLine::Added("x".into())));
    assert!(d.contains(&DiffLine::Same("c".into())));
}

#[test]
fn diff_equal_inputs() {
    assert_eq!(
        diff("a\nb\n", "a\nb\n"),
        vec![DiffLine::Same("a".into()), DiffLine::Same("b".into())]
    );
}

#[test]
fn diff_to_string_roundtrips() {
    let d = diff("a\nb\n", "a\nc\n");
    let s = diff_to_string(&d);
    assert!(s.contains("+ c"));
    assert!(s.contains("- b"));
    assert!(s.contains("  a"));
}

#[test]
fn atomic_write_roundtrips_and_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("book.toml");
    atomic_write(&path, "# comment\nvalue = 1\n").unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "# comment\nvalue = 1\n"
    );
    atomic_write(&path, "# changed\nvalue = 2\n").unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "# changed\nvalue = 2\n"
    );
}

#[test]
fn split_list_is_lenient() {
    assert_eq!(
        split_list("\"links\", \"print\",  foo"),
        vec!["links", "print", "foo"]
    );
    assert_eq!(split_list(""), Vec::<String>::new());
}

#[test]
fn input_editing_ops() {
    let mut input = Input::new();
    for c in "ab".chars() {
        input.insert(c);
    }
    input.left();
    input.insert('X');
    assert_eq!(input.text, "aXb");
    input.backspace();
    assert_eq!(input.text, "ab");
    input.right();
    input.delete();
    assert_eq!(input.text, "ab");
}
