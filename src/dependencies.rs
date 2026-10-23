use crate::{
    config::{safe_path, Config},
    release::Release,
};
use anyhow::{ensure, Context, Result};
use semver::VersionReq;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

/// Reject unsupported propagation using local dependency paths, not registry package names.
pub fn validate(root: &Path, cfg: &Config, releases: &[Release]) -> Result<()> {
    if releases.is_empty() {
        return Ok(());
    }
    let mut versions = BTreeMap::new();
    let mut queue = BTreeSet::new();
    for package in &cfg.packages {
        for path in std::iter::once(&package.manifest).chain(package.additional_manifests.iter()) {
            if Path::new(path)
                .file_name()
                .is_none_or(|n| n != "Cargo.toml")
            {
                continue;
            }
            let absolute = safe_path(root, path)?;
            if let Some(release) = releases.iter().find(|r| r.package == package.name) {
                versions.insert(absolute.clone(), release.next_version.clone());
            }
            for parent in absolute
                .parent()
                .context("Manifest requires a parent")?
                .ancestors()
                .take_while(|p| p.starts_with(root))
            {
                let candidate = parent.join("Cargo.toml");
                if candidate.exists() {
                    queue.insert(candidate);
                }
            }
        }
    }
    let mut docs = BTreeMap::new();
    while let Some(path) = queue.pop_first() {
        if docs.contains_key(&path) {
            continue;
        }
        safe_path(
            root,
            path.strip_prefix(root)?
                .to_str()
                .context("Non-UTF8 manifest path")?,
        )?;
        let doc: toml::Value = toml::from_str(&fs::read_to_string(&path)?)?;
        let base = path.parent().context("Manifest requires a parent")?;
        if let Some(workspace) = doc.get("workspace") {
            let excludes: Vec<_> = workspace
                .get("exclude")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
                .map(|v| {
                    glob::Pattern::new(v.as_str().context("Workspace exclude must be a string")?)
                        .map_err(Into::into)
                })
                .collect::<Result<_>>()?;
            for member in workspace
                .get("members")
                .and_then(toml::Value::as_array)
                .into_iter()
                .flatten()
            {
                let pattern = base.join(
                    member
                        .as_str()
                        .context("Workspace member must be a string")?,
                );
                for member in glob::glob(pattern.to_str().context("Non-UTF8 workspace member")?)? {
                    let member = member?;
                    if excludes
                        .iter()
                        .any(|p| p.matches_path(member.strip_prefix(base).unwrap_or(&member)))
                    {
                        continue;
                    }
                    let manifest = member
                        .join("Cargo.toml")
                        .canonicalize()
                        .context("Workspace member has no Cargo.toml")?;
                    ensure!(
                        manifest.starts_with(root),
                        "Workspace member escapes --root: {}",
                        manifest.display()
                    );
                    queue.insert(manifest);
                }
            }
        }
        // Cargo also includes in-tree path dependencies as implicit workspace members.
        for spec in dependency_specs(&doc) {
            if let Some(local) = local_manifest(root, base, spec)? {
                queue.insert(local);
            }
        }
        docs.insert(path, doc);
    }
    for (path, doc) in docs {
        let base = path.parent().context("Manifest requires a parent")?;
        for spec in dependency_specs(&doc) {
            let Some(local) = local_manifest(root, base, spec)? else {
                continue;
            };
            let Some(next) = versions.get(&local) else {
                continue;
            };
            if let Some(req) = spec.get("version").and_then(toml::Value::as_str) {
                let req =
                    VersionReq::parse(req).context("Invalid internal dependency requirement")?;
                ensure!(req.matches(next), "Internal dependency on {} in {} requires {req}, but the plan releases {next}. Update the dependency range manually and add a dependent-package changeset before bumping; automatic propagation is not supported", local.display(), path.display());
            }
        }
    }
    Ok(())
}

fn local_manifest(root: &Path, base: &Path, spec: &toml::Value) -> Result<Option<PathBuf>> {
    let Some(path) = spec.get("path").and_then(toml::Value::as_str) else {
        return Ok(None);
    };
    let path = base
        .join(path)
        .join("Cargo.toml")
        .canonicalize()
        .context("Local dependency manifest is missing")?;
    if !path.starts_with(root) {
        return Ok(None);
    }
    Ok(Some(path))
}

fn dependency_specs(value: &toml::Value) -> Vec<&toml::Value> {
    let mut specs = Vec::new();
    if let Some(table) = value.as_table() {
        for (key, value) in table {
            if ["dependencies", "dev-dependencies", "build-dependencies"].contains(&key.as_str()) {
                if let Some(deps) = value.as_table() {
                    specs.extend(deps.values());
                }
            } else {
                specs.extend(dependency_specs(value));
            }
        }
    }
    specs
}
