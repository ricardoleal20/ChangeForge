# Contributing to ReleaseWeave

ReleaseWeave is a Rust CLI for changeset-driven releases in single repositories and independently versioned monorepos. The current scope and limitations are documented in the [README](../README.md) and [roadmap](../ROADMAP.md).

## Local setup

Use a current stable Rust toolchain with rustfmt and clippy, Git, and Python 3.9+ for the demo and GitHub publisher tests. Fork the repository, clone your fork and create a feature branch from main.

```sh
cargo build --locked
cargo run -- --help
```

No API keys or GitHub account are needed to run the CLI locally. Only the GitHub publishing workflow uses GitHub credentials. Never put real tokens or private repository contents into fixtures or issues.

## Before submitting a PR

```sh
cargo fmt --all -- --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
python3 -m unittest discover -s scripts -p 'test_*.py' -v
cargo build --release --locked
python3 scripts/demo.py
python3 docs/docs_build.py build
```

The CI also tests the Rust CLI on macOS and Windows. Linux runs the full demonstration and publishing-boundary tests. Keep Cargo.lock committed. Do not commit target/, generated documentation pages, .demo/, operation locks or credentials.

## Changes and release notes

Keep each PR focused. For behavior changes, add a subprocess regression test in tests/cli.rs and update the relevant documentation. Explain user-visible behavior and migration implications in the PR. Use the real filesystem and CLI in tests; keep external network operations behind a boundary that can be tested without credentials.

For a user-visible fix or feature, add a changeset from this repository root:

```sh
cargo run -- create --bump patch --message "Describe the user-visible change"
```

Use minor for compatible features and major for breaking behavior. The maintainer coordinates pre-1.0 changes and release timing. Documentation or maintenance-only PRs can explain why no changeset is needed. Do not run bump on the development branch unless the PR is specifically preparing a release.

## Demos and safety

Run demos only on disposable copies of examples. scripts/demo.py creates them under .demo/. Never point a destructive test at a developer's real project. Include regression coverage for changes to planning, manifest editing, cleanup, rollback, dependency validation or release publication.

## Reviews and maintenance

The main branch requires PR review, successful CI and resolved review conversations. The maintainer reviews external contributions. A green check does not replace review. Dependency update PRs are generated weekly and follow the same review process.

Search existing issues before reporting a bug or proposing a feature. Use [private vulnerability reporting](../SECURITY.md) for security issues, and follow the [code of conduct](../CODE_OF_CONDUCT.md).
