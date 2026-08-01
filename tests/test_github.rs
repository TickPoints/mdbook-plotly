//! Tests for the GitHub hosts/URL configuration and proxy support.

#![cfg(feature = "tui")]

use mdbook_plotly::tui::github::{DEFAULT_API_BASE, DEFAULT_DOWNLOAD_BASE, GithubHosts, RepoSpec};
use mdbook_plotly::tui::settings::GithubOverrides;

fn overrides(proxy: Option<&str>, api: Option<&str>, download: Option<&str>) -> GithubOverrides {
    GithubOverrides {
        proxy: proxy.map(String::from),
        api: api.map(String::from),
        download: download.map(String::from),
    }
}

#[test]
fn defaults_are_the_public_github_endpoints() {
    let hosts = GithubHosts::resolve(&GithubOverrides::default());
    assert_eq!(hosts.api, DEFAULT_API_BASE);
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
fn download_url_rewrites_github_com_host() {
    let hosts = GithubHosts::resolve(&overrides(None, None, Some("https://mirror.example.com")));
    assert_eq!(
        hosts.download_url("https://github.com/owner/repo/releases/download/v1/a.zip"),
        "https://mirror.example.com/owner/repo/releases/download/v1/a.zip"
    );
}

#[test]
fn proxy_prefix_is_prepended_everywhere() {
    let hosts = GithubHosts::resolve(&overrides(Some("https://ghproxy.com/"), None, None));
    let repo = RepoSpec::from_pkg_repository();
    assert!(
        hosts
            .api_releases_latest(&repo)
            .starts_with("https://ghproxy.com/")
    );
    assert!(
        hosts
            .download_url("https://github.com/a/b")
            .starts_with("https://ghproxy.com/")
    );
}

#[test]
fn custom_api_base_replaces_host() {
    let hosts = GithubHosts::resolve(&overrides(None, Some("https://api.example.com"), None));
    let repo = RepoSpec::from_pkg_repository();
    assert!(
        hosts
            .api_releases_latest(&repo)
            .starts_with("https://api.example.com/")
    );
    // Download base untouched.
    assert_eq!(
        hosts.download_url("https://github.com/a/b"),
        "https://github.com/a/b"
    );
}
