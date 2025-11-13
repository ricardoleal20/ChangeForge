# changeforge create

Creates a new changeset file through an interactive flow. It helps you name the changeset, select the type and tag, choose the affected module, craft the message, and then saves it into the `.changesets` directory.

## Usage

```
$ changeforge create
```

## Interactive steps

- Name: provide a changeset name (leave blank for a random one).
- Change type: choose between `MAJOR`, `MINOR`, or `PATCH`.
- Tag: after the type, pick a tag from a curated list (e.g., Feature, Bug, Refactor; options vary per type).
- Module/file:
  - If Git has pending changes, you can pick from `git diff --name-only HEAD`.
  - Otherwise, auto‑detected files from `src`, `tests`, `lib`, `app`, or specify manually.
- Message: choose how to create it (see below).
- Review & confirm: you will see a summary box and can confirm to save.

## Message methods

- Generate with AI (available when `ai_enabled` in `changeforge.toml`): analyzes the context and proposes a message, which you can edit.
- Use message template (available when `templates_dir` has files): pick a template, then customize it. The template cannot be saved as‑is.
- Write from scratch: type your own message (required, cannot be empty).

## What it does

- Reads the current version and calculates the next version based on the selected type.
- Writes a TOML file into `.changesets/<name>.toml` with the change data.
- If `commit_on_create` is enabled in `changeforge.toml`, optionally prompts to git add and commit the changeset (and the selected module).

## Example

```
$ changeforge create
# …answer the prompts…
# ✔ Changeset `my-feature.toml` has been created!
```


