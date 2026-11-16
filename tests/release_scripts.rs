//! Purpose: behavioral tests for the release channel scripts invoked by
//! `.github/workflows/release.yml`.
//! Responsibilities: prove the Homebrew formula and Scoop manifest are rendered
//! from the release `checksums.txt`, that a missing platform digest aborts
//! without writing, and that re-rendering is byte-identical (retry safety).
//! Rationale: release.yml only runs on a `v*` tag, so these scripts are the
//! only release-path logic testable before a tag exists.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const VERSION: &str = "0.1.0";
const TAG: &str = "v0.1.0";

const DIGESTS: &[(&str, &str)] = &[
    ("kaya_0.1.0_linux_amd64.tar.gz", "1111"),
    ("kaya_0.1.0_linux_arm64.tar.gz", "2222"),
    ("kaya_0.1.0_darwin_amd64.tar.gz", "3333"),
    ("kaya_0.1.0_darwin_arm64.tar.gz", "4444"),
    ("kaya_0.1.0_windows_amd64.zip", "5555"),
];

fn sha(fill: &str) -> String {
    fill.repeat(64 / fill.len())
}

fn checksums(entries: &[(&str, &str)]) -> String {
    entries
        .iter()
        .map(|(name, fill)| format!("{}  {}\n", sha(fill), name))
        .collect()
}

fn tempdir() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("kaya-release-scripts-")
        .tempdir()
        .unwrap()
}

fn write_checksums(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("checksums.txt");
    std::fs::write(&path, body).unwrap();
    path
}

fn run_script(script: &str, out: &Path, checksums: &Path) -> Output {
    run_script_with_tag(script, out, checksums, TAG)
}

fn run_script_with_tag(script: &str, out: &Path, checksums: &Path, tag: &str) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join(script);
    Command::new("python3")
        .arg(script)
        .arg(out)
        .arg(VERSION)
        .arg(tag)
        .arg(checksums)
        .output()
        .expect("python3 must be available to run the release scripts")
}

#[test]
fn homebrew_renders_formula_from_checksums() {
    let tmp = tempdir();
    let dir = tmp.path();
    let checksums_path = write_checksums(dir, &checksums(DIGESTS));
    let formula = dir.join("Formula/kaya.rb");

    let out = run_script("update-homebrew.py", &formula, &checksums_path);
    assert!(
        out.status.success(),
        "update-homebrew.py failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let body = std::fs::read_to_string(&formula).unwrap();
    assert!(body.contains("class Kaya < Formula"), "{body}");
    assert!(body.contains(&format!("version \"{VERSION}\"")), "{body}");
    assert!(body.contains("license \"Apache-2.0\""), "{body}");
    assert!(body.contains("bin.install \"kaya\""), "{body}");
    assert!(
        body.contains(&format!(
            "releases/download/{TAG}/kaya_{VERSION}_linux_arm64.tar.gz"
        )),
        "{body}"
    );
    // Each digest must sit directly under its own platform URL: a transposed
    // mapping passes a plain contains-check but fails `brew install` at the
    // checksum. Indentation differs per block (the linux arm block nests an
    // `if Hardware::CPU.is_64_bit?`), so compare the next non-empty line.
    for (name, fill) in DIGESTS.iter().filter(|(name, _)| name.ends_with(".tar.gz")) {
        let at = body
            .find(name)
            .unwrap_or_else(|| panic!("formula has no url for {name}"));
        let sha_line = body[at..].lines().nth(1).unwrap_or_default().trim();
        assert_eq!(
            sha_line,
            format!("sha256 \"{}\"", sha(fill)),
            "url/digest mismatch for {name}"
        );
    }
}

#[test]
fn homebrew_rejects_missing_platform() {
    let tmp = tempdir();
    let dir = tmp.path();
    let partial: Vec<_> = DIGESTS
        .iter()
        .filter(|(name, _)| !name.contains("darwin_arm64"))
        .copied()
        .collect();
    let checksums_path = write_checksums(dir, &checksums(&partial));
    let formula = dir.join("Formula/kaya.rb");

    let out = run_script("update-homebrew.py", &formula, &checksums_path);
    assert!(!out.status.success(), "a partial checksum file must abort");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("darwin_arm64"), "stderr was: {stderr}");
    assert!(!formula.exists(), "no formula may be written on failure");
}

#[test]
fn scoop_renders_manifest_from_checksums() {
    let tmp = tempdir();
    let dir = tmp.path();
    let checksums_path = write_checksums(dir, &checksums(DIGESTS));
    let manifest = dir.join("bucket/kaya.json");

    let out = run_script("update-scoop.py", &manifest, &checksums_path);
    assert!(
        out.status.success(),
        "update-scoop.py failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let body: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest).unwrap()).unwrap();
    assert_eq!(body["version"], VERSION);
    assert_eq!(body["license"], "Apache-2.0");
    assert_eq!(body["bin"], "kaya.exe");
    assert_eq!(
        body["url"],
        format!(
            "https://github.com/charly-vibes/kaya/releases/download/{TAG}/kaya_{VERSION}_windows_amd64.zip"
        )
    );
    assert_eq!(body["hash"], format!("sha256:{}", sha("5555")));
    // checkver-driven autoupdate bumps `version`; a literal URL would keep
    // pointing at the released asset. The bucket's house style is templated.
    assert_eq!(
        body["autoupdate"]["url"],
        "https://github.com/charly-vibes/kaya/releases/download/v$version/kaya_$version_windows_amd64.zip"
    );
}

#[test]
fn scoop_rejects_missing_windows_checksum() {
    let tmp = tempdir();
    let dir = tmp.path();
    let partial: Vec<_> = DIGESTS
        .iter()
        .filter(|(name, _)| !name.contains("windows_amd64"))
        .copied()
        .collect();
    let checksums_path = write_checksums(dir, &checksums(&partial));
    let manifest = dir.join("bucket/kaya.json");

    let out = run_script("update-scoop.py", &manifest, &checksums_path);
    assert!(
        !out.status.success(),
        "a checksum file without windows must abort"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("windows_amd64"),
        "stderr was: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!manifest.exists(), "no manifest may be written on failure");
}

/// `sha256sum ./*.tar.gz ./*.zip > checksums.txt` (the packaging step) writes
/// entries prefixed with `./`; the scripts must match on the basename.
#[test]
fn scripts_accept_sha256sum_dot_prefix() {
    let tmp = tempdir();
    let dir = tmp.path();
    let prefixed: String = DIGESTS
        .iter()
        .map(|(name, fill)| format!("{}  ./{}\n", sha(fill), name))
        .collect();
    let checksums_path = write_checksums(dir, &prefixed);
    let formula = dir.join("Formula/kaya.rb");
    let manifest = dir.join("bucket/kaya.json");

    for (script, out) in [
        ("update-homebrew.py", &formula),
        ("update-scoop.py", &manifest),
    ] {
        let result = run_script(script, out, &checksums_path);
        assert!(
            result.status.success(),
            "{script} rejected ./ prefixed checksums: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert!(
        std::fs::read_to_string(&formula)
            .unwrap()
            .contains(&sha("4444"))
    );
    assert!(
        std::fs::read_to_string(&manifest)
            .unwrap()
            .contains(&sha("5555"))
    );
}

/// The workflow derives VERSION from TAG, but an out-of-band call must not be
/// able to write a formula/manifest whose URLs point at a tag that does not
/// carry these artifacts.
#[test]
fn both_scripts_reject_tag_version_mismatch() {
    let tmp = tempdir();
    let dir = tmp.path();
    let checksums_path = write_checksums(dir, &checksums(DIGESTS));
    let formula = dir.join("Formula/kaya.rb");
    let manifest = dir.join("bucket/kaya.json");

    for (script, out) in [
        ("update-homebrew.py", &formula),
        ("update-scoop.py", &manifest),
    ] {
        let result = run_script_with_tag(script, out, &checksums_path, "v9.9.9");
        assert!(
            !result.status.success(),
            "{script} accepted a mismatched tag"
        );
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("does not match"),
            "stderr was: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            !out.exists(),
            "{script} wrote output despite a mismatched tag"
        );
    }
}

#[test]
fn both_scripts_rerender_identical_bytes() {
    let tmp = tempdir();
    let dir = tmp.path();
    let checksums_path = write_checksums(dir, &checksums(DIGESTS));
    let formula = dir.join("Formula/kaya.rb");
    let manifest = dir.join("bucket/kaya.json");

    for script in ["update-homebrew.py", "update-scoop.py"] {
        let out = if script == "update-homebrew.py" {
            &formula
        } else {
            &manifest
        };
        assert!(run_script(script, out, &checksums_path).status.success());
        let first = std::fs::read(out).unwrap();
        assert!(run_script(script, out, &checksums_path).status.success());
        assert_eq!(
            first,
            std::fs::read(out).unwrap(),
            "{script} is not idempotent"
        );
    }
}
