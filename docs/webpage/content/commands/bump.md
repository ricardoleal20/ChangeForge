# bump and release

Apply a validated release plan or preview it first.

```sh
releaseweave bump --dry-run
releaseweave bump
releaseweave release --json
```

Bump updates affected manifests and changelogs, local Cargo.lock package entries, and .releaseweave/release.json. Only successfully parsed changeset TOML files are consumed. Other files in the changeset directory remain. The version alias runs bump. There is no action when no changesets are pending.

Release is a read-only preview/export of the last applied release manifest. The GitHub workflow publishes its package tags and notes after the release PR merges. This CLI command never creates remote releases.

Writes are staged and rolled back on ordinary IO failure; they are not crash-atomic across multiple files. Inspect the diff after an interrupted process before removing .releaseweave/operation.lock.
