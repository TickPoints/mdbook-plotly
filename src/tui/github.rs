//! GitHub endpoint configuration and URL building.
//!
//! No GitHub URL is hardcoded in the rest of the codebase: every URL is
//! built from a [`GithubHosts`] value, which in turn is resolved from user
//! overrides ([`crate::tui::settings`]) on top of these built-in defaults.
//! That lets users route traffic through a GitHub proxy or mirror.

use crate::tui::settings::GithubOverrides;

/// Default GitHub API base.
pub const DEFAULT_API_BASE: &str = "https://api.github.com";
/// Default release-download base.
pub const DEFAULT_DOWNLOAD_BASE: &str = "https://github.com";

/// Repository identity (`owner` / `name`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSpec {
    pub owner: String,
    pub repo: String,
}

impl RepoSpec {
    /// Derive the repository identity from the package metadata
    /// (`repository = "https://github.com/<owner>/<repo>"`), so it never
    /// drifts from `Cargo.toml`.
    pub fn from_pkg_repository() -> Self {
        let url = env!("CARGO_PKG_REPOSITORY");
        let mut parts = url.trim_end_matches('/').rsplit('/');
        let repo = parts.next().unwrap_or("mdbook-plotly").to_string();
        let owner = parts.next().unwrap_or("TickPoints").to_string();
        Self { owner, repo }
    }

    /// `<owner>/<repo>` path fragment used in API paths.
    pub fn path(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

/// Resolved GitHub endpoints for one run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GithubHosts {
    pub api: String,
    pub download: String,
    /// Optional prefix prepended to every URL (proxy mode).
    pub proxy: Option<String>,
}

impl Default for GithubHosts {
    fn default() -> Self {
        Self {
            api: DEFAULT_API_BASE.to_string(),
            download: DEFAULT_DOWNLOAD_BASE.to_string(),
            proxy: None,
        }
    }
}

impl GithubHosts {
    /// Apply user overrides on top of the defaults.
    pub fn resolve(overrides: &GithubOverrides) -> Self {
        Self {
            api: overrides
                .api
                .clone()
                .unwrap_or_else(|| DEFAULT_API_BASE.to_string()),
            download: overrides
                .download
                .clone()
                .unwrap_or_else(|| DEFAULT_DOWNLOAD_BASE.to_string()),
            proxy: overrides.proxy.clone(),
        }
    }

    fn with_proxy(&self, url: String) -> String {
        match &self.proxy {
            Some(proxy) => format!("{proxy}{url}"),
            None => url,
        }
    }

    fn join(base: &str, path: &str) -> String {
        format!(
            "{}/{}",
            base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// The "latest release" API URL for a repository.
    pub fn api_releases_latest(&self, repo: &RepoSpec) -> String {
        self.with_proxy(Self::join(
            &self.api,
            &format!("repos/{}/releases/latest", repo.path()),
        ))
    }

    /// Rewrite a release asset `browser_download_url` (built on the default
    /// github.com host) through the configured download base and proxy.
    pub fn download_url(&self, browser_url: &str) -> String {
        let url = if self.download != DEFAULT_DOWNLOAD_BASE
            && browser_url.starts_with(DEFAULT_DOWNLOAD_BASE)
        {
            format!(
                "{}{}",
                self.download.trim_end_matches('/'),
                &browser_url[DEFAULT_DOWNLOAD_BASE.len()..]
            )
        } else {
            browser_url.to_string()
        };
        self.with_proxy(url)
    }
}
