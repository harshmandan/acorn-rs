//! The upstream pins agree everywhere they are written.
//!
//! The commits the port follows are the submodules under `upstream/`.
//! The same commits and versions are copied into
//! `tests/corpus/versions.json` (which the oracle writes with the corpus),
//! `[package.metadata.ported-from]` in `Cargo.toml` and the README, and
//! `scripts/package.json` pins the acorn the oracle answers with. A repin
//! that moves one and forgets another fails here.
//!
//! The submodule commits are read from git's index, so a repin that has
//! been staged is checked before it is committed, and a checkout without
//! the submodules initialised (as in CI) still has them.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;
use std::process::Command;

/// Each pinned package, with its submodule path, its key in `Cargo.toml`
/// and its repository on GitHub.
const PACKAGES: &[(&str, &str, &str, &str)] = &[
    ("acorn", "upstream/acorn", "acorn", "acornjs/acorn"),
    (
        "@sveltejs/acorn-typescript",
        "upstream/acorn-typescript",
        "\"@sveltejs/acorn-typescript\"",
        "sveltejs/acorn-typescript",
    ),
];

fn read(root: &Path, file: &str) -> String {
    std::fs::read_to_string(root.join(file)).unwrap_or_else(|e| panic!("read {file}: {e}"))
}

/// The commit git records for the submodule at `path`.
fn submodule_commit(root: &Path, path: &str) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--stage", "--", path])
        .output()
        .expect("run git");
    assert!(out.status.success(), "git ls-files failed for {path}");
    let line = String::from_utf8(out.stdout).unwrap();
    let mut fields = line.split_whitespace();
    assert_eq!(
        fields.next(),
        Some("160000"),
        "{path} is not a submodule: {line:?}"
    );
    fields.next().unwrap().to_string()
}

#[test]
fn pins_agree() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let versions: serde_json::Value =
        serde_json::from_str(&read(root, "tests/corpus/versions.json")).unwrap();
    let cargo = read(root, "Cargo.toml");
    let readme = read(root, "README.md");

    for &(name, path, cargo_key, repo) in PACKAGES {
        let pin = &versions[name];
        let version = pin["version"].as_str().unwrap();
        let commit = pin["commit"].as_str().unwrap();

        assert_eq!(
            submodule_commit(root, path),
            commit,
            "{path} is not at the commit tests/corpus/versions.json pins for {name}"
        );

        let line = cargo
            .lines()
            .find(|l| l.starts_with(&format!("{cargo_key} = ")))
            .unwrap_or_else(|| panic!("Cargo.toml has no ported-from entry for {name}"));
        assert!(
            line.contains(&format!("version = \"{version}\""))
                && line.contains(&format!("commit = \"{commit}\"")),
            "Cargo.toml's ported-from entry for {name} is not {version} at {commit}: {line}"
        );

        assert!(
            readme.contains(&format!("**{name} {version}**"))
                && readme.contains(&format!("github.com/{repo}/commit/{commit}")),
            "the README does not name {name} {version} at {commit}"
        );
    }

    let scripts: serde_json::Value =
        serde_json::from_str(&read(root, "scripts/package.json")).unwrap();
    assert_eq!(
        scripts["dependencies"]["acorn"], versions["acorn"]["version"],
        "scripts/package.json installs a different acorn than the one pinned"
    );
}
