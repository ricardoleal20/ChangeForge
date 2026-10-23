# Getting started

ReleaseWeave is a Rust CLI for changeset-driven releases in single repositories and monorepos.

## Build and install

```sh
cargo build --release --locked
cargo install --path . --locked --bin releaseweave
```

## Run the local PoC

```sh
python3 scripts/demo.py
```

The demo runs both examples on disposable copies, verifies dry-run, and leaves updated manifests, changelogs and JSON evidence under .demo/. Python 3.9+ is needed for this runner only.

## First release

```sh
releaseweave init --yes
releaseweave create --bump minor --message "Add a new feature"
releaseweave plan
releaseweave bump --dry-run
releaseweave bump
releaseweave release --dry-run
```

For a monorepo, initialize with --mode monorepo and select each changed package using --package NAME. Only affected packages receive versions, changelogs and proposed GitHub tags.
