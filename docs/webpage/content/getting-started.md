# Getting Started with ChangeForge

Follow this guide to install, configure, and generate your first changelog in just a few minutes.

## 1. Installation

Install the package using pip.

```
$ pip install changeforge
```

## 2. Initializing Your Project

Run the `init` command in your project's root directory. This will guide you through an interactive
setup and write a `changeforge.toml` configuration file. You can also opt-in to GitHub workflows
for PRs and releases.

```
$ changeforge init
```

Example configuration written to `changeforge.toml`:

```
[changeforge]
version_path = ["pyproject.toml"]   # or Cargo.toml, etc.
changesets_dir = ".changesets"
changelog_path = "CHANGELOG.md"
ai_enabled = false
templates_dir = ""
commit_on_create = false
```

See the full configuration reference in [Configuration](./configuration.html).

## 3. Creating Your First Changelog Entry

Use the `create` command to record a new change. This interactively collects name, type, tag,
module/file, and message, then writes a TOML file in the `.changesets` directory.

```
$ changeforge create
```

If AI is enabled or templates are configured in `changeforge.toml`, you'll be offered those methods
to craft the message; otherwise you can write it from scratch.

## 4. Previewing Your Changes

Before updating your changelog, preview the suggested version and a grouped summary of changes with
`list`.

```
$ changeforge list
```

## 5. Generating the CHANGELOG.md

When you're ready, run `bump`. This computes the next version from pending changesets, builds the
entry, and inserts it into your `CHANGELOG.md`.

```
$ changeforge bump
```


