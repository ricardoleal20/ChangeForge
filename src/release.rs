use crate::{
    changeset::{self, Bump},
    config::{safe_path, Config, Mode, RELEASE_FILE},
    manifest,
};
use anyhow::{ensure, Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    pub package: String,
    pub current_version: Version,
    pub next_version: Version,
    pub bump: Bump,
    pub tag: String,
    pub manifest: String,
    pub changelog: String,
    pub notes: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Plan {
    pub schema_version: u32,
    pub mode: Mode,
    pub changesets: Vec<String>,
    pub releases: Vec<Release>,
}

pub fn plan(root: &Path, cfg: &Config) -> Result<Plan> {
    let pending = changeset::read_all(root, cfg)?;
    let mut releases = Vec::new();
    for package in &cfg.packages {
        let manifest = manifest::read(root, &package.manifest, package.version_key.as_deref())?;
        for extra in &package.additional_manifests {
            let extra = manifest::read(root, extra, None)?;
            ensure!(
                extra.version == manifest.version,
                "Synchronized manifests for {} have different versions",
                package.name
            );
        }
        let relevant: Vec<_> = pending
            .iter()
            .filter_map(|p| {
                p.change
                    .releases
                    .get(&package.name)
                    .map(|b| (*b, &p.change.message))
            })
            .collect();
        if let Some(bump) = relevant.iter().map(|(b, _)| *b).max() {
            let next_version = bump.next(&manifest.version)?;
            let tag = match cfg.mode {
                Mode::Single => format!("v{next_version}"),
                Mode::Monorepo => format!("{}/v{next_version}", package.name),
            };
            let notes = format!(
                "## [{next_version}]\n\n{}\n",
                relevant
                    .iter()
                    .map(|(_, message)| format!("- {}\n", message.trim().replace('\n', "\n  ")))
                    .collect::<String>()
            );
            releases.push(Release {
                package: package.name.clone(),
                current_version: manifest.version,
                next_version,
                bump,
                tag,
                manifest: package.manifest.clone(),
                changelog: package.changelog.clone(),
                notes,
            });
        }
    }
    releases.sort_by(|a, b| a.package.cmp(&b.package));
    crate::dependencies::validate(root, cfg, &releases)?;
    Ok(Plan {
        schema_version: 1,
        mode: cfg.mode,
        changesets: pending.into_iter().map(|p| p.path).collect(),
        releases,
    })
}

pub fn bump(root: &Path, cfg: &Config, plan: &Plan) -> Result<()> {
    if plan.releases.is_empty() {
        return Ok(());
    }
    let mut writes = BTreeMap::new();
    for release in &plan.releases {
        let package = cfg
            .packages
            .iter()
            .find(|p| p.name == release.package)
            .context("Package missing from config")?;
        for (path, key) in std::iter::once((&package.manifest, package.version_key.as_deref()))
            .chain(package.additional_manifests.iter().map(|p| (p, None)))
        {
            let manifest = manifest::read(root, path, key)?;
            ensure!(
                manifest.version == release.current_version,
                "Manifest changed since plan was created: {path}"
            );
            writes.insert(
                path.clone(),
                manifest.with_version(&release.next_version)?.into_bytes(),
            );
        }
        let path = safe_path(root, &package.changelog)?;
        let previous = if path.exists() {
            fs::read_to_string(&path)?
        } else {
            "# Changelog\n\n".into()
        };
        let index = previous
            .split_inclusive('\n')
            .scan(0, |offset, line| {
                let pos = *offset;
                *offset += line.len();
                Some((pos, line))
            })
            .find(|(_, line)| line.starts_with("## "))
            .map(|(pos, _)| pos)
            .unwrap_or(previous.len());
        let (header, history) = previous.split_at(index);
        let content = format!("{}\n\n{}{}", header.trim_end(), release.notes, history);
        writes.insert(package.changelog.clone(), content.into_bytes());
    }
    writes.insert(
        RELEASE_FILE.into(),
        format!("{}\n", serde_json::to_string_pretty(plan)?).into_bytes(),
    );
    crate::cargo_lock::update(root, cfg, plan, &mut writes)?;
    crate::transaction::apply(root, writes, plan.changesets.clone())
}
