//! User settings for the TUI tools: language selection and GitHub
//! proxy/mirror overrides, resolved from (highest priority first) the
//! process environment, then the config file, then built-in defaults.
//!
//! Config file location (XDG): `$XDG_CONFIG_HOME/mdbook-plotly/config.toml`
//! (or the OS equivalent). Example:
//!
//! ```toml
//! [language]
//! doc = "zh_CN"   # "zh_CN" or "en"
//!
//! [github]
//! proxy = "https://ghproxy.com/"
//! # api / download overrides are also accepted, e.g.:
//! # download = "https://github.example.com"
//! ```

use std::path::PathBuf;

/// Config file name inside the XDG config directory.
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Environment variable that pins the generator schema language
/// (`zh`, `zh-CN`, `en`, …).
pub const ENV_LANG: &str = "MDBOOK_PLOTLY_LANG";

/// Environment variable holding a proxy prefix prepended to every GitHub
/// URL (e.g. `https://ghproxy.com/`).
pub const ENV_GITHUB_PROXY: &str = "MDBOOK_PLOTLY_GITHUB_PROXY";
/// Environment variable overriding the GitHub API base URL.
pub const ENV_GITHUB_API: &str = "MDBOOK_PLOTLY_GITHUB_API";
/// Environment variable overriding the release-download base URL.
pub const ENV_GITHUB_DOWNLOAD: &str = "MDBOOK_PLOTLY_GITHUB_DOWNLOAD";

/// GitHub host overrides.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GithubOverrides {
    /// Prefix prepended to every GitHub URL.
    pub proxy: Option<String>,
    /// Replacement for the GitHub API base.
    pub api: Option<String>,
    /// Replacement for the release-download base.
    pub download: Option<String>,
}

/// What a parsed config file contains.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileSettings {
    pub github: GithubOverrides,
    /// `[language] doc` value, e.g. `"zh_CN"`.
    pub language: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct FileConfig {
    #[serde(default)]
    github: Option<FileGithub>,
    #[serde(default)]
    language: Option<FileLanguage>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct FileGithub {
    proxy: Option<String>,
    api: Option<String>,
    download: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
struct FileLanguage {
    doc: Option<String>,
}

/// Absolute path of the config file.
pub fn config_file_path() -> PathBuf {
    directories::ProjectDirs::from("", "", "mdbook-plotly")
        .map(|dirs| dirs.config_dir().join(CONFIG_FILE_NAME))
        .unwrap_or_else(|| std::env::temp_dir().join(CONFIG_FILE_NAME))
}

/// Parse a config file's text. Unknown keys are ignored so future config
/// additions do not break older binaries.
pub fn parse_config(text: &str) -> FileSettings {
    let raw: FileConfig = toml::from_str(text).unwrap_or_default();
    let github = raw.github.unwrap_or_default();
    FileSettings {
        github: GithubOverrides {
            proxy: github.proxy,
            api: github.api,
            download: github.download,
        },
        language: raw.language.and_then(|l| l.doc),
    }
}

/// Resolve GitHub overrides: config file, then environment (env wins).
pub fn github_overrides() -> GithubOverrides {
    let mut overrides = file_github_overrides();
    env_or(&mut overrides.proxy, ENV_GITHUB_PROXY);
    env_or(&mut overrides.api, ENV_GITHUB_API);
    env_or(&mut overrides.download, ENV_GITHUB_DOWNLOAD);
    overrides
}

/// The `[language] doc` name from the config file, if any.
pub fn language_override() -> Option<String> {
    file_config().language
}

/// Language name from the `MDBOOK_PLOTLY_LANG` environment variable, if any.
pub fn env_language_override() -> Option<String> {
    std::env::var(ENV_LANG)
        .ok()
        .filter(|v| !v.trim().is_empty())
}

fn env_or(target: &mut Option<String>, key: &str) {
    if let Ok(value) = std::env::var(key) {
        let value = value.trim();
        if !value.is_empty() {
            *target = Some(value.to_string());
        }
    }
}

fn file_github_overrides() -> GithubOverrides {
    file_config().github
}

fn file_config() -> FileSettings {
    let path = config_file_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return FileSettings::default();
    };
    parse_config(&text)
}
