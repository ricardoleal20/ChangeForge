# Configuration

Reference for the `changeforge.toml` file written by `changeforge init`.

## Overview

ChangeForge stores its settings in a simple TOML file at your project root: `changeforge.toml`. You can create or update it anytime by running `changeforge init`.

## Example

```
[changeforge]
# Files used to read/write your project version
version_path = ["pyproject.toml"]
# Directory where changesets are saved
changesets_dir = ".changesets"
# Path to your changelog
changelog_path = "CHANGELOG.md"
# Enable AI-assisted messages during 'create'
ai_enabled = false
# Directory with message templates (empty disables)
templates_dir = ""
# Ask to commit the changeset right after creation
commit_on_create = false
```

## Options

- **version_path** (array of strings): files where ChangeForge will extract and update the version. Common values include `pyproject.toml` or `Cargo.toml`. You can provide multiple paths.
- **changesets_dir** (string): directory where interactive changesets are stored as TOML files. Default `.changesets`.
- **changelog_path** (string): path to your `CHANGELOG.md` file.
- **ai_enabled** (bool): when true, the `create` command offers an AI option to generate a message based on detected changes.
- **templates_dir** (string): directory with message templates. When set and non‑empty, `create` offers to pick a template and customize it.
- **commit_on_create** (bool): when true, after saving the changeset ChangeForge will offer to `git add`/`git commit` the file (and the selected module/file, if applicable).

## Notes

The `init` command helps you select _version_path_ files, toggle _ai_enabled_ and _commit_on_create_, set a _templates_dir_, and optionally scaffold GitHub workflows for PRs and Releases.


