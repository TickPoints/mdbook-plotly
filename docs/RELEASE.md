# Release Assets

This document is the single source of truth shared by the release CI
(`.github/workflows/release.yml`), the self-update logic in the `tui`
variant, and anyone publishing an asset by hand.

## Release Asset Naming

Every published binary follows this scheme:

```
mdbook-plotly-<version>-<target-triple>[-tui].<ext>
mdbook-plotly-<version>-<target-triple>[-tui].<ext>.sha256
```

Field rules:

- `<version>` is the release tag with the leading `v` removed (e.g. tag
  `v0.3.0` produces asset version `0.3.0`).
- `<target-triple>` is the Rust target triple the binary was built for
  (e.g. `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`,
  `x86_64-pc-windows-msvc`).
- `[-tui]` is a literal `-tui` suffix that marks the **full** variant
  (built with the `tui` feature). The slim variant has **no** suffix.
  The suffix is the only thing that distinguishes the two variants, so it
  must never be elided.
- `<ext>` is `tar.gz` on Unix-like targets and `zip` on Windows targets.
- Every asset ships with a matching `.sha256` checksum file containing
  the hex digest followed by a space and the asset file name, e.g.:
  `4a2f…9b1e  mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz`

Examples (release `v0.3.0`):

```
mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu.tar.gz
mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz
mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu.tar.gz.sha256
mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz.sha256
mdbook-plotly-0.3.0-x86_64-pc-windows-msvc.zip
mdbook-plotly-0.3.0-x86_64-pc-windows-msvc-tui.zip
```

## Variant Semantics

The compiled binary knows its own variant at compile time:

- `#[cfg(feature = "tui")]` -> variant `tui`, asset name contains `-tui`.
- otherwise -> variant `slim`, asset name contains no suffix.

The self-update logic only ever looks for assets belonging to its own
variant. A slim binary must never update itself from a `-tui` asset and
vice versa; updating across variants is a user error and is reported as
such.

## Checksum Verification

- Updates verify the downloaded archive's SHA-256 against the matching
  `.sha256` asset **before** replacing the running binary.
- The CI generates both the archive and its `.sha256` in the same step so
  they can never go stale relative to each other.

## Network Configuration

The self-update logic never hardcodes a GitHub URL: it builds every URL
from the user's GitHub proxy/mirror settings (see the "GitHub Proxy /
Mirror" section of `docs/USAGE.md`). Asset names, checksums, and asset
layout are independent of those settings — only the transport changes.

## What a Release Must Contain

1. One slim binary asset + checksum per supported target.
2. One `-tui` binary asset + checksum per supported target.
3. Release notes that explain the difference between the two variants and
   tell the reader how to pick (see `docs/CHANGELOG.md`).

## Pushing a Release (Humans)

```shell
# bump Cargo.toml, update docs/CHANGELOG.md, commit, then:
git tag -s v0.3.0
git push origin v0.3.0
```

The tag pattern `v[0-9]+.*` triggers the release workflow, which builds
both variants for every supported target and uploads them under the names
defined above.
