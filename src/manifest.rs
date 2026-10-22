use anyhow::{bail, ensure, Context, Result};
use semver::Version;
use std::{fs, path::Path};
use toml_edit::{DocumentMut, Item};

pub struct Manifest {
    pub name: String,
    pub version: Version,
    text: String,
    key: Vec<String>,
    json: bool,
}

pub fn read(root: &Path, path: &str, version_key: Option<&str>) -> Result<Manifest> {
    let text = fs::read_to_string(crate::config::safe_path(root, path)?)
        .with_context(|| format!("Cannot read manifest {path}"))?;
    let json = Path::new(path).extension().is_some_and(|e| e == "json");
    if json {
        ensure!(
            version_key.is_none(),
            "JSON manifests use the top-level version field"
        );
        let value: serde_json::Value =
            serde_json::from_str(&text).with_context(|| format!("Invalid JSON in {path}"))?;
        let name = value["name"]
            .as_str()
            .context("Manifest requires name")?
            .into();
        let version = Version::parse(
            value["version"]
                .as_str()
                .context("Manifest requires version")?,
        )?;
        return Ok(Manifest {
            name,
            version,
            text,
            key: vec![],
            json,
        });
    }
    let doc: DocumentMut = text
        .parse()
        .with_context(|| format!("Invalid TOML in {path}"))?;
    let key = if let Some(key) = version_key {
        key.split('.').map(str::to_owned).collect::<Vec<_>>()
    } else if doc.get("package").is_some() {
        vec!["package".into(), "version".into()]
    } else if doc.get("project").and_then(|p| p.get("version")).is_some() {
        vec!["project".into(), "version".into()]
    } else if doc.get("tool").and_then(|p| p.get("poetry")).is_some() {
        vec!["tool".into(), "poetry".into(), "version".into()]
    } else {
        bail!("No supported version field in {path}; set version_key explicitly")
    };
    let item = lookup(&doc, &key)
        .with_context(|| format!("Missing version field {} in {path}", key.join(".")))?;
    let version = Version::parse(item.as_str().with_context(|| format!("Version must be explicit in {path}. Cargo version.workspace inheritance is not supported in this PoC"))?)?;
    let parent = &key[..key.len().saturating_sub(1)];
    let name = lookup(
        &doc,
        &parent
            .iter()
            .cloned()
            .chain(std::iter::once("name".into()))
            .collect::<Vec<_>>(),
    )
    .and_then(Item::as_str)
    .context("Manifest requires a package name")?
    .into();
    Ok(Manifest {
        name,
        version,
        text,
        key,
        json,
    })
}

fn lookup<'a>(doc: &'a DocumentMut, key: &[String]) -> Option<&'a Item> {
    let (first, rest) = key.split_first()?;
    rest.iter()
        .try_fold(doc.get(first)?, |item, key| item.get(key))
}

impl Manifest {
    pub fn with_version(&self, version: &Version) -> Result<String> {
        if self.json {
            let mut value: serde_json::Value = serde_json::from_str(&self.text)?;
            value["version"] = version.to_string().into();
            return Ok(format!("{}\n", serde_json::to_string_pretty(&value)?));
        }
        let mut doc: DocumentMut = self.text.parse()?;
        let (first, rest) = self.key.split_first().context("Version key is empty")?;
        let mut item = doc.get_mut(first).context("Missing version field")?;
        for key in rest {
            item = item.get_mut(key).context("Missing version field")?;
        }
        let decor = item
            .as_value()
            .context("Version must be a string")?
            .decor()
            .clone();
        *item = toml_edit::value(version.to_string());
        if let Some(value) = item.as_value_mut() {
            *value.decor_mut() = decor;
        }
        Ok(doc.to_string())
    }
}
