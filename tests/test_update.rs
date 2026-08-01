//! Tests for the self-update logic (asset naming, release parsing, sha256,
//! checksum handling). No network is touched.

#![cfg(feature = "tui")]

use mdbook_plotly::tui::update::{
    UpdateError, expected_asset_name, expected_checksum_name, is_newer, parse_checksum,
    parse_release, parse_version, sha256_hex, verify_sha256,
};
use semver::Version;

#[test]
fn asset_names_follow_contract_a() {
    let v = Version::parse("0.3.0").unwrap();
    assert_eq!(
        expected_asset_name(&v, "x86_64-unknown-linux-gnu"),
        "mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz"
    );
    assert_eq!(
        expected_checksum_name("mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz"),
        "mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz.sha256"
    );
}

#[test]
fn parses_latest_release_json() {
    let json = r###"{
        "tag_name": "v0.3.0",
        "body": "## Notes\nSome release notes.",
        "assets": [
            { "name": "mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu.tar.gz", "browser_download_url": "https://example.com/a" },
            { "name": "mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz", "browser_download_url": "https://example.com/b" },
            { "name": "mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz.sha256", "browser_download_url": "https://example.com/b.sha" }
        ]
    }"###;
    let release = parse_release(json).unwrap();
    assert_eq!(release.version, Version::parse("0.3.0").unwrap());
    assert_eq!(release.tag_name, "v0.3.0");
    assert!(release.body.contains("Notes"));
    assert_eq!(release.assets.len(), 3);
    assert!(
        release
            .find_asset("mdbook-plotly-0.3.0-x86_64-unknown-linux-gnu-tui.tar.gz")
            .is_some()
    );
    assert!(release.find_asset("nope").is_none());
}

#[test]
fn newer_detection_respects_semver() {
    let current = Version::parse("0.3.0").unwrap();
    assert!(is_newer(&Version::parse("0.3.1").unwrap(), &current));
    assert!(is_newer(&Version::parse("1.0.0").unwrap(), &current));
    assert!(!is_newer(&Version::parse("0.3.0").unwrap(), &current));
    assert!(!is_newer(&Version::parse("0.2.9").unwrap(), &current));
}

#[test]
fn sha256_hex_matches_known_vector() {
    // SHA-256("abc") is a well-known value.
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn checksum_parsing_is_lenient() {
    assert_eq!(
        parse_checksum(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  file.tar.gz"
        ),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into())
    );
    assert_eq!(
        parse_checksum("  BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD\n"),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into())
    );
    assert_eq!(parse_checksum(""), None);
}

#[test]
fn verify_sha256_ok_and_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let archive = dir.path().join("a.bin");
    std::fs::write(&archive, b"abc").unwrap();
    let good = format!("{}  a.bin", sha256_hex(b"abc"));
    assert_eq!(verify_sha256(&archive, &good).unwrap(), sha256_hex(b"abc"));

    let bad = format!("{}  a.bin", sha256_hex(b"abd"));
    assert!(matches!(
        verify_sha256(&archive, &bad),
        Err(UpdateError::ChecksumMismatch { .. })
    ));
}

#[test]
fn parse_version_handles_v_prefix_and_junk() {
    assert_eq!(
        parse_version("v0.3.0").unwrap(),
        Version::parse("0.3.0").unwrap()
    );
    assert_eq!(
        parse_version("0.3.0").unwrap(),
        Version::parse("0.3.0").unwrap()
    );
    assert!(matches!(
        parse_version("not-a-version"),
        Err(UpdateError::NotANumber(_))
    ));
}
