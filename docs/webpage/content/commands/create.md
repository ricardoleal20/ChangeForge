# create

Record the change intent. No version files, Git commits or remote services are modified.

```sh
releaseweave create
releaseweave create --bump patch --message "Fix timeout"
releaseweave create --package acme-api --bump minor --message "Add health endpoint" --name api-health
```

Without flags, create prompts in a terminal. In automation, supply --bump and --message plus --package for monorepos. Repeat --package to target several packages at the same bump level. The alias add behaves identically. Messages are serialized as TOML, including quotes and newlines.
