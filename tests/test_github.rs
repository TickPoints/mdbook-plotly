//! Tests for the GitHub hosts/URL configuration and proxy support.

#![cfg(feature = "tui")]

use mdbook_plotly::tui::github::{
    DEFAULT_API_BASE, DEFAULT_DOWNLOAD_BASE, DEFAULT_RAW_BASE, GithubHosts, RepoSpec,
};
use mdbook_plotly::tui::settings::GithubOverrides;

fn overrides(
    proxy: Option<&str>,
    api: Option<&str>,
    raw: Option<&str>,
    download: Option<&str>,
) -> GithubOverrides {
    GithubOverrides {
        proxy: proxy.map(String::from),
        api: api.map(String::from),
        raw: raw.map(String::from),
        download: download.map(String::from),
    }
}

#[test]
fn defaults_are_the_public_github_endpoints() {
    let hosts = GithubHosts::resolve(&GithubOverrides::default());
    assert_eq!(hosts.api, DEFAULT_API_BASE);
    assert_eq!(hosts.raw, DEFAULT_RAW_BASE);
    assert_eq!(hosts.download, DEFAULT_DOWNLOAD_BASE);
    assert_eq!(hosts.proxy, None);
}

#[test]
fn repo_spec_is_derived_from_package_metadata() {
    let repo = RepoSpec::from_pkg_repository();
    assert!(!repo.owner.is_empty());
    assert!(!repo.repo.is_empty());
    assert_eq!(repo.path(), format!("{}/{}", repo.owner, repo.repo));
}

#[test]
fn api_latest_url_uses_configured_base_and_repo() {
    let repo = RepoSpec::from_pkg_repository();
    let hosts = GithubHosts::resolve(&GithubOverrides::default());
    assert_eq!(
        hosts.api_releases_latest(&repo),
        format!("{DEFAULT_API_BASE}/repos/{}/releases/latest", repo.path())
    );
}

#[test]
fn raw_file_url_builds_tagged_path() {
    let repo = RepoSpec::from_pkg_repository();
    let hosts = GithubHosts::resolve(&GithubOverrides::default());
    assert_eq!(
        hosts.raw_file_url(&repo, "v0.3.0", "docs/USAGE.md"),
        format!("{DEFAULT_RAW_BASE}/{}/v0.3.0/docs/USAGE.md", repo.path())
    );
}

#[test]
fn download_url_rewrites_github_com_host() {
    let hosts = GithubHosts::resolve(&overrides(
        None,
        None,
        None,
        Some("https://mirror.example.com"),
    ));
    assert_eq!(
        hosts.download_url("https://github.com/owner/repo/releases/download/v1/a.zip"),
        "https://mirror.example.com/owner/repo/releases/download/v1/a.zip"
    );
}

#[test]
fn proxy_prefix_is_prepended_everywhere() {
    let hosts = GithubHosts::resolve(&overrides(Some("https://ghproxy.com/"), None, None, None));
    let repo = RepoSpec::from_pkg_repository();
    assert!(
        hosts
            .api_releases_latest(&repo)
            .starts_with("https://ghproxy.com/")
    );
    assert!(
        hosts
            .raw_file_url(&repo, "v1", "docs/USAGE.md")
            .starts_with("https://ghproxy.com/")
    );
    assert!(
        hosts
            .download_url("https://github.com/a/b")
            .starts_with("https://ghproxy.com/")
    );
}

#[test]
fn custom_api_and_raw_bases_replace_hosts() {
    let hosts = GithubHosts::resolve(&overrides(
        None,
        Some("https://api.example.com"),
        Some("https://raw.example.com"),
        None,
    ));
    let repo = RepoSpec::from_pkg_repository();
    assert!(
        hosts
            .api_releases_latest(&repo)
            .starts_with("https://api.example.com/")
    );
    assert!(
        hosts
            .raw_file_url(&repo, "v1", "docs/USAGE.md")
            .starts_with("https://raw.example.com/")
    );
    // Download base untouched.
    assert_eq!(
        hosts.download_url("https://github.com/a/b"),
        "https://github.com/a/b"
    );
}
