# plan, list and status

Show pending package releases without changing files.

```sh
releaseweave plan
releaseweave list
releaseweave status --json
```

The highest requested bump wins once per package. Unknown-package or malformed changesets stop planning with an error. No changesets is a successful empty plan. JSON includes current_version, next_version, tag and notes for each affected package.
