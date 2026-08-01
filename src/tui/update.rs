//! Self-update logic: GitHub Releases API, asset selection per contract A
//! (`docs/RELEASE.md`), SHA-256 verification, and atomic binary
//! replacement via `self_replace`.
//!
//! All GitHub URLs are built from a [`GithubHosts`] value (see
//! [`crate::tui::github`]) so proxies/mirrors are honoured everywhere.
//! The network/replace path runs on a background thread and reports
//! progress through a channel so the UI never blocks.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use semver::Version;
use ureq::Agent;

use crate::tui::github::{DEFAULT_DOWNLOAD_BASE, GithubHosts, RepoSpec};

/// Env var holding the GitHub token used to raise the API rate limit.
pub const GITHUB_TOKEN_ENV: &str = "GITHUB_TOKEN";
/// Overall timeout for update network operations.
pub const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30);
/// Per-call timeout.
pub const PER_CALL_TIMEOUT: Duration = Duration::from_secs(15);
/// Read chunk size while streaming a download.
pub const DOWNLOAD_BUFFER: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// Pure, testable logic
// ---------------------------------------------------------------------------

/// The target triple baked in at build time (CI sets `--target`), falling
/// back to a cfg-derived mapping for local builds.
pub fn target_triple() -> String {
    if let Some(t) = option_env!("CARGO_BUILD_TARGET") {
        return t.to_string();
    }
    cfg_target_triple()
}

fn cfg_target_triple() -> String {
    if cfg!(target_os = "macos") {
        let arch = if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        };
        return format!("{arch}-apple-darwin");
    }
    if cfg!(target_os = "windows") {
        return "x86_64-pc-windows-msvc".to_string();
    }
    if cfg!(target_os = "linux") {
        let arch = if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        };
        let env = if cfg!(target_env = "musl") {
            "musl"
        } else {
            "gnu"
        };
        return format!("{arch}-unknown-linux-{env}");
    }
    "unknown-unknown-unknown".to_string()
}

/// This module only compiles in the full build, so the variant is always
/// `tui`. The slim build has no updater at all.
pub fn variant_suffix() -> &'static str {
    "-tui"
}

pub fn asset_ext() -> &'static str {
    if cfg!(windows) { "zip" } else { "tar.gz" }
}

/// Asset name per `docs/RELEASE.md`:
/// `mdbook-plotly-<version>-<target-triple>[-tui].<ext>`
pub fn expected_asset_name(version: &Version, target: &str) -> String {
    format!(
        "mdbook-plotly-{}-{}{}.{}",
        version,
        target,
        variant_suffix(),
        asset_ext()
    )
}

pub fn expected_checksum_name(asset_name: &str) -> String {
    format!("{asset_name}.sha256")
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: Version,
    pub tag_name: String,
    pub body: String,
    pub assets: Vec<Asset>,
}

impl ReleaseInfo {
    pub fn find_asset(&self, name: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.name == name)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("network error: {0}")]
    Network(String),
    #[error("github api returned http {0}")]
    Http(u16),
    #[error(
        "github api rate limit exceeded (60 requests/hour unauthenticated). \
         set the {GITHUB_TOKEN_ENV} environment variable to raise the limit and retry."
    )]
    RateLimited,
    #[error("unexpected response from github: {0}")]
    InvalidResponse(String),
    #[error(
        "no release asset matching '{expected}' was found for this platform.\n\
         check the release assets on {releases_url}"
    )]
    NoLatestAsset {
        expected: String,
        releases_url: String,
    },
    #[error("no checksum asset '{expected}' was found for this release.")]
    NoChecksumAsset { expected: String },
    #[error(
        "sha-256 verification failed.\n  expected: {expected}\n  actual:   {actual}\n\
         the download may be corrupt or tampered with; refusing to replace the binary."
    )]
    ChecksumMismatch { expected: String, actual: String },
    #[error("i/o error: {0}")]
    Io(String),
    #[error("invalid version number: {0}")]
    NotANumber(String),
}

pub fn parse_version(tag: &str) -> Result<Version, UpdateError> {
    let v = tag.trim().trim_start_matches('v');
    Version::parse(v).map_err(|e| UpdateError::NotANumber(e.to_string()))
}

/// Parse a GitHub "latest release" JSON payload into a `ReleaseInfo`.
pub fn parse_release(body: &str) -> Result<ReleaseInfo, UpdateError> {
    #[derive(serde::Deserialize)]
    struct Raw {
        tag_name: String,
        #[serde(default)]
        body: String,
        #[serde(default)]
        assets: Vec<RawAsset>,
    }
    #[derive(serde::Deserialize)]
    struct RawAsset {
        name: String,
        #[serde(rename = "browser_download_url")]
        url: String,
    }
    let raw: Raw =
        serde_json::from_str(body).map_err(|e| UpdateError::InvalidResponse(e.to_string()))?;
    Ok(ReleaseInfo {
        version: parse_version(&raw.tag_name)?,
        tag_name: raw.tag_name,
        body: raw.body,
        assets: raw
            .assets
            .into_iter()
            .map(|a| Asset {
                name: a.name,
                url: a.url,
            })
            .collect(),
    })
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    hex(&h.finalize())
}

/// Expected first whitespace-separated token of a `.sha256` file.
pub fn parse_checksum(checksum_file: &str) -> Option<String> {
    checksum_file
        .split_whitespace()
        .next()
        .map(|s| s.to_lowercase())
        .filter(|s| !s.is_empty())
}

/// Read the checksum asset text and compare against the archive.
pub fn verify_sha256(archive: &Path, checksum_text: &str) -> Result<String, UpdateError> {
    let expected = parse_checksum(checksum_text)
        .ok_or_else(|| UpdateError::InvalidResponse("empty checksum file".into()))?;
    let bytes = std::fs::read(archive).map_err(|e| UpdateError::Io(e.to_string()))?;
    let actual = sha256_hex(&bytes);
    if actual == expected {
        Ok(actual)
    } else {
        Err(UpdateError::ChecksumMismatch { expected, actual })
    }
}

/// Is `latest` a newer release than the running binary's version?
pub fn is_newer(latest: &Version, current: &Version) -> bool {
    latest > current
}

// ---------------------------------------------------------------------------
// Network
// ---------------------------------------------------------------------------

fn build_agent() -> Agent {
    Agent::config_builder()
        .user_agent(format!(
            "mdbook-plotly/{} (self-update)",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout_global(Some(DOWNLOAD_TIMEOUT))
        .timeout_per_call(Some(PER_CALL_TIMEOUT))
        .http_status_as_error(false)
        .build()
        .into()
}

fn github_token() -> Option<String> {
    std::env::var(GITHUB_TOKEN_ENV)
        .ok()
        .filter(|t| !t.is_empty())
}

fn rate_limit_remaining(headers: &ureq::http::HeaderMap) -> Option<u64> {
    headers
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
}

fn map_http(status: u16, headers: &ureq::http::HeaderMap) -> UpdateError {
    if status == 403 && rate_limit_remaining(headers) == Some(0) {
        return UpdateError::RateLimited;
    }
    UpdateError::Http(status)
}

/// Fetch the "latest release" for the repo.
pub fn fetch_latest_release(
    hosts: &GithubHosts,
    repo: &RepoSpec,
) -> Result<ReleaseInfo, UpdateError> {
    let agent = build_agent();
    let mut req = agent
        .get(hosts.api_releases_latest(repo))
        .header("Accept", "application/vnd.github+json");
    if let Some(token) = github_token() {
        req = req.header("Authorization", format!("Bearer {token}"));
    }
    let resp = req
        .call()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    let status = resp.status();
    if status != 200 {
        return Err(map_http(status.into(), resp.headers()));
    }
    let body = resp
        .into_body()
        .read_to_string()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    parse_release(&body)
}

/// Download an asset to `dest`, reporting progress over `tx`.
pub fn download_asset(
    hosts: &GithubHosts,
    url: &str,
    dest: &mut File,
    tx: &Sender<UpdateMsg>,
) -> Result<Option<u64>, UpdateError> {
    let agent = build_agent();
    let mut resp = agent
        .get(hosts.download_url(url))
        .call()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    if resp.status() != 200 {
        return Err(map_http(resp.status().into(), resp.headers()));
    }
    let total = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    let mut reader = resp.body_mut().as_reader();
    let mut buf = [0u8; DOWNLOAD_BUFFER];
    let mut downloaded = 0u64;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| UpdateError::Io(e.to_string()))?;
        if n == 0 {
            break;
        }
        dest.write_all(&buf[..n])
            .map_err(|e| UpdateError::Io(e.to_string()))?;
        downloaded += n as u64;
        let _ = tx.send(UpdateMsg::DownloadProgress {
            downloaded,
            total: total.unwrap_or(0),
        });
    }
    Ok(total)
}

fn fetch_checksum(hosts: &GithubHosts, asset: &Asset) -> Result<String, UpdateError> {
    let agent = build_agent();
    let resp = agent
        .get(hosts.download_url(&asset.url))
        .call()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    if resp.status() != 200 {
        return Err(map_http(resp.status().into(), resp.headers()));
    }
    resp.into_body()
        .read_to_string()
        .map_err(|e| UpdateError::Network(e.to_string()))
}

// ---------------------------------------------------------------------------
// Worker + messages
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum UpdateMsg {
    CheckStarted,
    Available {
        release: ReleaseInfo,
        current: String,
        dry_run: bool,
    },
    AlreadyLatest {
        current: String,
    },
    DownloadProgress {
        downloaded: u64,
        total: u64,
    },
    Downloaded {
        archive: PathBuf,
    },
    Verified {
        sha: String,
    },
    WaitingConfirm {
        archive: PathBuf,
    },
    Replaced,
    Failed(String),
}

/// Run the whole update flow on a background thread.
///
/// - Checks the latest release.
/// - If a newer version exists, downloads + verifies the archive for this
///   variant/platform, then blocks on `confirm_rx` for user confirmation
///   before replacing the running binary.
/// - `dry_run` stops after reporting what would happen.
pub fn spawn_update_worker(
    dry_run: bool,
    cache_dir: &Path,
    hosts: GithubHosts,
    repo: RepoSpec,
    tx: Sender<UpdateMsg>,
    confirm_rx: Receiver<bool>,
) {
    let cache_dir = cache_dir.to_path_buf();
    std::thread::spawn(move || {
        let _ = tx.send(UpdateMsg::CheckStarted);
        let current = env!("CARGO_PKG_VERSION").to_string();
        let current_version = match Version::parse(&current) {
            Ok(v) => v,
            Err(e) => {
                let _ = tx.send(UpdateMsg::Failed(format!(
                    "current version '{current}' is not semver: {e}"
                )));
                return;
            }
        };

        let release = match fetch_latest_release(&hosts, &repo) {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.send(UpdateMsg::Failed(e.to_string()));
                return;
            }
        };

        if !is_newer(&release.version, &current_version) {
            let _ = tx.send(UpdateMsg::AlreadyLatest { current });
            return;
        }

        let target = target_triple();
        let asset_name = expected_asset_name(&release.version, &target);
        let checksum_name = expected_checksum_name(&asset_name);

        let asset = match release.find_asset(&asset_name) {
            Some(a) => a,
            None => {
                let releases_url = hosts
                    .download_url(&format!("{DEFAULT_DOWNLOAD_BASE}/{}/releases", repo.path()));
                let _ = tx.send(UpdateMsg::Failed(
                    UpdateError::NoLatestAsset {
                        expected: asset_name.clone(),
                        releases_url,
                    }
                    .to_string(),
                ));
                return;
            }
        };
        let checksum_asset = match release.find_asset(&checksum_name) {
            Some(a) => a,
            None => {
                let _ = tx.send(UpdateMsg::Failed(
                    UpdateError::NoChecksumAsset {
                        expected: checksum_name,
                    }
                    .to_string(),
                ));
                return;
            }
        };

        let _ = tx.send(UpdateMsg::Available {
            release: release.clone(),
            current,
            dry_run,
        });
        if dry_run {
            return;
        }

        // Download the archive into the cache dir.
        std::fs::create_dir_all(&cache_dir).ok();
        let archive_path = cache_dir.join(&asset_name);
        let mut archive_file = match File::create(&archive_path) {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(UpdateMsg::Failed(format!(
                    "cannot create download file: {e}"
                )));
                return;
            }
        };
        match download_asset(&hosts, &asset.url, &mut archive_file, &tx) {
            Ok(_) => {}
            Err(e) => {
                let _ = std::fs::remove_file(&archive_path);
                let _ = tx.send(UpdateMsg::Failed(e.to_string()));
                return;
            }
        }
        let _ = tx.send(UpdateMsg::Downloaded {
            archive: archive_path.clone(),
        });

        // Fetch and verify the checksum.
        let checksum_text = match fetch_checksum(&hosts, checksum_asset) {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(UpdateMsg::Failed(e.to_string()));
                return;
            }
        };
        let sha = match verify_sha256(&archive_path, &checksum_text) {
            Ok(s) => s,
            Err(e) => {
                let _ = tx.send(UpdateMsg::Failed(e.to_string()));
                return;
            }
        };
        let _ = tx.send(UpdateMsg::Verified { sha: sha.clone() });

        // Wait for the user to confirm before touching the running binary.
        let _ = tx.send(UpdateMsg::WaitingConfirm {
            archive: archive_path.clone(),
        });
        match confirm_rx.recv() {
            Ok(true) => {
                if let Err(e) = replace_binary(&hosts, &repo, &archive_path) {
                    let _ = tx.send(UpdateMsg::Failed(e));
                } else {
                    let _ = tx.send(UpdateMsg::Replaced);
                }
            }
            _ => {
                let _ = tx.send(UpdateMsg::Failed("update cancelled.".into()));
            }
        }
    });
}

/// Replace the running binary with the freshly downloaded one.
fn replace_binary(hosts: &GithubHosts, repo: &RepoSpec, archive_path: &Path) -> Result<(), String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("cannot locate the current binary: {e}"))?;

    // Extract the archive to a staging directory first, then hand the real
    // binary to self_replace.
    let stage_dir = archive_path.with_extension("staging");
    let _ = std::fs::remove_dir_all(&stage_dir);
    std::fs::create_dir_all(&stage_dir).map_err(|e| format!("cannot create staging dir: {e}"))?;

    let extracted = if cfg!(windows) {
        extract_zip(archive_path, &stage_dir).map_err(|e| format!("cannot extract archive: {e}"))?
    } else {
        extract_tar_gz(archive_path, &stage_dir)
            .map_err(|e| format!("cannot extract archive: {e}"))?
    };

    if let Err(e) = self_replace::self_replace(&extracted) {
        let releases_url =
            hosts.download_url(&format!("{DEFAULT_DOWNLOAD_BASE}/{}/releases", repo.path()));
        let manual = format!(
            "automatic replacement failed: {e}\n\n\
             manual upgrade: download the matching asset from\n  {releases_url},\n\
             extract it, and copy it over {} yourself.",
            current_exe.display()
        );
        return Err(manual);
    }
    let _ = std::fs::remove_dir_all(&stage_dir);
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<PathBuf, String> {
    // Avoid a `tar`/`flate2` dependency: shell out to system `tar`, which
    // is present on every Unix platform we ship for.
    let status = std::process::Command::new("tar")
        .args(["-xzf"])
        .arg(archive)
        .arg("-C")
        .arg(dest)
        .status()
        .map_err(|e| format!("tar: {e}"))?;
    if !status.success() {
        return Err(format!("tar exited with status {status}"));
    }
    let bin = dest.join(if cfg!(windows) {
        "mdbook-plotly.exe"
    } else {
        "mdbook-plotly"
    });
    if !bin.exists() {
        return Err(format!(
            "extracted archive does not contain the binary at {}",
            bin.display()
        ));
    }
    Ok(bin)
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<PathBuf, String> {
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "Expand-Archive"])
        .arg(archive)
        .arg("-DestinationPath")
        .arg(dest)
        .status()
        .map_err(|e| format!("powershell: {e}"))?;
    if !status.success() {
        return Err(format!("Expand-Archive exited with status {status}"));
    }
    let bin = dest.join("mdbook-plotly.exe");
    if !bin.exists() {
        return Err("extracted archive does not contain mdbook-plotly.exe".to_string());
    }
    Ok(bin)
}
