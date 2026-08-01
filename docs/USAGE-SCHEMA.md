# USAGE.md Machine-Readable Schema

`docs/USAGE.md` and `docs/USAGE-zh_CN.md` are human-readable manuals **and**
machine-readable datasets for the plot cheat-sheet (`mdbook-plotly tui`).
Both language versions must carry the same schema, the same block `id`s, and
the same recipe examples; the cheat-sheet picks one by language (see the
"Documentation Language" section of `USAGE.md`). Editing a manual must never
silently break the parser. This document defines the contract the parser
relies on; `tests/test_usage_schema.rs` enforces it against **both** real
files in CI.

## Schema Version Marker

The document starts with an HTML comment declaring the schema version:

```markdown
<!-- usage-schema: 1 -->
```

- The parser reads this value from near the top of the file (before any
  plot blocks).
- **Unknown (newer) schema version** -> the parser still attempts a
  best-effort parse of `plot:begin`/`plot:end` blocks, records a warning,
  and exposes the version through `UsageDoc::schema_supported() == false`
  so the caller can tell the user to upgrade. It never panics and never
  silently pretends the file is empty.
- **Missing marker** -> treated as "schema version 0"; blocks are still
  parsed so old snapshots keep working.

## Plot Blocks

Each searchable example lives inside a pair of HTML comment sentinels.
HTML comments are invisible in rendered Markdown and cheap for humans to
type:

```markdown
<!-- plot:begin id=line-basic title="Basic Line" tags=line,2d -->
A short description in Markdown. Anything between the begin marker and the
code fence is treated as the block description.

```plotly
{
    data: [{
        type: "scatter",
        x: [0, 1, 2],
        y: [1, 3, 2],
    }]
}
```
<!-- plot:end -->
```

The closing sentinel must be the last content of the block.

### Marker Grammar

`<!-- plot:begin <attr>... -->` where each attribute is
`key="value"` (quoted values may contain spaces / non-ASCII) or
`key=value` (unquoted, no spaces).

Attributes:

| Attribute | Required | Meaning                              | Default      |
|-----------|----------|--------------------------------------|--------------|
| `id`      | yes      | stable unique identifier (ASCII slug)| — (skip+warn if missing) |
| `title`   | no       | human-readable title                 | same as `id` |
| `tags`    | no       | comma-separated search tags          | empty        |

Any other attribute is **ignored** (forward compatible). Unknown keys
must never be an error.

### Body Rules

- The body contains Markdown description text followed by exactly one
  fenced code block whose info string is `plotly` (a `plot` fence is also
  accepted). That block's content is the copyable example.
- The description is the text between the begin sentinel and the code
  fence, trimmed.

## Parser Guarantees (Lenient)

1. Unknown attributes on a block are ignored.
2. Missing optional attributes fall back to their defaults.
3. A single malformed block (missing `id`, missing code fence, unclosed
   sentinel) is skipped with a warning; parsing continues for the rest of
   the document.
4. The parser never returns an `Err` for malformed input. Warnings are
   collected on `UsageDoc.warnings`.
5. `id` values must be unique; duplicates produce a warning and the later
   block wins.

## Conventions for Editors

- Put the `<!-- usage-schema: 1 -->` marker on the first line of **both**
  `docs/USAGE.md` and `docs/USAGE-zh_CN.md`.
- Give every example block a stable `id` (don't rename existing ids —
  search muscle memory and cached copies depend on them), and keep the ids
  identical across the English and Chinese versions.
- Keep the example code minimal, copy-pasteable, and valid JSON5.
- Add `tags` with the trace type plus 1–2 topical tags (e.g.
  `scatter,2d,line`).

## Version History

| Schema version | Meaning                                    |
|----------------|--------------------------------------------|
| 1             | Initial contract: marker + `plot:begin/end` blocks with `id`/`title`/`tags`. |
