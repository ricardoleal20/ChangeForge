#!/usr/bin/env python3
"""Run the real CLI against disposable copies of both fixtures. Requires Python 3.9+."""
import argparse
import datetime
import json
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=REPO / 'target/release/releaseweave')
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.exists():
        subprocess.run(['cargo', 'build', '--release', '--locked'], cwd=REPO, check=True)
    if not binary.is_file():
        raise SystemExit('CLI binary not found: ' + str(binary))
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
    output = (args.output or REPO / '.demo' / stamp).resolve()
    output.mkdir(parents=True, exist_ok=False)
    transcript = []

    def run(root, *command, quiet=False):
        result = subprocess.run([str(binary), '--root', str(root), *command],
                                text=True, capture_output=True, check=True)
        transcript.append({'scenario': root.name, 'command': list(command), 'stdout': result.stdout})
        if not quiet:
            print('\n$ releaseweave --root ' + root.name + ' ' + ' '.join(command))
            print(result.stdout, end='')
        return result.stdout

    reports = {}
    for scenario in ['single-repo', 'monorepo']:
        root = output / scenario
        shutil.copytree(REPO / 'examples' / scenario, root,
                        ignore=shutil.ignore_patterns('target', 'Cargo.lock', '.releaseweave', '.changesets'))
        run(root, 'init', '--yes', '--github')
        if scenario == 'single-repo':
            run(root, 'create', '--bump', 'minor', '--message', 'Add a configurable greeting', '--name', 'configurable-greeting')
        else:
            run(root, 'create', '--package', 'acme-api', '--bump', 'minor', '--message', 'Add a health endpoint', '--name', 'api-health')
            run(root, 'create', '--package', '@acme/web', '--bump', 'patch', '--message', 'Fix the status indicator', '--name', 'web-status')
        run(root, 'plan')
        before = {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()}
        run(root, 'bump', '--dry-run')
        after = {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()}
        if before != after:
            raise AssertionError('Dry-run modified files')
        run(root, 'bump')
        applied = json.loads(run(root, 'release', '--json', quiet=True))
        pending = json.loads(run(root, 'plan', '--json', quiet=True))
        if pending['releases'] or pending['changesets']:
            raise AssertionError('Changesets were not consumed')
        expected = {'hello-release': '1.3.0'} if scenario == 'single-repo' else {'acme-api': '1.5.0', '@acme/web': '2.1.1'}
        if {r['package']: r['next_version'] for r in applied['releases']} != expected:
            raise AssertionError('Unexpected release versions')
        if scenario == 'monorepo':
            worker = root / 'crates/worker/Cargo.toml'
            if worker.read_bytes() != before['crates/worker/Cargo.toml'] or (root / 'crates/worker/CHANGELOG.md').exists():
                raise AssertionError('Untouched worker was changed')
        reports[scenario] = {'dry_run_verified': True, 'pending_after_bump': 0, **applied}
    (output / 'report.json').write_text(json.dumps(reports, indent=2, ensure_ascii=False) + '\n')
    (output / 'transcript.json').write_text(json.dumps(transcript, indent=2, ensure_ascii=False) + '\n')
    print('\nBoth scenarios verified. No remote releases, tags or pushes were made.')
    print('Results, manifests and changelogs: ' + str(output))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as error:
        print(error.stderr or str(error), file=sys.stderr)
        raise SystemExit(error.returncode)
