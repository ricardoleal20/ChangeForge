use crate::config::safe_path;
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub struct Lock(PathBuf);
impl Lock {
    pub fn acquire(root: &Path) -> Result<Self> {
        let path = safe_path(root, ".releaseweave/operation.lock")?;
        fs::create_dir_all(path.parent().context("Lock requires a parent")?)?;
        fs::OpenOptions::new().write(true).create_new(true).open(&path)
            .context("Another operation is running. If an earlier process crashed, remove .releaseweave/operation.lock and retry")?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn stage(path: &Path, content: &[u8]) -> Result<NamedTempFile> {
    let parent = path.parent().context("File requires a parent")?;
    fs::create_dir_all(parent)?;
    let mut tmp = NamedTempFile::new_in(parent)?;
    if path.exists() {
        tmp.as_file()
            .set_permissions(fs::metadata(path)?.permissions())?;
    }
    tmp.write_all(content)?;
    tmp.as_file().sync_all()?;
    Ok(tmp)
}

/// Prepare every output before replacing files; restore snapshots on ordinary IO failures.
/// This is rollback protection, not a cross-file transaction against power loss or process kill.
pub fn apply(root: &Path, writes: BTreeMap<String, Vec<u8>>, deletes: Vec<String>) -> Result<()> {
    let mut before = BTreeMap::new();
    for relative in writes.keys().chain(deletes.iter()) {
        let path = safe_path(root, relative)?;
        let content = if path.exists() {
            ensure!(
                path.is_file(),
                "Expected a regular file: {}",
                path.display()
            );
            Some(fs::read(&path)?)
        } else {
            None
        };
        before.insert(path, content);
    }
    let mut staged = Vec::new();
    for (relative, content) in &writes {
        let path = safe_path(root, relative)?;
        let tmp = stage(&path, content)?;
        staged.push((path, tmp));
    }
    let result: Result<()> = (|| {
        for (path, tmp) in staged {
            tmp.persist(&path)
                .map_err(|e| e.error)
                .with_context(|| format!("Cannot replace {}", path.display()))?;
        }
        for relative in deletes {
            fs::remove_file(safe_path(root, &relative)?)
                .with_context(|| format!("Cannot consume changeset {relative}"))?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        let mut failures = Vec::new();
        for (path, content) in before {
            let restored = match content {
                Some(content) => {
                    // An unconsumed changeset may be in a read-only directory. Restoring
                    // identical content there is unnecessary and must not stop rollback.
                    if fs::read(&path).ok().as_ref() == Some(&content) {
                        Ok(())
                    } else {
                        stage(&path, &content).and_then(|tmp| {
                            tmp.persist(&path).map(|_| ()).map_err(|e| e.error.into())
                        })
                    }
                }
                None => {
                    if path.exists() {
                        fs::remove_file(&path).map_err(Into::into)
                    } else {
                        Ok(())
                    }
                }
            };
            if let Err(restore_error) = restored {
                failures.push(format!("{}: {restore_error:#}", path.display()));
            }
        }
        if !failures.is_empty() {
            return Err(error.context(format!(
                "Rollback was incomplete; inspect these files: {}",
                failures.join("; ")
            )));
        }
        return Err(error.context("Operation rolled back; changesets retained"));
    }
    Ok(())
}
