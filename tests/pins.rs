//! The upstream pins agree everywhere they are written.
//!
//! The commits the port follows are the submodules under `upstream/`.
//! The same commits and versions are copied into
//! `tests/corpus/versions.json` (which the oracle writes with the corpus),
//! `[package.metadata.ported-from]` in `Cargo.toml`, the README and the
//! latest CHANGELOG entry, and `scripts/package.json` pins the acorn the
//! oracle answers with. The tables copied out of acorn name the release
//! they came from. A repin that moves one and forgets another fails
//! here.
//!
//! The submodule commits are read from git's index, so a repin that has
//! been staged is checked before it is committed, and a checkout without
//! the submodules initialised (as in CI) still has them. Outside a git
//! work tree (a source package) only the files are compared.

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

/// Files holding tables copied out of acorn, which name its version.
const COPIED_FROM_ACORN: &[&str] = &["src/identifier_tables.rs", "src/regexp/properties.rs"];

fn read(root: &Path, file: &str) -> String {
    std::fs::read_to_string(root.join(file)).unwrap_or_else(|e| panic!("read {file}: {e}"))
}

fn in_git_work_tree(root: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .is_ok_and(|out| out.status.success())
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

/// The lines of one `[section]` of a TOML file.
fn toml_section<'a>(toml: &'a str, header: &str) -> Vec<&'a str> {
    toml.lines()
        .skip_while(|l| l.trim() != header)
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .collect()
}

/// The latest entry of the CHANGELOG: from its first `## ` heading to
/// the next.
fn latest_changelog_entry(changelog: &str) -> String {
    changelog
        .lines()
        .skip_while(|l| !l.starts_with("## "))
        .enumerate()
        .take_while(|(i, l)| *i == 0 || !l.starts_with("## "))
        .map(|(_, l)| l)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn pins_agree() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let versions: serde_json::Value =
        serde_json::from_str(&read(root, "tests/corpus/versions.json")).unwrap();
    let cargo = read(root, "Cargo.toml");
    let ported_from = toml_section(&cargo, "[package.metadata.ported-from]");
    let readme = read(root, "README.md");
    let changelog = latest_changelog_entry(&read(root, "CHANGELOG.md"));
    let git = in_git_work_tree(root);

    for &(name, path, cargo_key, repo) in PACKAGES {
        let pin = &versions[name];
        let version = pin["version"].as_str().unwrap();
        let commit = pin["commit"].as_str().unwrap();
        assert!(
            commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()),
            "{name} is pinned by a full commit hash, not {commit:?}"
        );
        assert!(!version.is_empty(), "{name} names a release");

        if git {
            assert_eq!(
                submodule_commit(root, path),
                commit,
                "{path} is not at the commit tests/corpus/versions.json pins for {name}"
            );
        }

        let line = ported_from
            .iter()
            .find(|l| l.starts_with(&format!("{cargo_key} = ")))
            .unwrap_or_else(|| panic!("Cargo.toml's ported-from has no entry for {name}"));
        assert!(
            line.contains(&format!("version = \"{version}\""))
                && line.contains(&format!("commit = \"{commit}\"")),
            "Cargo.toml's ported-from entry for {name} is not {version} at {commit}: {line}"
        );

        let readme_line = format!(
            "- **{name} {version}** ([`{}`](https://github.com/{repo}/commit/{commit}))",
            &commit[..7]
        );
        assert!(
            readme.lines().any(|l| l.trim_end() == readme_line),
            "the README has no line {readme_line}"
        );

        assert!(
            changelog.contains(&format!("{name} {version}"))
                && changelog.contains(&format!("`{commit}`")),
            "the latest CHANGELOG entry does not name {name} {version} at {commit}"
        );
    }

    let acorn = versions["acorn"]["version"].as_str().unwrap();
    for file in COPIED_FROM_ACORN {
        assert!(
            read(root, file).contains(&format!("acorn {acorn}")),
            "{file} does not say it was copied from acorn {acorn}"
        );
    }

    let scripts: serde_json::Value =
        serde_json::from_str(&read(root, "scripts/package.json")).unwrap();
    assert_eq!(
        scripts["dependencies"]["acorn"], versions["acorn"]["version"],
        "scripts/package.json installs a different acorn than the one pinned"
    );
}
