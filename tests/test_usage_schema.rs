//! Enforces the machine-readable schema contract against the real
//! `docs/USAGE.md` and `docs/USAGE-zh_CN.md`, and covers the parser's
//! lenient behaviors. If someone edits either manual in a way that breaks
//! the plot-block schema, this test goes red in CI instead of surfacing
//! later as a bug report.

use mdbook_plotly::docs_parser::{self, USAGE_SCHEMA_VERSION, parse_doc};
use std::path::Path;

const DOCS: &[&str] = &["docs/USAGE.md", "docs/USAGE-zh_CN.md"];

fn read_doc(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

#[test]
fn all_docs_parse_with_blocks_and_complete_fields() {
    for doc_name in DOCS {
        let doc = docs_parser::parse_doc(&read_doc(doc_name));
        assert!(
            !doc.plots.is_empty(),
            "{doc_name}: must contain at least one plot block; found none — \
             did the schema markers get removed?"
        );
        for (index, plot) in doc.plots.iter().enumerate() {
            let ctx = format!(
                "{doc_name} plot block #{} ('{}', lines {}..{})",
                index, plot.id, plot.begin_line, plot.end_line
            );
            assert!(!plot.id.is_empty(), "{ctx}: id must not be empty");
            assert!(!plot.title.is_empty(), "{ctx}: title must not be empty");
            assert!(!plot.code.is_empty(), "{ctx}: code must not be empty");
            assert!(
                !plot.description.is_empty(),
                "{ctx}: description must not be empty"
            );
            assert!(
                plot.begin_line < plot.end_line,
                "{ctx}: begin_line must precede end_line"
            );
        }
    }
}

#[test]
fn all_docs_schema_version_is_supported() {
    for doc_name in DOCS {
        let doc = docs_parser::parse_doc(&read_doc(doc_name));
        assert!(
            doc.schema_supported(),
            "{doc_name}: declares schema version {}, but this binary supports up to {}. \
             Update the docs or the binary.",
            doc.declared_schema_version()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            USAGE_SCHEMA_VERSION
        );
    }
}

#[test]
fn all_docs_block_ids_are_unique() {
    for doc_name in DOCS {
        let doc = docs_parser::parse_doc(&read_doc(doc_name));
        let mut ids = std::collections::BTreeSet::new();
        for plot in &doc.plots {
            assert!(
                ids.insert(plot.id.clone()),
                "{doc_name}: duplicate plot id '{}' (line {})",
                plot.id,
                plot.begin_line
            );
        }
    }
}

#[test]
fn all_docs_have_no_parser_warnings() {
    for doc_name in DOCS {
        let doc = docs_parser::parse_doc(&read_doc(doc_name));
        assert!(
            doc.warnings.is_empty(),
            "{doc_name} produced parser warnings:\n  {}",
            doc.warnings.join("\n  ")
        );
    }
}

#[test]
fn recipe_examples_are_valid_plot_input() {
    let config = mdbook_plotly::preprocessor::config::MapEvalConfig::default();
    for doc_name in DOCS {
        let doc = docs_parser::parse_doc(&read_doc(doc_name));
        for plot in &doc.plots {
            if let Err(e) =
                mdbook_plotly::code_handler::handle_json_input(plot.code.clone(), &config)
            {
                panic!(
                    "{doc_name} plot block '{}' (line {}) is not valid chart input: {e}",
                    plot.id, plot.begin_line
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Parser behaviors (exercised through the public `parse_doc` API)
// ---------------------------------------------------------------------------

const SAMPLE: &str = r#"<!-- usage-schema: 1 -->

# Title

<!-- plot:begin id=line-basic title="Basic Line" tags=line,2d -->
A line description.
```plotly
{
    data: [{ type: "scatter", x: [0, 1], y: [1, 2] }],
}
```
<!-- plot:end -->

<!-- plot:begin id=bar-basic title="Bar" tags=bar -->
Bar description.
```plot
{ data: [{ type: "bar" }] }
```
<!-- plot:end -->
"#;

#[test]
fn parses_basic_document() {
    let doc = parse_doc(SAMPLE);
    assert!(doc.schema_supported());
    assert_eq!(doc.declared_schema_version(), Some(1));
    assert_eq!(doc.plots.len(), 2);
    assert_eq!(doc.warnings.len(), 0);
    assert_eq!(doc.plots[0].id, "line-basic");
    assert_eq!(doc.plots[0].title, "Basic Line");
    assert_eq!(doc.plots[0].tags, vec!["line", "2d"]);
    assert!(doc.plots[0].code.contains("\"scatter\""));
    assert_eq!(doc.plots[1].id, "bar-basic");
}

#[test]
fn tolerates_newer_schema_version() {
    let source = SAMPLE.replace("usage-schema: 1", "usage-schema: 99");
    let doc = parse_doc(&source);
    assert!(!doc.schema_supported());
    assert_eq!(doc.declared_schema_version(), Some(99));
    assert_eq!(doc.plots.len(), 2, "best-effort parse still happens");
    assert!(doc.warnings.iter().any(|w| w.contains("upgrade")));
}

#[test]
fn missing_marker_means_version_zero() {
    let source = SAMPLE.lines().skip(1).collect::<Vec<_>>().join("\n");
    let doc = parse_doc(&source);
    assert!(doc.schema_supported());
    assert_eq!(doc.declared_schema_version(), None);
    assert_eq!(doc.plots.len(), 2);
}

#[test]
fn block_without_id_is_skipped_with_warning() {
    let source = SAMPLE.replace("id=line-basic ", "");
    let doc = parse_doc(&source);
    assert_eq!(doc.plots.len(), 1, "the id-less block is dropped");
    assert_eq!(doc.plots[0].id, "bar-basic");
    assert!(
        doc.warnings
            .iter()
            .any(|w| w.contains("missing required 'id'"))
    );
}

#[test]
fn block_without_code_fence_is_skipped_with_warning() {
    let source = SAMPLE.replace("```plotly\n", "```json\n");
    let doc = parse_doc(&source);
    assert_eq!(doc.plots.len(), 1);
    assert!(
        doc.warnings
            .iter()
            .any(|w| w.contains("no plotly/plot code fence"))
    );
}

#[test]
fn unclosed_block_is_skipped_with_warning() {
    let source = SAMPLE.replacen("<!-- plot:end -->", "", 1);
    let doc = parse_doc(&source);
    assert_eq!(doc.plots.len(), 1, "the unclosed block is dropped");
    assert_eq!(doc.plots[0].id, "bar-basic");
    assert!(
        doc.warnings
            .iter()
            .any(|w| w.contains("missing '<!-- plot:end -->'"))
    );
}

#[test]
fn unknown_attributes_are_ignored() {
    let source = SAMPLE.replace("id=line-basic ", "id=line-basic custom=yes ");
    let doc = parse_doc(&source);
    assert_eq!(doc.plots.len(), 2);
    assert_eq!(doc.warnings.len(), 0);
}

#[test]
fn duplicate_ids_warn_and_last_wins() {
    let source = SAMPLE.replace("id=bar-basic", "id=line-basic");
    let doc = parse_doc(&source);
    assert_eq!(doc.plots.len(), 2);
    assert!(doc.warnings.iter().any(|w| w.contains("Duplicate plot id")));
    assert!(doc.plots[0].code.contains("\"scatter\""));
    assert!(doc.plots[1].code.contains("\"bar\""));
}

#[test]
fn quoted_titles_keep_inner_spaces_and_unicode() {
    let source = SAMPLE.replace("title=\"Basic Line\"", "title=\"基础折线 chart\"");
    let doc = parse_doc(&source);
    assert_eq!(doc.plots[0].title, "基础折线 chart");
}

#[test]
fn missing_optional_fields_have_defaults() {
    let source = SAMPLE
        .replace(" title=\"Basic Line\"", "")
        .replace(" tags=line,2d", "");
    let doc = parse_doc(&source);
    assert_eq!(doc.plots[0].title, "line-basic");
    assert!(doc.plots[0].tags.is_empty());
}

#[test]
fn matches_searches_id_title_tags_and_code() {
    let doc = parse_doc(SAMPLE);
    assert!(doc.plots[0].matches("line"));
    assert!(doc.plots[0].matches("basic"));
    assert!(doc.plots[0].matches("2d"));
    assert!(doc.plots[0].matches("scatter"));
    assert!(!doc.plots[0].matches("bar"));
}
