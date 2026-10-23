use crate::{
    config::{safe_path, Config},
    manifest,
    release::Plan,
};
use anyhow::{ensure, Context, Result};
use std::{collections::BTreeMap, fs, path::Path};
use toml_edit::DocumentMut;

/// Keep local Cargo.lock entries consistent without resolving or changing registry dependencies.
pub fn update(
    root: &Path,
    cfg: &Config,
    plan: &Plan,
    writes: &mut BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    let mut locks: BTreeMap<String, DocumentMut> = BTreeMap::new();
    for release in &plan.releases {
        let package = cfg
            .packages
            .iter()
            .find(|p| p.name == release.package)
            .context("Missing package")?;
        for path in std::iter::once(&package.manifest).chain(package.additional_manifests.iter()) {
            if Path::new(path)
                .file_name()
                .is_none_or(|n| n != "Cargo.toml")
            {
                continue;
            }
            let absolute = safe_path(root, path)?;
            let lock = absolute
                .parent()
                .context("Manifest requires a parent")?
                .ancestors()
                .take_while(|parent| parent.starts_with(root))
                .map(|parent| parent.join("Cargo.lock"))
                .find(|lock| lock.exists());
            let Some(lock) = lock else {
                continue;
            };
            let relative = lock.strip_prefix(root)?.to_string_lossy().into_owned();
            safe_path(root, &relative)?;
            ensure!(
                !writes.contains_key(&relative),
                "Cargo.lock overlaps configured outputs: {relative}"
            );
            if !locks.contains_key(&relative) {
                locks.insert(
                    relative.clone(),
                    fs::read_to_string(&lock)?
                        .parse()
                        .context("Invalid Cargo.lock")?,
                );
            }
            let doc = locks.get_mut(&relative).context("Missing lock document")?;
            let name = manifest::read(root, path, None)?.name;
            let packages = doc
                .get_mut("package")
                .and_then(|p| p.as_array_of_tables_mut())
                .context("Cargo.lock requires [[package]] entries")?;
            for entry in packages.iter_mut() {
                if entry.get("name").and_then(|v| v.as_str()) == Some(&name)
                    && entry.get("source").is_none()
                    && entry.get("version").and_then(|v| v.as_str())
                        == Some(&release.current_version.to_string())
                {
                    entry["version"] = toml_edit::value(release.next_version.to_string());
                }
                if let Some(dependencies) =
                    entry.get_mut("dependencies").and_then(|v| v.as_array_mut())
                {
                    for dependency in dependencies.iter_mut() {
                        if dependency.as_str()
                            == Some(&format!("{name} {}", release.current_version))
                        {
                            *dependency =
                                toml_edit::Value::from(format!("{name} {}", release.next_version));
                        }
                    }
                }
            }
        }
    }
    for (path, doc) in locks {
        writes.insert(path, doc.to_string().into_bytes());
    }
    Ok(())
}
