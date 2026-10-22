# Contributing to ReleaseWeave

Use a current stable Rust toolchain. Python 3.9+ is only needed for the demo runner.

```sh
cargo test --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
python3 scripts/demo.py
```

Add a subprocess regression test under tests/cli.rs for behavior changes. Keep Cargo.lock committed. Never run a destructive demo in a developer repository; use copies of the examples. Submit a PR against the original ChangeForge upstream until its owner elects to rename it remotely.
