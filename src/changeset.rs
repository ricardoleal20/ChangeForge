use crate::config::{safe_path, Config, Mode};
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Bump {
    Patch,
    Minor,
    Major,
}

impl Bump {
    pub fn next(self, current: &semver::Version) -> Result<semver::Version> {
        ensure!(current.pre.is_empty() && current.build.is_empty(), "Prerelease/build versions require a dedicated release policy; supported versions are stable MAJOR.MINOR.PATCH");
        let mut version = current.clone();
        match self {
            Self::Major => {
                version.major = version
                    .major
                    .checked_add(1)
                    .context("Major version overflow")?;
                version.minor = 0;
                version.patch = 0;
            }
            Self::Minor => {
                version.minor = version
                    .minor
                    .checked_add(1)
                    .context("Minor version overflow")?;
                version.patch = 0;
            }
            Self::Patch => {
                version.patch = version
                    .patch
                    .checked_add(1)
                    .context("Patch version overflow")?
            }
        }
        Ok(version)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Changeset {
    pub message: String,
    pub releases: BTreeMap<String, Bump>,
}

pub struct Pending {
    pub path: String,
    pub change: Changeset,
}

impl Changeset {
    pub fn validate(&self, cfg: &Config) -> Result<()> {
        ensure!(
            !self.message.trim().is_empty(),
            "Changeset message must not be empty"
        );
        ensure!(
            !self.releases.is_empty(),
            "Changeset must target at least one package"
        );
        for name in self.releases.keys() {
            ensure!(
                cfg.packages.iter().any(|p| &p.name == name),
                "Unknown package in changeset: {name}"
            );
        }
        Ok(())
    }
}

pub fn read_all(root: &Path, cfg: &Config) -> Result<Vec<Pending>> {
    let dir = safe_path(root, &cfg.changesets_dir)?;
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "toml") {
            ensure!(
                entry.file_type()?.is_file(),
                "Changesets must be regular files: {}",
                path.display()
            );
            paths.push(path);
        }
    }
    paths.sort();
    let mut changes = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path)?;
        let value: toml::Value = toml::from_str(&text)
            .with_context(|| format!("Invalid changeset {}", path.display()))?;
        let change = if value.get("changeset").is_some() {
            parse_legacy(&value, cfg)?
        } else {
            toml::from_str::<Changeset>(&text)
                .with_context(|| format!("Invalid changeset {}", path.display()))?
        };
        change
            .validate(cfg)
            .with_context(|| format!("Invalid changeset {}", path.display()))?;
        changes.push(Pending {
            path: path.strip_prefix(root)?.to_string_lossy().into_owned(),
            change,
        });
    }
    Ok(changes)
}

fn parse_legacy(value: &toml::Value, cfg: &Config) -> Result<Changeset> {
    ensure!(
        cfg.mode == Mode::Single,
        "Legacy changesets have no package targets; migrate them before using monorepo mode"
    );
    let change_type = value
        .get("changeset")
        .and_then(|c| c.get("change_type"))
        .and_then(toml::Value::as_str)
        .context("Legacy changeset requires change_type")?;
    let bump = match change_type.to_ascii_lowercase().as_str() {
        "major" => Bump::Major,
        "minor" => Bump::Minor,
        "patch" => Bump::Patch,
        _ => bail!("Invalid legacy change_type: {change_type}"),
    };
    let message = value
        .get("changes")
        .and_then(|c| c.get("description"))
        .and_then(toml::Value::as_str)
        .context("Legacy changeset requires changes.description")?
        .into();
    Ok(Changeset {
        message,
        releases: BTreeMap::from([(cfg.packages[0].name.clone(), bump)]),
    })
}

pub fn create(
    root: &Path,
    cfg: &Config,
    change: Changeset,
    name: Option<String>,
) -> Result<String> {
    change.validate(cfg)?;
    let name = name.unwrap_or_else(|| {
        format!(
            "change-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        )
    });
    ensure!(
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
        "Changeset name must contain only letters, digits, '-' or '_'"
    );
    let path = format!("{}/{name}.toml", cfg.changesets_dir.trim_end_matches('/'));
    let resolved = safe_path(root, &path)?;
    ensure!(!resolved.exists(), "Changeset already exists: {path}");
    crate::transaction::apply(
        root,
        BTreeMap::from([(path.clone(), toml::to_string_pretty(&change)?.into_bytes())]),
        vec![],
    )?;
    Ok(path)
}
