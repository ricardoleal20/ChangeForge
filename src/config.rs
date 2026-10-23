use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

fn default_changesets() -> String {
    ".changesets".into()
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, clap::ValueEnum, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Single,
    Monorepo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub name: String,
    pub manifest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_key: Option<String>,
    pub changelog: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_manifests: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub mode: Mode,
    #[serde(default = "default_changesets")]
    pub changesets_dir: String,
    pub packages: Vec<Package>,
}

pub const RELEASE_FILE: &str = ".releaseweave/release.json";

/// Resolve repository-relative paths and reject traversal and symbolic links.
pub fn safe_path(root: &Path, relative: &str) -> Result<PathBuf> {
    ensure!(!relative.trim().is_empty(), "A configured path is empty");
    let mut result = root.to_path_buf();
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(part) => {
                result.push(part);
                if let Ok(meta) = fs::symlink_metadata(&result) {
                    ensure!(
                        !meta.file_type().is_symlink(),
                        "Symbolic links are not supported: {}",
                        result.display()
                    );
                }
            }
            Component::CurDir => (),
            _ => bail!("Path must stay inside the repository: {relative}"),
        }
    }
    ensure!(result != root, "Path cannot be the repository root");
    Ok(result)
}

pub fn load(root: &Path) -> Result<Config> {
    let config = root.join("releaseweave.toml");
    let cfg = if config.exists() {
        toml::from_str::<Config>(&fs::read_to_string(safe_path(root, "releaseweave.toml")?)?)
            .context("Invalid releaseweave.toml")?
    } else {
        load_legacy(root)?
    };
    cfg.validate(root)?;
    Ok(cfg)
}

fn load_legacy(root: &Path) -> Result<Config> {
    let legacy = root.join("changeforge.toml");
    let (text, section) = if legacy.exists() {
        (
            fs::read_to_string(safe_path(root, "changeforge.toml")?)?,
            vec!["changeforge"],
        )
    } else {
        (
            fs::read_to_string(safe_path(root, "pyproject.toml")?)
                .context("No releaseweave.toml found. Run `releaseweave init --yes` first")?,
            vec!["tool", "changeforge"],
        )
    };
    let value: toml::Value = toml::from_str(&text)?;
    let table = section
        .iter()
        .try_fold(&value, |v, key| v.get(key))
        .context("Missing legacy [changeforge] configuration; run `releaseweave init --yes`")?;
    let paths: Vec<String> = table
        .get("version_path")
        .and_then(toml::Value::as_array)
        .context("Legacy config requires version_path")?
        .iter()
        .map(|p| {
            p.as_str()
                .map(str::to_owned)
                .context("version_path must contain strings")
        })
        .collect::<Result<_>>()?;
    let manifest = paths
        .first()
        .context("version_path cannot be empty")?
        .clone();
    let info = crate::manifest::read(root, &manifest, None)?;
    Ok(Config {
        mode: Mode::Single,
        changesets_dir: table
            .get("changesets_dir")
            .and_then(toml::Value::as_str)
            .unwrap_or(".changesets")
            .into(),
        packages: vec![Package {
            name: info.name,
            manifest,
            version_key: None,
            changelog: table
                .get("changelog_path")
                .and_then(toml::Value::as_str)
                .unwrap_or("CHANGELOG.md")
                .into(),
            additional_manifests: paths.into_iter().skip(1).collect(),
        }],
    })
}

pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.@/".contains(c))
        && !name.contains("..")
        && name.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && !part.ends_with('.')
                && !part.ends_with(".lock")
        })
}

impl Config {
    pub fn validate(&self, root: &Path) -> Result<()> {
        ensure!(!self.packages.is_empty(), "Configure at least one package");
        ensure!(
            self.mode != Mode::Single || self.packages.len() == 1,
            "single mode requires exactly one package; use monorepo for multiple packages"
        );
        let changesets = safe_path(root, &self.changesets_dir)?;
        let release_file = safe_path(root, RELEASE_FILE)?;
        let mut names = BTreeSet::new();
        let mut files = BTreeSet::new();
        for protected in ["releaseweave.toml", "changeforge.toml", RELEASE_FILE] {
            files.insert(safe_path(root, protected)?);
        }
        for package in &self.packages {
            ensure!(
                valid_name(&package.name),
                "Invalid package name: {}",
                package.name
            );
            ensure!(
                names.insert(&package.name),
                "Duplicate package name: {}",
                package.name
            );
            for path in std::iter::once(&package.manifest)
                .chain(package.additional_manifests.iter())
                .chain(std::iter::once(&package.changelog))
            {
                let resolved = safe_path(root, path)?;
                ensure!(
                    !resolved.starts_with(root.join(".git"))
                        && !resolved.starts_with(root.join(".releaseweave")),
                    "Cannot use internal repository metadata as an output: {path}"
                );
                ensure!(
                    !resolved.starts_with(&changesets) && !changesets.starts_with(&resolved),
                    "Manifest/changelog overlaps changesets_dir: {path}"
                );
                ensure!(files.insert(resolved), "Configured paths overlap: {path}");
            }
        }
        ensure!(
            !release_file.starts_with(&changesets),
            "changesets_dir overlaps release metadata"
        );
        for protected in [
            "releaseweave.toml",
            "changeforge.toml",
            ".git",
            ".releaseweave",
        ] {
            let protected = safe_path(root, protected)?;
            ensure!(
                !changesets.starts_with(&protected) && !protected.starts_with(&changesets),
                "changesets_dir overlaps protected repository files"
            );
        }
        Ok(())
    }
}

pub fn discover(root: &Path, mode: Option<Mode>) -> Result<Config> {
    fn scan(root: &Path, dir: &Path, paths: &mut Vec<String>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if kind.is_symlink()
                || name.starts_with('.')
                || [
                    "target",
                    "node_modules",
                    "vendor",
                    "dist",
                    "build",
                    "examples",
                    "tests",
                ]
                .contains(&name.as_ref())
            {
                continue;
            }
            let path = entry.path();
            if kind.is_dir() {
                scan(root, &path, paths)?;
            } else if ["Cargo.toml", "package.json", "pyproject.toml"].contains(&name.as_ref()) {
                let relative = path
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/");
                if name == "Cargo.toml" {
                    let value: toml::Value = toml::from_str(&fs::read_to_string(&path)?)
                        .with_context(|| format!("Invalid {relative}"))?;
                    if value.get("package").is_none() {
                        continue;
                    }
                }
                paths.push(relative);
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    scan(root, root, &mut paths)?;
    paths.sort();
    ensure!(
        !paths.is_empty(),
        "No supported manifests found. Add Cargo.toml, package.json or pyproject.toml"
    );
    let selected_mode = mode.unwrap_or(if paths.len() > 1 {
        Mode::Monorepo
    } else {
        Mode::Single
    });
    ensure!(selected_mode != Mode::Single || paths.len() == 1, "Multiple manifests found. Configure synchronized additional_manifests manually, or use --mode monorepo");
    let mut packages = Vec::new();
    for path in paths {
        let info = crate::manifest::read(root, &path, None)?;
        let changelog = if selected_mode == Mode::Single {
            "CHANGELOG.md".into()
        } else {
            Path::new(&path)
                .with_file_name("CHANGELOG.md")
                .to_string_lossy()
                .into_owned()
        };
        packages.push(Package {
            name: info.name,
            manifest: path,
            version_key: None,
            changelog,
            additional_manifests: vec![],
        });
    }
    let cfg = Config {
        mode: selected_mode,
        changesets_dir: default_changesets(),
        packages,
    };
    cfg.validate(root)?;
    Ok(cfg)
}
