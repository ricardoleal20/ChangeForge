# init

Discover supported package manifests and save releaseweave.toml without overwriting existing configuration.

```sh
releaseweave init --yes
releaseweave init --yes --mode monorepo
releaseweave init --yes --github
```

Review the discovered packages in the config. Discovery skips virtual Cargo workspace roots and build, vendor, example and test directories. For synchronized manifests, configure additional_manifests manually. --github adds release PR and release-on-merge workflow templates without overwriting existing files. Consumer templates require the repaired action to be merged upstream first.
