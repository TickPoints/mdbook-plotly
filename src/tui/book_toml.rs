//! Guided editor logic for the plugin's configuration inside `book.toml`.
//!
//! Uses `toml_edit` (not `toml`) so comments, key order and formatting are
//! preserved. Only plugin-relevant keys are editable. Writes go through a
//! temp file + rename so the file is never left half-written.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use toml_edit::DocumentMut;

/// Walk up from `start` looking for a `book.toml`.
pub fn find_book_toml(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start.to_path_buf());
    while let Some(d) = dir {
        let candidate = d.join("book.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    /// Free-form string.
    Text(String),
    /// Comma-separated list, edited as text.
    StringList(Vec<String>),
    /// Boolean toggle.
    Bool(bool),
    /// One-of-N choices.
    Enum(Vec<String>, usize),
}

impl ItemKind {
    pub fn enum_choice(&self, idx: usize) -> Option<String> {
        match self {
            ItemKind::Enum(choices, _) => choices.get(idx).cloned(),
            _ => None,
        }
    }

    pub fn display_value(&self) -> String {
        match self {
            ItemKind::Text(s) => s.clone(),
            ItemKind::StringList(list) => {
                if list.is_empty() {
                    "[]".into()
                } else {
                    format!(
                        "[{}]",
                        list.iter()
                            .map(|s| format!("\"{s}\""))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            ItemKind::Bool(b) => b.to_string(),
            ItemKind::Enum(_, idx) => self
                .enum_choice(*idx)
                .map(|s| format!("\"{s}\""))
                .unwrap_or_default(),
        }
    }

    pub fn parse_text(&mut self, text: &str) {
        match self {
            ItemKind::Text(s) => *s = text.trim().to_string(),
            ItemKind::StringList(list) => {
                *list = split_list(text);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigItem {
    /// Dotted TOML path, e.g. `preprocessor.plotly.output-type`.
    pub path: String,
    /// One-line description shown in the UI.
    pub description: String,
    /// Allowed values / range, shown in the UI.
    pub valid: String,
    /// Default when unset, shown in the UI.
    pub default: String,
    pub kind: ItemKind,
}

/// The focused set of plugin-relevant items. Read from an existing
/// `DocumentMut`; defaults are filled in so the user always sees values.
pub fn build_items(doc: &DocumentMut) -> Vec<ConfigItem> {
    let items = vec![
        ConfigItem {
            path: "book.title".into(),
            description: "The book title used in the HTML header and search.".into(),
            valid: "any string".into(),
            default: "mdbook".into(),
            kind: ItemKind::Text(get_str(doc, "book.title").unwrap_or_else(|| "mdbook".into())),
        },
        ConfigItem {
            path: "preprocessor.plotly.after".into(),
            description: "Other preprocessors that must run before this one.".into(),
            valid: "list of names, e.g. [\"links\"]".into(),
            default: "[\"links\"]".into(),
            kind: ItemKind::StringList(get_array(doc, "preprocessor.plotly.after")),
        },
        ConfigItem {
            path: "preprocessor.plotly.output-type".into(),
            description: "Rendered chart format.".into(),
            valid: "\"plotly-html\" | \"plotly-svg\" (experimental)".into(),
            default: "\"plotly-html\"".into(),
            kind: enum_item(
                doc,
                "preprocessor.plotly.output-type",
                &["plotly-html", "plotly-svg"],
            ),
        },
        ConfigItem {
            path: "preprocessor.plotly.input-type".into(),
            description: "Syntax of chart definitions inside plot blocks.".into(),
            valid: "\"json-input\" | \"toml-input\"".into(),
            default: "\"json-input\"".into(),
            kind: enum_item(
                doc,
                "preprocessor.plotly.input-type",
                &["json-input", "toml-input"],
            ),
        },
        ConfigItem {
            path: "preprocessor.plotly.map-eval.enabled".into(),
            description: "Master switch for map expression evaluation.".into(),
            valid: "true | false".into(),
            default: "true".into(),
            kind: ItemKind::Bool(
                get_bool(doc, "preprocessor.plotly.map-eval.enabled").unwrap_or(true),
            ),
        },
        ConfigItem {
            path: "preprocessor.plotly.map-eval.reuse-slab".into(),
            description: "Reuse the evaluation slab across expressions.".into(),
            valid: "true | false".into(),
            default: "true".into(),
            kind: ItemKind::Bool(
                get_bool(doc, "preprocessor.plotly.map-eval.reuse-slab").unwrap_or(true),
            ),
        },
        ConfigItem {
            path: "preprocessor.plotly.map-eval.compile-expressions".into(),
            description: "Compile fasteval expressions for faster evaluation.".into(),
            valid: "true | false".into(),
            default: "true".into(),
            kind: ItemKind::Bool(
                get_bool(doc, "preprocessor.plotly.map-eval.compile-expressions").unwrap_or(true),
            ),
        },
        ConfigItem {
            path: "preprocessor.plotly.map-eval.namespace-scope".into(),
            description: "Which symbols are visible to map expressions.".into(),
            valid: "\"full-map\" | \"exports-only\"".into(),
            default: "\"full-map\"".into(),
            kind: enum_item(
                doc,
                "preprocessor.plotly.map-eval.namespace-scope",
                &["full-map", "exports-only"],
            ),
        },
    ];
    items
}

fn enum_item(doc: &DocumentMut, path: &str, choices: &[&str]) -> ItemKind {
    let current = get_str(doc, path).unwrap_or_default();
    let idx = choices.iter().position(|c| *c == current).unwrap_or(0);
    ItemKind::Enum(choices.iter().map(|s| s.to_string()).collect(), idx)
}

fn get_str(doc: &DocumentMut, path: &str) -> Option<String> {
    let mut parts = path.split('.');
    let first = parts.next()?;
    let mut item = doc.get(first)?;
    for part in parts {
        item = item.get(part)?;
    }
    item.as_str().map(String::from)
}

fn get_bool(doc: &DocumentMut, path: &str) -> Option<bool> {
    let mut parts = path.split('.');
    let first = parts.next()?;
    let mut item = doc.get(first)?;
    for part in parts {
        item = item.get(part)?;
    }
    item.as_bool()
}

fn get_array(doc: &DocumentMut, path: &str) -> Vec<String> {
    let mut parts = path.split('.');
    let Some(first) = parts.next() else {
        return Vec::new();
    };
    let Some(mut item) = doc.get(first) else {
        return Vec::new();
    };
    for part in parts {
        let Some(next) = item.get(part) else {
            return Vec::new();
        };
        item = next;
    }
    item.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Apply the edited value back into the document. Intermediate tables are
/// created on demand; existing formatting/comment structure is preserved.
pub fn apply_item(doc: &mut DocumentMut, item: &ConfigItem) {
    let parts: Vec<&str> = item.path.split('.').collect();
    let value: toml_edit::Value = match &item.kind {
        ItemKind::Text(s) => s.as_str().into(),
        ItemKind::StringList(list) => toml_edit::Value::Array(
            list.iter()
                .map(|s| toml_edit::Value::from(s.as_str()))
                .collect(),
        ),
        ItemKind::Bool(b) => (*b).into(),
        ItemKind::Enum(_, idx) => {
            item.kind
                .enum_choice(*idx)
                .map(|s| s.into())
                .unwrap_or(toml_edit::Value::String(toml_edit::Formatted::new(
                    String::new(),
                )))
        }
    };

    let mut cur = &mut **doc;
    for (i, part) in parts.iter().enumerate() {
        let part = *part;
        let is_last = i + 1 == parts.len();
        if is_last {
            cur[part] = toml_edit::Item::Value(value);
            return;
        }
        let missing_or_not_table = cur.get(part).is_none_or(|it| !it.is_table());
        if missing_or_not_table {
            cur[part] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        cur = cur[part].as_table_mut().expect("table was just created");
    }
}

pub fn split_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(|s| s.trim().trim_matches('"').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Minimal line-based diff (`-` removed, `+` added, ` ` unchanged).
pub fn diff(old: &str, new: &str) -> Vec<DiffLine> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let n = a.len();
    let m = b.len();
    // LCS DP
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if a[i] == b[j] {
            out.push(DiffLine::Same(a[i].to_string()));
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            out.push(DiffLine::Removed(a[i].to_string()));
            i += 1;
        } else {
            out.push(DiffLine::Added(b[j].to_string()));
            j += 1;
        }
    }
    while i < n {
        out.push(DiffLine::Removed(a[i].to_string()));
        i += 1;
    }
    while j < m {
        out.push(DiffLine::Added(b[j].to_string()));
        j += 1;
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLine {
    Same(String),
    Added(String),
    Removed(String),
}

/// Atomic write: write to a temp file in the same directory, then rename
/// over the target. On Windows (where rename-over-existing fails) the old
/// file is removed first as a fallback.
pub fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::Builder::new()
        .prefix(".book.toml.")
        .tempfile_in(dir)?;
    std::io::Write::write_all(&mut tmp, content.as_bytes())?;
    std::io::Write::flush(&mut tmp)?;
    match tmp.persist(path) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.error.kind() == std::io::ErrorKind::AlreadyExists {
                let _ = std::fs::remove_file(path);
                e.file.persist(path).map_err(|e| e.error)?;
                Ok(())
            } else {
                Err(e.error)
            }
        }
    }
}

/// Render the diff into a printable string.
pub fn diff_to_string(diff: &[DiffLine]) -> String {
    let mut s = String::new();
    for line in diff {
        match line {
            DiffLine::Same(t) => {
                let _ = writeln!(s, "  {t}");
            }
            DiffLine::Added(t) => {
                let _ = writeln!(s, "+ {t}");
            }
            DiffLine::Removed(t) => {
                let _ = writeln!(s, "- {t}");
            }
        }
    }
    s
}
