<p align="center">
    <img src="https://github.com/ricardoleal20/ChangeForge/blob/main/docs/img/logo.png" width="50%" height="50%" />
</p>
<p align="center">
    <b>Tool for teams that manage the creation and modification of the </b>CHANGELOG<b> based on a specified set of changes.</b>
</p>

<!-- ![PyPi version](https://img.shields.io/pypi/v/changeforge?label=PyPi%20version&logo=PyPi&style=for-the-badge)
![Python versions supported](https://img.shields.io/pypi/pyversions/changeforge?label=Python%20Versions%20Supported&logo=Python&style=for-the-badge)
![Deployed](https://img.shields.io/github/actions/workflow/status/ricardoleal20/changeforge/.github/workflows/publish_on_release.yml?branch=main&label=LAST%20VERSION%20DEPLOYED%20%F0%9F%9A%80&logo=Github&style=for-the-badge)
![License](https://img.shields.io/github/license/ricardoleal20/changeforge?color=%23808000&label=%F0%9F%93%84%20LICENSE&style=for-the-badge) -->

## Installation

To install `ChangeForge`, you can do it through pip:

```
pip install changeforge
```

Please consider that it requires `Python >=3.9`

## Quickstart

1) Initialize configuration (once per repo):

```sh
changeforge init
```

Creates `changeforge.toml`, lets you enable 🤖 AI messages and 💾 commit‑after‑create, and optionally generates the CI workflows (`.github/workflows/bump_version.yml`, `.github/workflows/release_on_merge.yml`).

Example `changeforge.toml`:

```toml
[changeforge]
version_path = ["pyproject.toml", "Cargo.toml"]
changesets_dir = ".changesets"
changelog_path = "CHANGELOG.md"
ai_enabled = true
templates_dir = "templates/messages"   # empty to disable
commit_on_create = true
```

2) Create a changeset:

```sh
changeforge create
```

- Select the change type (MAJOR/MINOR/PATCH) and a tag
- Pick a module from Git changes or the filesystem, or type a path
- Changeset message: AI (if `ai_enabled`), a template from `templates_dir` (if any files exist), or manual text
- If `commit_on_create = true`, you'll be prompted to commit the changeset and the selected file

3) View pending changes and next version:

```sh
changeforge list
```

4) Perform the bump (updates the version and `CHANGELOG.md`, clears `.changesets/`):

```sh
changeforge bump
```

For more options:

```sh
changeforge --help
```

## Optional CI

- `bump_version.yml`: automatically creates/updates a bump PR on `bump-new-version` (reads paths from `changeforge.toml`).
- `release_on_merge.yml`: creates a GitHub Release when the bump PR is merged, only if it comes from the configured bump branch.

## Contributing

Everyone can contribute. Before contributing, please read our [code of conduct](CODE_OF_CONDUCT.md).

To contribute to `ChangeForge`, follow these steps:

1. Fork this repository.
2. Create a new branch.
3. Make your changes and commit them.
4. Push your changes to your fork.
5. Create a pull request.

## License

Project Name is released under the [MIT License](LICENSE).

## Inspiration

Inspired by [Changesets](https://github.com/changesets/changesets).
