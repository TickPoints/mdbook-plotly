//! Lenient parser for the machine-readable plot-block schema in
//! `docs/USAGE.md`. See `docs/USAGE-SCHEMA.md` for the contract.
//!
//! This module is intentionally dependency-free so it compiles and is
//! tested in every feature combination (including the slim build).

/// The highest schema version this binary understands.
pub const USAGE_SCHEMA_VERSION: u32 = 1;

const SCHEMA_MARKER_PREFIX: &str = "<!-- usage-schema: ";
const PLOT_BEGIN_PREFIX: &str = "<!-- plot:begin";
const PLOT_END: &str = "<!-- plot:end -->";

/// A parsed usage document. Parsing never fails; problems are recorded
/// as warnings and the rest of the document is still parsed.
#[derive(Debug, Clone, Default)]
pub struct UsageDoc {
    /// Schema version declared in the document (0 when absent).
    pub schema_version: u32,
    /// False when the document declares a newer schema than we support.
    pub schema_supported: bool,
    /// Successfully parsed plot blocks, in document order.
    pub plots: Vec<PlotEntry>,
    /// Non-fatal problems encountered while parsing.
    pub warnings: Vec<String>,
}

impl UsageDoc {
    /// True when the document's schema version is understood by this binary.
    pub fn schema_supported(&self) -> bool {
        self.schema_supported
    }

    /// The declared schema version, or `None` when the document has none.
    pub fn declared_schema_version(&self) -> Option<u32> {
        (self.schema_version != 0).then_some(self.schema_version)
    }

    /// Look up a plot block by its stable `id`.
    pub fn get(&self, id: &str) -> Option<&PlotEntry> {
        self.plots.iter().find(|p| p.id == id)
    }
}

/// A single `plot:begin`/`plot:end` block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlotEntry {
    /// Stable identifier (required attribute).
    pub id: String,
    /// Human-readable title (defaults to `id`).
    pub title: String,
    /// Comma-separated search tags.
    pub tags: Vec<String>,
    /// Markdown description between the begin sentinel and the code fence.
    pub description: String,
    /// Content of the `plotly`/`plot` fenced code block.
    pub code: String,
    /// 1-based line of the begin sentinel.
    pub begin_line: usize,
    /// 1-based line of the end sentinel.
    pub end_line: usize,
}

impl PlotEntry {
    /// True when `needle` appears in the id, title, tags, or code.
    pub fn matches(&self, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        self.id.to_lowercase().contains(&needle)
            || self.title.to_lowercase().contains(&needle)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&needle))
            || self.code.to_lowercase().contains(&needle)
    }
}

/// Parse a usage document. Never fails; tolerant of malformed input.
pub fn parse_doc(source: &str) -> UsageDoc {
    let mut doc = UsageDoc {
        schema_supported: true,
        ..UsageDoc::default()
    };

    let lines: Vec<&str> = source.lines().collect();
    parse_schema_marker(&lines, &mut doc);

    let mut i = 0usize;
    while i < lines.len() {
        if lines[i].trim().starts_with(PLOT_BEGIN_PREFIX) {
            match parse_block(&lines, i, &mut doc) {
                Some(next) => i = next,
                None => break,
            }
        } else {
            i += 1;
        }
    }
    doc
}

/// Parse the schema version marker within the first few lines.
fn parse_schema_marker(lines: &[&str], doc: &mut UsageDoc) {
    for (idx, line) in lines.iter().take(50).enumerate() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(SCHEMA_MARKER_PREFIX) else {
            continue;
        };
        let Some(version_part) = rest.strip_suffix(" -->") else {
            doc.warnings.push(format!(
                "Malformed usage-schema marker on line {}: '{}'",
                idx + 1,
                trimmed
            ));
            continue;
        };
        match version_part.trim().parse::<u32>() {
            Ok(version) => {
                doc.schema_version = version;
                if version > USAGE_SCHEMA_VERSION {
                    doc.schema_supported = false;
                    doc.warnings.push(format!(
                        "docs/USAGE.md declares schema version {}, but this binary \
                         supports up to {}. Parsing on a best-effort basis; please \
                         upgrade mdbook-plotly.",
                        version, USAGE_SCHEMA_VERSION
                    ));
                }
            }
            Err(_) => {
                doc.warnings.push(format!(
                    "Invalid usage-schema version '{}' on line {}.",
                    version_part.trim(),
                    idx + 1
                ));
            }
        }
    }
}

/// Parse one block starting at `start` (the begin sentinel line).
/// Returns the index of the next line to resume scanning from.
fn parse_block(lines: &[&str], start: usize, doc: &mut UsageDoc) -> Option<usize> {
    let begin_line = start + 1;
    let attrs = parse_begin_marker(lines[start].trim());
    if attrs.is_none() {
        doc.warnings.push(format!(
            "Skipping malformed plot block at line {}: unreadable begin marker.",
            begin_line
        ));
        return Some(start + 1);
    }
    let attrs = attrs.unwrap();

    let id = match get_attr(&attrs, "id") {
        Some(id) if !id.trim().is_empty() => id.trim().to_string(),
        _ => {
            doc.warnings.push(format!(
                "Skipping plot block at line {}: missing required 'id' attribute.",
                begin_line
            ));
            return skip_to_block_end(lines, start + 1);
        }
    };

    // Find the matching end sentinel, tolerating a nested begin sentinel
    // (which implies the current block is unclosed).
    let mut end = None;
    let mut j = start + 1;
    while j < lines.len() {
        let trimmed = lines[j].trim();
        if trimmed == PLOT_END {
            end = Some(j);
            break;
        }
        if trimmed.starts_with(PLOT_BEGIN_PREFIX) {
            break;
        }
        j += 1;
    }
    let end = match end {
        Some(e) => e,
        None => {
            doc.warnings.push(format!(
                "Skipping plot block '{}' starting at line {}: missing '{}' sentinel.",
                id, begin_line, PLOT_END
            ));
            return Some(j);
        }
    };

    let (code, description) = match extract_code_fence(&lines[start + 1..end]) {
        Some(found) => found,
        None => {
            doc.warnings.push(format!(
                "Skipping plot block '{}' at line {}: no plotly/plot code fence found.",
                id, begin_line
            ));
            return Some(end + 1);
        }
    };

    let title = get_attr(&attrs, "title")
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| id.clone());
    let tags: Vec<String> = get_attr(&attrs, "tags")
        .map(|t| {
            t.split(',')
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
                .collect()
        })
        .unwrap_or_default();

    if let Some(prev) = doc.get(&id) {
        doc.warnings.push(format!(
            "Duplicate plot id '{}' (first at line {}, later at line {}); the later block wins.",
            id, prev.begin_line, begin_line
        ));
    }

    doc.plots.push(PlotEntry {
        id,
        title,
        tags,
        description,
        code,
        begin_line,
        end_line: end + 1,
    });

    Some(end + 1)
}

/// Parse the attributes out of a `<!-- plot:begin ... -->` marker.
fn parse_begin_marker(trimmed: &str) -> Option<Vec<(String, String)>> {
    let rest = trimmed.strip_prefix(PLOT_BEGIN_PREFIX)?;
    let inner = rest.strip_suffix(" -->")?;
    Some(parse_attrs(inner))
}

/// Look up an attribute by key (first occurrence wins).
fn get_attr<'a>(attrs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// Scan forward until the next begin sentinel or EOF. Used after skipping
/// a malformed block so parsing can resume cleanly.
fn skip_to_block_end(lines: &[&str], from: usize) -> Option<usize> {
    let mut j = from;
    while j < lines.len() {
        if lines[j].trim().starts_with(PLOT_BEGIN_PREFIX) {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// Find the first `plotly`/`plot` fenced code block inside the slice.
/// Returns `(code, description)`.
fn extract_code_fence(lines: &[&str]) -> Option<(String, String)> {
    let mut i = 0usize;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some("plotly" | "plot") = fence_info(trimmed) {
            let description = lines[..i].join("\n").trim().to_string();
            let mut code_lines = Vec::new();
            i += 1;
            while i < lines.len() && fence_info(lines[i].trim()).is_none() {
                code_lines.push(lines[i]);
                i += 1;
            }
            let code = code_lines.join("\n");
            let code = code.trim_matches('\n').to_string();
            return Some((code, description));
        }
        i += 1;
    }
    None
}

/// For a trimmed line, return the fence info string when it opens/closes a
/// backtick-fenced code block, otherwise `None`. A closing fence has an
/// empty info string.
fn fence_info(trimmed: &str) -> Option<&str> {
    let ticks = trimmed.as_bytes();
    if ticks.first() != Some(&b'`') {
        return None;
    }
    let run = trimmed.bytes().take_while(|&b| b == b'`').count();
    if run < 3 {
        return None;
    }
    let info = &trimmed[run..];
    if info.trim().is_empty() {
        Some("")
    } else {
        Some(info.trim())
    }
}

/// Parse `key=value` / `key="value with spaces"` / `key='value'` pairs.
/// Unknown keys and malformed tokens are ignored.
fn parse_attrs(inner: &str) -> Vec<(String, String)> {
    let bytes = inner.as_bytes();
    let mut i = 0usize;
    let n = bytes.len();
    let mut attrs = Vec::new();

    while i < n {
        while i < n && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= n {
            break;
        }
        let key_start = i;
        while i < n && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let key = &inner[key_start..i];
        while i < n && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= n || bytes[i] != b'=' {
            continue;
        }
        i += 1;
        while i < n && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= n {
            break;
        }
        let value = match bytes[i] {
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                let vstart = i;
                while i < n && bytes[i] != quote {
                    i += 1;
                }
                let value = inner[vstart..i].to_string();
                if i < n {
                    i += 1;
                }
                value
            }
            _ => {
                let vstart = i;
                while i < n && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                inner[vstart..i].to_string()
            }
        };
        attrs.push((key.to_string(), value));
    }
    attrs
}
