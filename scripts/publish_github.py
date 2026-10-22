#!/usr/bin/env python3
"""Publish a validated release manifest. Used only by the release-on-merge workflow."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


def run(*args):
    return subprocess.run(args, check=True, text=True, capture_output=True).stdout


def remote_tag_commit(tag):
    ref = 'refs/tags/' + tag
    # ls-remote exits nonzero on connection/auth errors, and returns no lines for missing refs.
    refs = dict(line.split()[::-1] for line in run('git', 'ls-remote', '--tags', 'origin', ref, ref + '^{}').splitlines())
    return refs.get(ref + '^{}', refs.get(ref))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, required=True)
    args = parser.parse_args()
    plan = json.loads(args.manifest.read_text())
    if plan.get('schema_version') != 1:
        raise ValueError('Unsupported release manifest schema')
    commit = os.environ['RELEASE_COMMIT']
    if not re.fullmatch(r'[0-9a-fA-F]{40}', commit):
        raise ValueError('RELEASE_COMMIT must be a full Git commit SHA')
    releases = plan['releases']
    existing = {}
    # Validate all targets before making the first remote mutation.
    for release in releases:
        tag = release['tag']
        run('git', 'check-ref-format', 'refs/tags/' + tag)
        resolved = remote_tag_commit(tag)
        if resolved is not None and resolved != commit:
            raise ValueError('Tag ' + tag + ' points to ' + resolved + ', not the release merge commit ' + commit)
        existing[tag] = resolved
    if not releases:
        print('No releases in the applied manifest')
        return 0
    known_releases = set(run('gh', 'api', '--paginate', 'repos/{owner}/{repo}/releases', '--jq', '.[].tag_name').splitlines())
    for release in releases:
        tag = release['tag']
        if tag in known_releases:
            if existing[tag] is None:
                raise ValueError('Release exists without a verifiable tag: ' + tag)
            print('Verified existing release:', tag)
            continue
        if existing[tag] is None:
            # Create the ref explicitly. A conflicting concurrent creation fails rather than
            # letting gh release create reuse an unverified tag through --target semantics.
            run('gh', 'api', '--method', 'POST', 'repos/{owner}/{repo}/git/refs',
                '-f', 'ref=refs/tags/' + tag, '-f', 'sha=' + commit)
        with tempfile.TemporaryDirectory() as tmp:
            notes = Path(tmp) / 'notes.md'
            notes.write_text(release['notes'])
            run('gh', 'release', 'create', tag, '--verify-tag', '--title',
                release['package'] + ' ' + release['next_version'], '--notes-file', str(notes))
        print('Published:', tag)
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error))
