# ReleaseWeave functional PoC implementation plan

**Goal:** Repair the existing Rust CLI and demonstrate changeset-driven releases for a single package and independently versioned packages in a monorepo.

**Design:** Replace the global version replacement with a validated release plan. Read structured manifests, aggregate the highest requested bump per package, preserve unrelated content, and consume only validated changeset files after successful writes. Separate configuration, manifests, changesets, planning, and CLI. The existing GitHub repository remains the upstream; rename the local product and executable to ReleaseWeave, retaining a compatibility executable and legacy single-package input formats.

**Constraints:** No credentials or AI service needed for the PoC. No remote publishing during local validation. Commit Cargo.lock. Preserve manifest formatting for TOML. Explicitly reject unsupported Cargo inherited versions/dependency propagation rather than silently corrupting a workspace. Declare packages explicitly or discover manifests with init. Generated release tags are vVERSION in a single repo and PACKAGE/vVERSION in a monorepo.

## Implementation and verification

- [x] Add subprocess regression tests for single-repo, independent monorepo, scoped version updates, malformed changesets, configured paths, missing changesets, and dry-run. Run against the existing CLI to confirm failures.
- [x] Implement CLI commands init, create/add, list/status/plan, bump/version with noninteractive flags and JSON output. Keep interactive create. Introduce configuration and safe manifest editing; support Cargo.toml, package.json and pyproject.toml. Preserve legacy single-package changesets.
- [x] Implement release planning and bump application with prevalidation, rollback of writes on failure, selective cleanup and an explicit release manifest for GitHub. Test error exits and no-op behavior.
- [x] Replace broken GitHub automation with a composite action, release PR workflow and release-on-merge workflow that consumes per-package tags/notes from the release manifest. Generate these workflows through init --github.
- [x] Add runnable single-repo and monorepo fixtures, a local demo runner and a Spanish PoC guide. Rename current docs/package metadata, retain historical CHANGELOG content, remove obsolete runtime code and false pip instructions.
- [x] Verify cargo fmt, cargo test --locked, cargo clippy --all-targets --locked -- -D warnings, cargo build --release --locked, both demos and GitHub workflow structure. Record exact outcomes and limitations.

**Initial evidence:** Baseline cargo test succeeds with zero tests. Existing bump unwraps empty changesets; version updates use String::replace globally; config paths are ignored by changeset IO; cleanup removes every file in the changeset directory; workflows depend on Python/Poetry and nonexistent generated scripts. There is no per-package release model.
