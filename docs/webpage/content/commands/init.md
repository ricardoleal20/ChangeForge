# changeforge init

Initializes the ChangeForge configuration for the package. This command walks you through an interactive setup and writes a `changeforge.toml` configuration file. It can also scaffold helpful GitHub workflows.

## Usage

```
$ changeforge init
```

## Interactive steps

- Select one or more version files that contain your project version (auto‑detects `pyproject.toml`, `Cargo.toml`; you can add custom paths).
- Choose creation preferences:
  - AI messages: allow generating messages with AI during `create`.
  - Commit after create: ask to commit the changeset and selected files.
- Optionally set a templates directory for custom message templates (created if it doesn't exist).
- Optionally add GitHub workflows:
  - Open PR on push: on pushes to a watched branch (default `bump-new-version`) open a PR into a base branch (default `main`).
  - Auto Release: on pushes to a target branch (default `main`) extract the version and create a GitHub Release from `CHANGELOG.md`.

## Configuration written

The following keys are written to `changeforge.toml` (editable later):

```
[changeforge]
version_path = ["pyproject.toml", "Cargo.toml"]   # one or many files you select
changesets_dir = ".changesets"
changelog_path = "CHANGELOG.md"
ai_enabled = true
templates_dir = ""                                # empty means disabled
commit_on_create = false
```

## Files that may be created

- `changeforge.toml`
- `.github/workflows/open_pr_on_push.yml` (if selected)
- `.github/workflows/auto_release.yml` (if selected)
- Templates folder you specify (if any)


