# Contributing Guide

Thank you for your interest in contributing. This repository follows a Gitflow workflow, semantic versioning, and conventional commits augmented with GitMoji.

## Workflow and Branching

We protect `main` and `develop`. Never commit directly to these branches. All changes go through Pull Requests.

- `main`: Production-ready code. Only merges from `release/*` and `hotfix/*`. Tag every merge with the version.
- `develop`: Latest development changes. Source for feature branches.

### Branch Types

- Feature: branch from `develop`, merge back to `develop`.
  - Naming: `feature/<author>/<LINEAR_TASK_ID>-<short-slug>`
  - Example: `feature/ricardo/ABC-123-user-authentication`
  - Keep your branch up to date with `develop` before opening a PR.
- Release: branch from `develop`, merge back to `main` and `develop`.
  - Naming: `release/vX.Y.Z` (no new features; only fixes/docs/release tasks)
- Hotfix: branch from `main`, merge back to `main` and `develop`.
  - Naming: `hotfix/vX.Y.Z` (urgent production fixes only)

Ask a maintainer for the `LINEAR_TASK_ID` before creating a feature branch if you don't have one.

## Commit Messages

Use Conventional Commits with GitMoji. Format:

`<gitmoji> type(scope): short imperative description`

Common types:
- `feat` new feature
- `fix` bug fix
- `docs` documentation changes
- `style` formatting (no code changes)
- `refactor` code refactoring
- `test` add/update tests
- `chore` maintenance tasks

Examples:
- `✨ feat(api): add user authentication endpoints`
- `🐛 fix(auth): handle token refresh race condition`

Make small, focused commits.

## Pull Requests

PR requirements:
- At least 1 approval
- CI checks pass
- Branch up to date with base
- No direct commits to protected branches
- Delete the feature branch after merge

Include:
- What changed and why (problem/solution)
- Any breaking changes and migration steps
- Screenshots for UI changes (if applicable)
- Tests added or updated

## Versioning and Releases

We use Semantic Versioning (MAJOR.MINOR.PATCH).

Release process:
1. Create `release/vX.Y.Z` from `develop`
2. Bump versions and finalize notes
3. Fix release-only issues (no new features)
4. PR into `main`
5. After merge to `main`:
   - Tag `vX.Y.Z`
   - Merge back into `develop`
   - Delete the release branch

Hotfix process:
1. Create `hotfix/vX.Y.Z` from `main`
2. Implement fix and bump patch version
3. PR into `main`
4. After merge to `main`:
   - Tag `vX.Y.Z`
   - Merge back into `develop`
   - Delete the hotfix branch

## Code Quality

- Preserve existing structures and do not remove unrelated functionality
- Prefer constants over magic numbers
- Use meaningful names and keep functions focused
- Avoid duplication; extract reusable logic
- Add tests for new behavior and bug fixes

## Testing

- Add or update tests to cover your changes, including edge cases and error paths
- Ensure the test suite passes before opening a PR

## Documentation

- Update `README.md` and other docs as needed
- Document public APIs and non-obvious decisions

## Security

Do not report vulnerabilities via public issues. See the Security Policy (`SECURITY.md`) and use GitHub Security Advisories to report privately.

## PR Checklist

- [ ] Branch type and name follow the rules
- [ ] Small, focused commits using Conventional Commits + GitMoji
- [ ] Tests added/updated and passing
- [ ] Docs updated where needed
- [ ] No unrelated changes
- [ ] Ready for review (and branch up to date)


