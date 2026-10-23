use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;

fn cli(root: &Path, args: &[&str]) -> Output {
    Command::new(
        option_env!("CARGO_BIN_EXE_releaseweave").unwrap_or(env!("CARGO_BIN_EXE_changeforge")),
    )
    .arg("--root")
    .arg(root)
    .args(args)
    .output()
    .unwrap()
}
fn ok(root: &Path, args: &[&str]) -> String {
    let out = cli(root, args);
    assert!(
        out.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}
fn write(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}
fn single() -> TempDir {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "Cargo.toml", "# keep this comment\n[package]\nname = \"demo\"\nversion = \"1.2.3\"\n[dependencies]\nother = \"1.2.3\"\n");
    dir
}

#[test]
fn single_repo_bump_preserves_dependency_versions_and_unrelated_files() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    ok(
        root,
        &[
            "create",
            "--bump",
            "minor",
            "--message",
            "Support \"quoted\" messages\nand new lines",
            "--name",
            "feature",
        ],
    );
    write(root, ".changesets/README.md", "Keep me");
    ok(root, &["bump"]);
    let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(manifest.contains("version = \"1.3.0\""));
    assert!(manifest.contains("other = \"1.2.3\""));
    assert!(manifest.contains("# keep this comment"));
    assert!(root.join(".changesets/README.md").exists());
    assert!(!root.join(".changesets/feature.toml").exists());
    assert!(fs::read_to_string(root.join("CHANGELOG.md"))
        .unwrap()
        .contains("Support \"quoted\""));
}

#[test]
fn monorepo_releases_only_selected_packages_at_their_own_versions() {
    let d = TempDir::new().unwrap();
    let root = d.path();
    for (name, version) in [("api", "1.2.3"), ("web", "3.0.0"), ("docs", "0.4.0")] {
        write(
            root,
            &format!("packages/{name}/package.json"),
            &format!(r#"{{"name":"{name}","version":"{version}"}}"#),
        );
    }
    ok(root, &["init", "--yes", "--mode", "monorepo"]);
    ok(
        root,
        &[
            "create",
            "--package",
            "api",
            "--bump",
            "minor",
            "--message",
            "API feature",
        ],
    );
    ok(
        root,
        &[
            "create",
            "--package",
            "web",
            "--bump",
            "patch",
            "--message",
            "Web fix",
        ],
    );
    let plan: serde_json::Value = serde_json::from_str(&ok(root, &["plan", "--json"])).unwrap();
    assert_eq!(plan["releases"].as_array().unwrap().len(), 2);
    ok(root, &["bump"]);
    assert!(fs::read_to_string(root.join("packages/api/package.json"))
        .unwrap()
        .contains("1.3.0"));
    assert!(fs::read_to_string(root.join("packages/web/package.json"))
        .unwrap()
        .contains("3.0.1"));
    assert!(!root.join("packages/docs/CHANGELOG.md").exists());
    assert!(root.join("packages/api/CHANGELOG.md").exists());
}

#[test]
fn dry_run_does_not_change_files_and_empty_bump_is_a_noop() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    ok(root, &["bump"]);
    ok(
        root,
        &[
            "create",
            "--bump",
            "major",
            "--message",
            "Breaking change",
            "--name",
            "breaking",
        ],
    );
    let before = fs::read(root.join("Cargo.toml")).unwrap();
    ok(root, &["bump", "--dry-run"]);
    assert_eq!(fs::read(root.join("Cargo.toml")).unwrap(), before);
    assert!(root.join(".changesets/breaking.toml").exists());
    assert!(!root.join(".releaseweave/release.json").exists());
}

#[test]
fn malformed_changeset_blocks_the_entire_bump() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    ok(
        root,
        &[
            "create",
            "--bump",
            "patch",
            "--message",
            "Fix",
            "--name",
            "valid",
        ],
    );
    write(root, ".changesets/broken.toml", "this is not TOML");
    assert!(!cli(root, &["bump"]).status.success());
    assert!(fs::read_to_string(root.join("Cargo.toml"))
        .unwrap()
        .contains("version = \"1.2.3\""));
    assert!(root.join(".changesets/valid.toml").exists());
}

#[test]
fn changeset_names_cannot_escape_directory_or_overwrite_existing_files() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    assert!(!cli(
        root,
        &[
            "create",
            "--bump",
            "patch",
            "--message",
            "Fix",
            "--name",
            "../escape"
        ]
    )
    .status
    .success());
    ok(
        root,
        &[
            "create",
            "--bump",
            "patch",
            "--message",
            "Fix",
            "--name",
            "same",
        ],
    );
    assert!(!cli(
        root,
        &[
            "create",
            "--bump",
            "patch",
            "--message",
            "Other",
            "--name",
            "same"
        ]
    )
    .status
    .success());
}

#[test]
fn highest_bump_wins_without_incrementing_once_per_changeset() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    for kind in ["patch", "minor", "major", "patch"] {
        ok(root, &["create", "--bump", kind, "--message", kind]);
    }
    let plan: serde_json::Value = serde_json::from_str(&ok(root, &["plan", "--json"])).unwrap();
    assert_eq!(plan["releases"][0]["next_version"], "2.0.0");
    assert_eq!(plan["releases"][0]["tag"], "v2.0.0");
    assert_eq!(plan["changesets"].as_array().unwrap().len(), 4);
}

#[test]
fn legacy_config_uses_custom_paths_and_recomputes_stale_versions() {
    let d = single();
    let root = d.path();
    write(root, "changeforge.toml", "[changeforge]\nversion_path = [\"Cargo.toml\"]\nchangesets_dir = \"notes/pending\"\nchangelog_path = \"docs/HISTORY.md\"\n");
    write(root, "notes/pending/legacy.toml", "[changeset]\nchange_type = \"PATCH\"\ntag = \"Bug\"\nversion = \"0.0.1\"\n[changes]\nmodules = [\"src\"]\ndescription = \"Legacy fix\"\n");
    ok(root, &["bump"]);
    assert!(fs::read_to_string(root.join("Cargo.toml"))
        .unwrap()
        .contains("version = \"1.2.4\""));
    assert!(root.join("docs/HISTORY.md").exists());
    assert!(!root.join("CHANGELOG.md").exists());
    assert!(!root.join("notes/pending/legacy.toml").exists());
}

#[test]
fn custom_changeset_paths_are_used_by_create_plan_and_bump() {
    let d = single();
    let root = d.path();
    write(root, "releaseweave.toml", "mode = \"single\"\nchangesets_dir = \"notes/pending\"\n[[packages]]\nname = \"demo\"\nmanifest = \"Cargo.toml\"\nchangelog = \"docs/HISTORY.md\"\n");
    ok(
        root,
        &[
            "create",
            "--bump",
            "patch",
            "--message",
            "Custom paths",
            "--name",
            "custom",
        ],
    );
    assert!(root.join("notes/pending/custom.toml").exists());
    assert!(ok(root, &["plan"]).contains("1.2.4"));
    ok(root, &["bump"]);
    assert!(root.join("docs/HISTORY.md").exists());
}

#[test]
fn missing_changesets_directory_and_repeated_bump_are_safe() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    fs::remove_dir(root.join(".changesets")).unwrap();
    assert!(ok(root, &["bump"]).contains("Nothing to release"));
    ok(root, &["create", "--bump", "patch", "--message", "Fix"]);
    ok(root, &["bump"]);
    let before = fs::read(root.join(".releaseweave/release.json")).unwrap();
    ok(root, &["bump"]);
    assert_eq!(
        before,
        fs::read(root.join(".releaseweave/release.json")).unwrap()
    );
}

#[test]
fn unknown_packages_fail_without_writing_a_changeset() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    assert!(!cli(
        root,
        &[
            "create",
            "--package",
            "missing",
            "--bump",
            "patch",
            "--message",
            "Fix"
        ]
    )
    .status
    .success());
    assert_eq!(fs::read_dir(root.join(".changesets")).unwrap().count(), 0);
}

#[test]
fn invalid_output_path_leaves_versions_and_changesets_intact() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    ok(
        root,
        &[
            "create",
            "--bump",
            "patch",
            "--message",
            "Fix",
            "--name",
            "retained",
        ],
    );
    fs::create_dir(root.join("CHANGELOG.md")).unwrap();
    assert!(!cli(root, &["bump"]).status.success());
    assert!(fs::read_to_string(root.join("Cargo.toml"))
        .unwrap()
        .contains("version = \"1.2.3\""));
    assert!(root.join(".changesets/retained.toml").exists());
}

#[test]
fn python_project_versions_and_multiline_messages_work() {
    let d = TempDir::new().unwrap();
    let root = d.path();
    write(root, "pyproject.toml", "[project]\nname = \"python-demo\"\nversion = \"0.1.0\" # keep\ndependencies = [\"other==0.1.0\"]\n");
    ok(root, &["init", "--yes"]);
    ok(
        root,
        &[
            "create",
            "--bump",
            "minor",
            "--message",
            "Mejora 🐈\nSegunda línea",
        ],
    );
    ok(root, &["bump"]);
    let content = fs::read_to_string(root.join("pyproject.toml")).unwrap();
    assert!(content.contains("version = \"0.2.0\" # keep"));
    assert!(content.contains("other==0.1.0"));
}

#[test]
fn release_preview_exports_per_package_tags_and_rejects_stale_metadata() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    ok(root, &["create", "--bump", "patch", "--message", "Fix"]);
    ok(root, &["bump"]);
    let manifest: serde_json::Value =
        serde_json::from_str(&ok(root, &["release", "--json"])).unwrap();
    assert_eq!(manifest["releases"][0]["tag"], "v1.2.4");
    write(
        root,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"9.0.0\"\n",
    );
    assert!(!cli(root, &["release", "--json"]).status.success());
}

#[test]
fn init_is_idempotent_and_does_not_overwrite_workflows() {
    let d = single();
    let root = d.path();
    write(
        root,
        ".github/workflows/releaseweave-pr.yml",
        "custom workflow",
    );
    ok(root, &["init", "--yes", "--github"]);
    let config = fs::read(root.join("releaseweave.toml")).unwrap();
    ok(root, &["init", "--yes", "--github"]);
    assert_eq!(config, fs::read(root.join("releaseweave.toml")).unwrap());
    assert_eq!(
        fs::read_to_string(root.join(".github/workflows/releaseweave-pr.yml")).unwrap(),
        "custom workflow"
    );
}

#[test]
fn protected_metadata_cannot_be_used_as_a_changelog() {
    let d = single();
    let root = d.path();
    write(root, "releaseweave.toml", "[[packages]]\nname = \"demo\"\nmanifest = \"Cargo.toml\"\nchangelog = \".releaseweave/operation.lock\"\n");
    assert!(!cli(root, &["plan"]).status.success());
}

#[test]
fn crlf_changelog_preserves_unicode_history() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    write(
        root,
        "CHANGELOG.md",
        "# Changelog\r\n\r\n## [1.2.3]\r\n\r\n- 🐈 Historial\r\n",
    );
    ok(root, &["create", "--bump", "patch", "--message", "Fix"]);
    ok(root, &["bump"]);
    let content = fs::read_to_string(root.join("CHANGELOG.md")).unwrap();
    assert!(content.contains("## [1.2.3]\r\n\r\n- 🐈 Historial\r\n"));
    assert!(content.find("## [1.2.4]").unwrap() < content.find("## [1.2.3]").unwrap());
}

#[cfg(unix)]
#[test]
fn symlinked_manifests_are_rejected_without_touching_the_target() {
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    let other = TempDir::new().unwrap();
    write(other.path(), "Cargo.toml", "private content");
    fs::remove_file(root.join("Cargo.toml")).unwrap();
    std::os::unix::fs::symlink(other.path().join("Cargo.toml"), root.join("Cargo.toml")).unwrap();
    assert!(!cli(root, &["bump"]).status.success());
    assert_eq!(
        fs::read_to_string(other.path().join("Cargo.toml")).unwrap(),
        "private content"
    );
}

#[test]
fn cargo_lock_tracks_local_package_bumps_without_changing_registry_entries() {
    let d = single();
    let root = d.path();
    write(root, "Cargo.lock", "version = 3\n\n[[package]]\nname = \"demo\"\nversion = \"1.2.3\"\n\n[[package]]\nname = \"other\"\nversion = \"1.2.3\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n");
    ok(root, &["init", "--yes"]);
    ok(root, &["create", "--bump", "minor", "--message", "Feature"]);
    ok(root, &["bump"]);
    let lock: toml::Value =
        toml::from_str(&fs::read_to_string(root.join("Cargo.lock")).unwrap()).unwrap();
    assert_eq!(lock["package"][0]["version"].as_str(), Some("1.3.0"));
    assert_eq!(lock["package"][1]["version"].as_str(), Some("1.2.3"));
}

#[test]
fn incompatible_internal_cargo_dependencies_block_release_without_consuming_changesets() {
    let d = TempDir::new().unwrap();
    let root = d.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"core\", \"app\"]\n",
    );
    write(
        root,
        "core/Cargo.toml",
        "[package]\nname = \"core-lib\"\nversion = \"1.0.0\"\n",
    );
    write(root, "app/Cargo.toml", "[package]\nname = \"app\"\nversion = \"1.0.0\"\n[dependencies]\ncore = { package = \"core-lib\", path = \"../core\", version = \"^1.0.0\" }\n");
    ok(root, &["init", "--yes", "--mode", "monorepo"]);
    ok(
        root,
        &[
            "create",
            "--package",
            "core-lib",
            "--bump",
            "major",
            "--message",
            "Breaking API",
            "--name",
            "breaking",
        ],
    );
    let output = cli(root, &["bump"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("dependency"));
    assert!(root.join(".changesets/breaking.toml").exists());
    assert!(fs::read_to_string(root.join("core/Cargo.toml"))
        .unwrap()
        .contains("version = \"1.0.0\""));
}

#[test]
fn compatible_internal_cargo_dependency_ranges_allow_minor_releases() {
    let d = TempDir::new().unwrap();
    let root = d.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"core\", \"app\"]\n",
    );
    write(
        root,
        "core/Cargo.toml",
        "[package]\nname = \"core-lib\"\nversion = \"1.0.0\"\n",
    );
    write(root, "app/Cargo.toml", "[package]\nname = \"app\"\nversion = \"1.0.0\"\n[target.'cfg(unix)'.dependencies]\ncore = { package = \"core-lib\", path = \"../core\", version = \"^1.0.0\" }\n");
    ok(root, &["init", "--yes", "--mode", "monorepo"]);
    ok(
        root,
        &[
            "create",
            "--package",
            "core-lib",
            "--bump",
            "minor",
            "--message",
            "Feature",
        ],
    );
    ok(root, &["bump"]);
    assert!(fs::read_to_string(root.join("core/Cargo.toml"))
        .unwrap()
        .contains("version = \"1.1.0\""));
}

#[test]
fn unconfigured_workspace_members_are_checked_for_internal_dependencies() {
    let d = TempDir::new().unwrap();
    let root = d.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/*\"]\n",
    );
    write(
        root,
        "crates/core/Cargo.toml",
        "[package]\nname = \"core-lib\"\nversion = \"1.0.0\"\n",
    );
    write(root, "crates/app/Cargo.toml", "[package]\nname = \"app\"\nversion = \"1.0.0\"\n[dependencies]\ncore = { package = \"core-lib\", path = \"../core\", version = \"^1.0.0\" }\n");
    write(root, "releaseweave.toml", "mode = \"monorepo\"\n[[packages]]\nname = \"core-lib\"\nmanifest = \"crates/core/Cargo.toml\"\nchangelog = \"crates/core/CHANGELOG.md\"\n");
    ok(
        root,
        &[
            "create",
            "--package",
            "core-lib",
            "--bump",
            "major",
            "--message",
            "Breaking",
            "--name",
            "retained",
        ],
    );
    assert!(!cli(root, &["bump"]).status.success());
    assert!(root.join(".changesets/retained.toml").exists());
}

#[test]
fn registry_dependency_with_same_name_is_not_treated_as_a_local_path_dependency() {
    let d = TempDir::new().unwrap();
    let root = d.path();
    write(
        root,
        "Cargo.toml",
        "[workspace]\nmembers = [\"core\", \"app\"]\n",
    );
    write(
        root,
        "core/Cargo.toml",
        "[package]\nname = \"core-lib\"\nversion = \"1.0.0\"\n",
    );
    write(
        root,
        "app/Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"1.0.0\"\n[dependencies]\ncore-lib = \"^1.0.0\"\n",
    );
    ok(root, &["init", "--yes", "--mode", "monorepo"]);
    ok(
        root,
        &[
            "create",
            "--package",
            "core-lib",
            "--bump",
            "major",
            "--message",
            "Breaking",
        ],
    );
    ok(root, &["bump"]);
    assert!(fs::read_to_string(root.join("app/Cargo.toml"))
        .unwrap()
        .contains("core-lib = \"^1.0.0\""));
}

#[test]
fn inherited_workspace_dependency_requirements_are_validated() {
    let d = TempDir::new().unwrap();
    let root = d.path();
    write(root, "Cargo.toml", "[workspace]\nmembers = [\"core\", \"app\"]\n[workspace.dependencies]\ncore = { package = \"core-lib\", path = \"core\", version = \"^1.0.0\" }\n");
    write(
        root,
        "core/Cargo.toml",
        "[package]\nname = \"core-lib\"\nversion = \"1.0.0\"\n",
    );
    write(
        root,
        "app/Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"1.0.0\"\n[dependencies]\ncore.workspace = true\n",
    );
    ok(root, &["init", "--yes", "--mode", "monorepo"]);
    ok(
        root,
        &[
            "create",
            "--package",
            "core-lib",
            "--bump",
            "major",
            "--message",
            "Breaking",
        ],
    );
    assert!(!cli(root, &["plan"]).status.success());
}

#[cfg(unix)]
#[test]
fn cleanup_failure_rolls_back_version_files_and_retains_changesets() {
    use std::os::unix::fs::PermissionsExt;
    let d = single();
    let root = d.path();
    ok(root, &["init", "--yes"]);
    ok(
        root,
        &[
            "create",
            "--bump",
            "patch",
            "--message",
            "Fix",
            "--name",
            "retained",
        ],
    );
    let before = fs::read(root.join("Cargo.toml")).unwrap();
    fs::set_permissions(root.join(".changesets"), fs::Permissions::from_mode(0o555)).unwrap();
    let result = cli(root, &["bump"]);
    fs::set_permissions(root.join(".changesets"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(root.join("Cargo.toml")).unwrap(), before);
    assert!(root.join(".changesets/retained.toml").exists());
    assert!(!root.join(".releaseweave/release.json").exists());
}
