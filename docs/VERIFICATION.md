# ReleaseWeave PoC verification

Verified locally on 2026-10-22 against the final implementation on `codex/releaseweave-poc`.

| Check | Result |
| --- | --- |
| `cargo test --all-targets --locked --offline` | 24 CLI integration tests passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py' -v` | 6 publishing regression tests passed |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy --all-targets --locked --offline -- -D warnings` | passed, no warnings |
| `cargo build --release --locked --offline` | passed |
| `cargo install --path . --locked --bin releaseweave --root .demo/install --target-dir target --offline` | local installation passed |
| `python3 scripts/demo.py --output .demo/delivery` | both scenarios passed, dry-run verified |
| Real single-package Cargo project: generate-lockfile → changeset → bump → cargo check --locked --offline | passed |
| actionlint v1.7.7, repository workflows and consumer templates | passed; shellcheck/pyflakes integrations disabled |
| Workflow shell and embedded Python syntax | checked locally |
| `python3 docs/docs_build.py build` | eight pages generated |

The demo's machine-readable release plans are saved in [poc-evidence.json](poc-evidence.json). The full local transcript and resulting project trees are under `.demo/delivery/`, which is intentionally ignored by Git. Running the demo again without --output creates a fresh directory.

The original repository compiled but had zero tests. Initial CLI regression tests failed against it. Subsequent regressions reproduced unsafe dependency planning, unverified GitHub tags and failed cleanup rollback before their fixes; the final tests cover the corrected behavior.

A read-only independent review found no remaining critical or important issues after fixes. It repeated the unconfigured-consumer, registry-source and inherited-dependency checks independently.

No authenticated GitHub mutation, push, PR, remote repository rename, registry upload or real GitHub release was performed. Workflows are prepared and statically validated; they have not been exercised in GitHub Actions in this session. Consumer templates require upstream integration of the new composite action. Publishing tests use actual local Git repositories and tags, with gh replaced at the external API boundary.

The PoC does not implement dependency bump propagation, inherited Cargo package versions, prereleases, fixed-version groups, other package-manager lockfile resolution, AI messages or automatic developer commits. Local path dependency incompatibilities in Cargo are rejected with instructions to coordinate their updates manually. See README.md and MIGRATION.md for the supported contract.
