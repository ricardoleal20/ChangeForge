# changeforge list

Generates a read‑only preview from the current changesets and computes the next version to bump. It scans your `.changesets`, determines the highest impact change (MAJOR > MINOR > PATCH), and prints a concise summary grouped by tag.

## Usage

```
$ changeforge list
```

## What it does

- Reads pending changesets and finds the suggested next version.
- Prints a header: _New version to be bumped: vX.Y.Z._
- Iterates through change types in order (MAJOR → MINOR → PATCH).
- For each tag, prints a bullet and lists all messages under that tag.
- If a changeset includes a module/file, it is shown before the message.
- Does not modify files or state.

## Example

```
$ changeforge list

# New version to be bumped: v1.1.0.

- [Feature]
    - app/auth/login.py: Add login route
- [Bug]
    - Fix crash on startup
- [Refactor]
    - src/utils/date.ts: Simplify date handling
```


