"""Exercise GitHub publishing boundaries with real local Git tags and a fake gh executable."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

REPO = Path(__file__).resolve().parents[1]


class PublisherTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.remote = self.root / 'remote.git'
        subprocess.run(['git', 'init', '--bare', str(self.remote)], check=True, capture_output=True)
        self.work = self.root / 'work'
        self.work.mkdir()
        self.git('init')
        self.git('remote', 'add', 'origin', str(self.remote))
        self.git('-c', 'user.name=Tester', '-c', 'user.email=test@example.invalid', 'commit', '--allow-empty', '-m', 'first')
        self.first = self.git('rev-parse', 'HEAD').strip()
        self.git('-c', 'user.name=Tester', '-c', 'user.email=test@example.invalid', 'commit', '--allow-empty', '-m', 'merge')
        self.merge = self.git('rev-parse', 'HEAD').strip()
        self.git('push', 'origin', 'HEAD:main')
        self.manifest = self.root / 'releases.json'
        self.manifest.write_text(json.dumps({'schema_version': 1, 'releases': [{
            'tag': 'v1.2.4', 'package': 'demo', 'next_version': '1.2.4', 'notes': 'Fix'}]}))
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        gh = self.bin / 'gh'
        gh.write_text('''#!/usr/bin/env python3
import json, os, sys
with open(os.environ['GH_CALLS'], 'a') as out:
    out.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1] == 'api' and '--jq' in sys.argv:
    print(os.environ.get('EXISTING_RELEASE', ''))
else:
    print('{}')
''')
        gh.chmod(0o755)

    def git(self, *args):
        return subprocess.run(['git', *args], cwd=self.work, check=True, text=True, capture_output=True).stdout

    def run_publisher(self, existing=''):
        env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ['PATH'],
                   RUNNER_TEMP=str(self.root), RELEASE_COMMIT=self.merge,
                   GH_REPO='acme/example', GH_CALLS=str(self.root / 'calls.jsonl'),
                   EXISTING_RELEASE=existing)
        script = REPO / 'scripts/publish_github.py'
        command = [sys.executable, str(script), "--manifest", str(self.manifest)]
        result = subprocess.run(command, cwd=self.work, env=env, text=True, capture_output=True)
        calls = self.root / 'calls.jsonl'
        return result, [json.loads(line) for line in calls.read_text().splitlines()] if calls.exists() else []

    def test_wrong_lightweight_tag_blocks_publication(self):
        self.git('tag', 'v1.2.4', self.first)
        self.git('push', 'origin', 'refs/tags/v1.2.4')
        result, calls = self.run_publisher()
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertFalse(any(c[:2] == ['release', 'create'] for c in calls))

    def test_wrong_annotated_tag_blocks_publication(self):
        self.git('-c', 'user.name=Tester', '-c', 'user.email=test@example.invalid', 'tag', '-a', 'v1.2.4', self.first, '-m', 'old')
        self.git('push', 'origin', 'refs/tags/v1.2.4')
        result, _ = self.run_publisher()
        self.assertNotEqual(result.returncode, 0)

    def test_existing_release_with_wrong_tag_is_not_silently_skipped(self):
        self.git('tag', 'v1.2.4', self.first)
        self.git('push', 'origin', 'refs/tags/v1.2.4')
        result, _ = self.run_publisher(existing='v1.2.4')
        self.assertNotEqual(result.returncode, 0)

    def test_retry_with_matching_annotated_tag_skips_existing_release(self):
        self.git('-c', 'user.name=Tester', '-c', 'user.email=test@example.invalid', 'tag', '-a', 'v1.2.4', self.merge, '-m', 'release')
        self.git('push', 'origin', 'refs/tags/v1.2.4')
        result, calls = self.run_publisher(existing='v1.2.4')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(any(c[:2] == ['release', 'create'] for c in calls))

    def test_new_tag_is_created_at_merge_sha_before_release(self):
        result, calls = self.run_publisher()
        self.assertEqual(result.returncode, 0, result.stderr)
        ref_calls = [c for c in calls if 'sha=' + self.merge in c]
        self.assertEqual(len(ref_calls), 1)
        release = next(c for c in calls if c[:2] == ['release', 'create'])
        self.assertIn('--verify-tag', release)

    def test_remote_lookup_failure_blocks_publication(self):
        self.git('remote', 'set-url', 'origin', str(self.root / 'missing.git'))
        result, calls = self.run_publisher()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(c[:2] == ['release', 'create'] for c in calls))


if __name__ == '__main__':
    unittest.main()
