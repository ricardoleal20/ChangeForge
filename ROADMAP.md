# ReleaseWeave roadmap

The current PoC supports explicit stable versions, single repositories, independent monorepo package releases, Cargo/npm/Python manifests, dry-run, JSON plans, changelogs and GitHub release workflows. See README.md for the exact supported contract.

Potential follow-up work is tracked through issues and must be designed and tested before being advertised as supported:

- Propagate internal dependency range updates and dependent-package bumps.
- Support inherited Cargo package versions and linked/fixed release groups.
- Define prerelease and snapshot release policies.
- Integrate package-manager lockfile resolution beyond local Cargo.lock entries.
- Publish native CLI distributions and define a supported release/version policy.
- Add optional npm, crates.io and PyPI publishing after the local plan/apply workflow.

These are roadmap items, not delivery commitments. Small documentation improvements, reproducible regression cases and platform fixes are welcome first contributions.
