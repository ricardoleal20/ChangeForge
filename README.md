# ReleaseWeave

Changeset-driven releases for **single repositories and monorepos**, written in Rust and built for GitHub. Previously called ChangeForge. Inspired by Changesets.

ReleaseWeave records the intent of a change, previews the next versions, then updates only the affected manifests and changelogs. In a monorepo, packages have independent versions and GitHub release tags.

## Run the PoC

```sh
cargo build --release --locked
python3 scripts/demo.py
```

The demo runs the actual CLI on fresh copies under `.demo/`. It verifies dry-run, applies both releases, and leaves manifests, changelogs, a transcript and `report.json` for inspection. Requires Python 3.9+ for the demo runner; the CLI itself only needs Rust to build.

| Scenario | Before | After | Tag |
| --- | --- | --- | --- |
| Single Rust package | hello-release 1.2.3 | 1.3.0 | v1.3.0 |
| Monorepo API | acme-api 1.4.0 | 1.5.0 | acme-api/v1.5.0 |
| Monorepo web | @acme/web 2.1.0 | 2.1.1 | @acme/web/v2.1.1 |
| Monorepo worker | acme-worker 0.7.2 | unchanged | no release |

See [the Spanish PoC walkthrough](docs/POC.md).

## Install from this checkout

```sh
cargo install --path . --locked --bin releaseweave
releaseweave --help
```

No Python, Node, API keys or GitHub login are required to use the CLI locally. The old `changeforge` executable is available through `cargo install --path . --locked --bin changeforge` as a migration alias. The renamed product has not been published to crates.io or PyPI by this change; use this checkout to try it.

## Single repository

```sh
releaseweave init --yes
releaseweave create --bump minor --message "Add configurable greetings"
releaseweave plan
releaseweave bump --dry-run
releaseweave bump
releaseweave release --dry-run
```

`create` without flags opens package, bump and message prompts in a terminal. All commands accept `--root PATH`; they never change your working directory. `add`, `list`, `status` and `version` are aliases for `create`, `plan`, `plan` and `bump`.

```toml
mode = "single"
changesets_dir = ".changesets"

[[packages]]
name = "hello-release"
manifest = "Cargo.toml"
changelog = "CHANGELOG.md"
```

## Monorepo

```sh
releaseweave init --yes --mode monorepo
releaseweave create --package acme-api --bump minor --message "Add health endpoint"
releaseweave create --package @acme/web --bump patch --message "Fix status indicator"
releaseweave plan --json
releaseweave bump
```

`init` discovers Cargo.toml, package.json and pyproject.toml recursively, skipping build, vendor, example and test directories. It treats each discovered manifest as a package; edit the resulting `releaseweave.toml` to select only the packages you release. Virtual Cargo workspace roots are skipped. A single project with multiple synchronized manifests should use `additional_manifests` explicitly instead of discovery.

```toml
mode = "monorepo"
changesets_dir = ".changesets"

[[packages]]
name = "acme-api"
manifest = "crates/api/Cargo.toml"
changelog = "crates/api/CHANGELOG.md"

[[packages]]
name = "@acme/web"
manifest = "packages/web/package.json"
changelog = "packages/web/CHANGELOG.md"
```

A changeset can target multiple packages with different bump levels:

```toml
message = "Coordinate the new API response and its client"

[releases]
acme-api = "minor"
"@acme/web" = "patch"
```

The highest requested level wins once per package: major > minor > patch. Bumps are computed from current manifest versions, so merging several changesets does not repeatedly increment a package or reuse stale versions.

## GitHub

```sh
releaseweave init --yes --github
```

This writes two workflows without replacing existing files:

1. A push to `main` produces a release plan and a `releaseweave/releases` PR with version changes, package changelogs and `.releaseweave/release.json`.
2. Merging that same-repository PR into `main` creates one GitHub Release per affected package, at the exact merge commit, using the recorded tag and notes.

The root `action.yml` builds the CLI from the checked-out action revision with `cargo install --locked`. The repository's workflows use the local action; generated consumer templates use `ricardoleal20/ChangeForge@main`. **Consumer templates become usable after these changes are merged upstream.** Pin the action to the resulting commit SHA for reproducible consumer setup. The existing repository URL remains valid; this PoC does not rename the remote repository.

Enable **Allow GitHub Actions to create and approve pull requests** in repository settings. The PR workflow requires `contents: write` and `pull-requests: write`; the release workflow requires `contents: write`. Change the `main` branch in the workflow files if needed. Package registries are not published by these workflows. `release` is a read-only local preview/export command.

## Integrity and scope

- Structured version editing preserves TOML comments and unrelated dependency versions; JSON manifests are pretty-printed.
- Local Cargo.lock package entries are updated along with Cargo versions. Missing lockfiles are not created. Refresh other package-manager lockfiles using their tools before merging the release PR.
- Malformed or unknown-package changesets block the entire release. No changesets is a successful no-op. Only validated TOML changesets are removed; README files and other content remain.
- Writes are staged before replacement. Ordinary write/cleanup failures restore snapshots. Operations use a lock to prevent concurrent CLI mutations. This is not crash-atomic across files; after an interrupted process, inspect the working tree before removing `.releaseweave/operation.lock`.
- Repository-relative paths cannot escape the root or use symbolic links.
- Incompatible local Cargo path-dependency requirements block planning, including workspace members omitted from release configuration and inherited workspace dependency requirements. Registry dependencies are left alone.
- The PoC supports stable SemVer versions and **independent package releases**. Cargo `version.workspace` inheritance, automatic internal-dependency range updates/propagation, linked/fixed version groups and prereleases are not implemented. Coordinate dependent-package changesets and update dependency ranges manually.
- AI message generation, message-template selection and automatic developer commits from the old experimental CLI are outside this repaired PoC. Messages can be supplied interactively or with `--message`; users commit and review changes normally.

## Migration

Existing `changeforge.toml` and `[tool.changeforge]` config remain readable for single-package repositories. `init --yes` can migrate an existing `changeforge.toml` into `releaseweave.toml`. Legacy TOML changesets are read by change type and description; their stored target version is recalculated from the manifest. Legacy changesets have no package identity and are rejected in monorepo mode until explicitly migrated. See [migration notes](docs/MIGRATION.md).

## Development

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
python3 -m unittest discover -s scripts -p 'test_*.py' -v
cargo build --release --locked
python3 scripts/demo.py
python3 docs/docs_build.py build
```

[Contributing](CONTRIBUTING.md) · [Code of conduct](CODE_OF_CONDUCT.md) · [Security](SECURITY.md) · [Roadmap](ROADMAP.md) · [MIT license](LICENCE)

Contributions are welcome through issues and pull requests. Main is maintained through reviewed PRs and required CI checks; see the contributor guide for the workflow.
