//! Plot cheat-sheet document loading.
//!
//! The user manual is fetched from GitHub at the release tag matching the
//! running binary's version (never `main`), in the language selected by
//! [`crate::tui::locale`]. There is no compile-time embedded copy; the
//! local cache makes the cheat-sheet work offline after the first fetch.
//! All URLs honour the GitHub proxy/mirror settings.

use std::path::PathBuf;
use std::time::Duration;

use crate::docs_parser::{UsageDoc, parse_doc};
use crate::tui::github::{GithubHosts, RepoSpec};
use crate::tui::locale::DocLang;

/// Bump together with the USAGE schema version so a changed format never
/// reuses a stale cache file.
pub const CACHE_VERSION: u32 = 1;
/// How long a cached copy is considered fresh.
pub const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
/// Timeout for fetching the manual.
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(15);
pub const FETCH_PER_CALL_TIMEOUT: Duration = Duration::from_secs(10);

/// Where the loaded document came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocSource {
    Cache,
    Network,
}

impl DocSource {
    pub fn label(self) -> &'static str {
        match self {
            DocSource::Cache => "cache",
            DocSource::Network => "github@tag",
        }
    }
}

pub fn cache_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "mdbook-plotly").map(|d| d.cache_dir().to_path_buf())
}

/// Language- and schema-versioned cache file, so switching locale or
/// upgrading the schema never serves stale content from the wrong source.
pub fn cache_file(lang: DocLang) -> PathBuf {
    cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("usage-v{CACHE_VERSION}-{}.md", lang.cache_key()))
}

fn cache_is_fresh(cache_path: &PathBuf) -> bool {
    std::fs::metadata(cache_path)
        .and_then(|m| m.modified())
        .map(|t| t.elapsed().map(|age| age < CACHE_TTL).unwrap_or(false))
        .unwrap_or(false)
}

fn read_cache(cache_path: &PathBuf) -> Option<String> {
    let text = std::fs::read_to_string(cache_path).ok()?;
    (!text.trim().is_empty()).then_some(text)
}

/// The release tag matching this binary.
pub fn doc_tag() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

/// The path of the manual for a language, relative to the repo root.
pub fn doc_path(lang: DocLang) -> String {
    format!("docs/{}", lang.doc_file_name())
}

/// Load the usage document: fresh cache → GitHub tag → stale cache.
/// `refresh` bypasses the cache TTL.
pub fn load_doc(
    lang: DocLang,
    hosts: &GithubHosts,
    repo: &RepoSpec,
    refresh: bool,
) -> Result<(UsageDoc, DocSource), String> {
    let cache_path = cache_file(lang);

    if !refresh
        && cache_is_fresh(&cache_path)
        && let Some(text) = read_cache(&cache_path)
    {
        return Ok((parse_doc(&text), DocSource::Cache));
    }

    match fetch_tagged_doc(lang, hosts, repo) {
        Ok(text) => {
            if let Some(dir) = cache_dir() {
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::fs::write(&cache_path, &text);
            }
            Ok((parse_doc(&text), DocSource::Network))
        }
        Err(fetch_err) => {
            // Fall back to any stale cache so offline still works.
            if let Some(text) = read_cache(&cache_path) {
                return Ok((parse_doc(&text), DocSource::Cache));
            }
            let url = doc_url(lang, hosts, repo);
            Err(format!("cannot fetch {url}: {fetch_err}"))
        }
    }
}

/// The raw URL of the manual at the versioned tag.
pub fn doc_url(lang: DocLang, hosts: &GithubHosts, repo: &RepoSpec) -> String {
    hosts.raw_file_url(repo, &doc_tag(), &doc_path(lang))
}

fn fetch_tagged_doc(lang: DocLang, hosts: &GithubHosts, repo: &RepoSpec) -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .user_agent(format!(
            "mdbook-plotly/{} (cheat-sheet)",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout_global(Some(FETCH_TIMEOUT))
        .timeout_per_call(Some(FETCH_PER_CALL_TIMEOUT))
        .http_status_as_error(false)
        .build()
        .into();
    let resp = agent
        .get(&doc_url(lang, hosts, repo))
        .call()
        .map_err(|e| e.to_string())?;
    if resp.status() != 200 {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.into_body().read_to_string().map_err(|e| e.to_string())
}

/// Message delivered when the background load finishes.
pub enum CheatMsg {
    Loaded(Result<(UsageDoc, DocSource), String>),
}
