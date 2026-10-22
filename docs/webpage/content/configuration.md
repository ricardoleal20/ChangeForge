# Configuration

ReleaseWeave reads releaseweave.toml at the repository root. Use --root PATH from another directory.

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

Use mode = "single" with exactly one package for a single repository. Supported version fields are package.version in Cargo, project.version or tool.poetry.version in Python, and top-level version in package.json. Set version_key for a custom TOML field. Add additional_manifests for synchronized version files belonging to one package.

The PoC requires explicit stable versions. Cargo version.workspace inheritance and automatic dependent-package propagation are not implemented. Local Cargo.lock entries are synchronized; other lockfiles must be refreshed using the relevant package manager.

Legacy changeforge.toml and embedded tool.changeforge configuration remain readable for single-package migration. New configuration takes precedence. All paths must stay inside the repository, and symbolic links are rejected.

Incompatible local Cargo path-dependency requirements block the release before any writes, including unconfigured workspace consumers. Registry dependencies are not rewritten.
