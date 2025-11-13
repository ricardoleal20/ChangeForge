# changeforge bump

Calculates the next version from pending changesets and writes a new entry into `CHANGELOG.md`.

## Usage

```
$ changeforge bump
```

## What it does

- Loads pending changesets.
- Computes the next version from them (MAJOR > MINOR > PATCH).
- Generates a formatted changelog entry.
- Opens `CHANGELOG.md`, inserts the new entry before the first `## [` version section, and saves it.

## Example

```
$ changeforge bump
# CHANGELOG.md updated with new entry v1.1.0
```


