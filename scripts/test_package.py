"""Offline regression tests. These do not claim to build a native installer."""
from __future__ import annotations

import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import tempfile
import tomllib
import unittest
from unittest import mock
import zipfile

SPEC = importlib.util.spec_from_file_location('burrow_package', Path(__file__).with_name('package.py'))
assert SPEC and SPEC.loader
package = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(package)
ROOT = Path(__file__).resolve().parent.parent


class VersionTests(unittest.TestCase):
    def test_manifest_and_lock_match(self):
        with (ROOT / 'Cargo.lock').open('rb') as source:
            packages = tomllib.load(source)['package']
        self.assertEqual([p['version'] for p in packages if p['name'] == 'burrow'], [package.VERSION])

    def test_ui_gets_its_version_from_cargo(self):
        lib = (ROOT / 'src/lib.rs').read_text(encoding='utf-8')
        ui = (ROOT / 'src/ui.rs').read_text(encoding='utf-8')
        self.assertIn('env!("CARGO_PKG_VERSION")', lib)
        self.assertIn('Burrow {VERSION}', ui)
        self.assertIn('{VERSION} · Preview', ui)
        self.assertNotIn('Burrow 0.1.0', ui)

    def test_release_names_come_from_the_package(self):
        workflow = (ROOT / '.github/workflows/burrow.yml').read_text(encoding='utf-8')
        self.assertIn('version="$(python3 scripts/package.py version)"', workflow)
        self.assertIn('tag="v${version}-preview-${GITHUB_SHA::7}"', workflow)
        self.assertIn('needs: [resolve, build]', workflow)
        self.assertIn('scripts/package.py verify-release', workflow)
        self.assertNotIn('generate-lockfile', workflow)
        self.assertNotIn('v0.1.0-preview-', workflow)

    def test_invalid_release_version_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for version in ['../bad', '0.1', '0.1.1;echo bad', '0.1.1-preview']:
                (root / 'Cargo.toml').write_text(f'[package]\nversion = "{version}"\n')
                with self.subTest(version=version), self.assertRaises(ValueError):
                    package.read_version(root)

    def test_windows_upgrade_identity_is_unchanged(self):
        installer = (ROOT / 'packaging/windows.iss').read_text()
        self.assertIn('AppId={{5C2B3BEE-49D9-44D4-9409-1875209B5F78}', installer)
        self.assertIn('PrivilegesRequired=lowest', installer)
        self.assertIn('UsePreviousAppDir=yes', installer)
        self.assertIn('CloseApplicationsFilter=burrow.exe', installer)
        self.assertIn('#error AppVersion must be supplied', installer)

    def test_python_is_selected_for_all_ci_jobs(self):
        workflow = (ROOT / '.github/workflows/burrow.yml').read_text(encoding='utf-8')
        self.assertEqual(workflow.count("python-version: '3.13'"), 3)

    def test_obsolete_tree_publisher_is_not_shipped(self):
        self.assertFalse((ROOT / '.github/workflows/publish-staged-tree.yml').exists())



class DmgRetryTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.stage = Path(self.temporary.name) / 'stage'
        self.output = Path(self.temporary.name) / 'Burrow.dmg'
        self.capture = contextlib.ExitStack()
        self.addCleanup(self.capture.close)
        self.capture.enter_context(contextlib.redirect_stdout(io.StringIO()))
        self.capture.enter_context(contextlib.redirect_stderr(io.StringIO()))

    @staticmethod
    def result(returncode=0, stderr=''):
        return subprocess.CompletedProcess(['hdiutil', 'create'], returncode, stdout='', stderr=stderr)

    @mock.patch.object(package.time, 'sleep')
    @mock.patch.object(package.subprocess, 'run')
    def test_success_does_not_retry(self, run, sleep):
        run.return_value = self.result()
        package.create_dmg(self.stage, self.output)
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.kwargs['timeout'], 180)
        sleep.assert_not_called()

    @mock.patch.object(package.time, 'sleep')
    @mock.patch.object(package.subprocess, 'run')
    def test_resource_busy_retries_then_succeeds(self, run, sleep):
        self.output.write_bytes(b'incomplete image')
        run.side_effect = [self.result(1, 'hdiutil: create failed - Resource busy'), self.result()]
        package.create_dmg(self.stage, self.output)
        self.assertEqual(run.call_count, 2)
        sleep.assert_called_once_with(2)
        self.assertFalse(self.output.exists(), 'only the partial output should be removed before retry')

    @mock.patch.object(package.time, 'sleep')
    @mock.patch.object(package.subprocess, 'run')
    def test_unrelated_failure_is_not_hidden(self, run, sleep):
        run.return_value = self.result(1, 'permission denied')
        with self.assertRaises(subprocess.CalledProcessError):
            package.create_dmg(self.stage, self.output)
        self.assertEqual(run.call_count, 1)
        sleep.assert_not_called()

    @mock.patch.object(package.time, 'sleep')
    @mock.patch.object(package.subprocess, 'run')
    def test_retries_are_bounded(self, run, sleep):
        run.return_value = self.result(1, 'Resource busy')
        with self.assertRaises(subprocess.CalledProcessError):
            package.create_dmg(self.stage, self.output)
        self.assertEqual(run.call_count, 4)
        self.assertEqual([c.args[0] for c in sleep.call_args_list], [2, 4, 8])

    @mock.patch.object(package.time, 'sleep')
    @mock.patch.object(package.subprocess, 'run')
    def test_timeout_is_not_marked_success(self, run, sleep):
        run.side_effect = subprocess.TimeoutExpired(['hdiutil'], 180)
        with self.assertRaises(subprocess.TimeoutExpired):
            package.create_dmg(self.stage, self.output)
        sleep.assert_not_called()

    def test_zero_attempts_is_invalid(self):
        with self.assertRaises(ValueError):
            package.create_dmg(self.stage, self.output, attempts=0)


class ReleaseAssetTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.dist = Path(self.temporary.name)
        patch = mock.patch.object(package, 'DIST', self.dist)
        patch.start()
        self.addCleanup(patch.stop)
        # Deliberate stub bytes: these tests validate asset selection, not binary validity.
        for suffix in ['macOS-AppleSilicon.dmg', 'macOS-Intel.dmg',
                       'Windows-x64-Setup.exe', 'Windows-x64-portable.zip', 'source.zip']:
            (self.dist / f'Burrow-{package.VERSION}-{suffix}').write_bytes(b'build-asset-fixture')
        (self.dist / 'THIRD_PARTY_NOTICES.txt').write_text('notices fixture')
        (self.dist / 'Cargo.lock').write_text(f'[[package]]\nname = "burrow"\nversion = "{package.VERSION}"\n')

    def test_expected_set_is_accepted(self):
        package.verify_release()

    def test_missing_mac_is_rejected(self):
        (self.dist / f'Burrow-{package.VERSION}-macOS-Intel.dmg').unlink()
        with self.assertRaises(ValueError):
            package.verify_release()

    def test_zero_byte_asset_is_rejected(self):
        (self.dist / f'Burrow-{package.VERSION}-Windows-x64-Setup.exe').write_bytes(b'')
        with self.assertRaises(ValueError):
            package.verify_release()

    def test_unexpected_old_version_is_rejected(self):
        (self.dist / 'Burrow-0.0.1-Windows-x64-Setup.exe').write_bytes(b'old')
        with self.assertRaises(ValueError):
            package.verify_release()

    def test_mismatched_lock_is_rejected(self):
        (self.dist / 'Cargo.lock').write_text('[[package]]\nname = "burrow"\nversion = "0.0.1"\n')
        with self.assertRaises(ValueError):
            package.verify_release()

    def test_checksums_are_repeatable_and_exclude_themselves(self):
        package.checksums()
        first = (self.dist / 'SHA256SUMS.txt').read_bytes()
        package.checksums()
        self.assertEqual((self.dist / 'SHA256SUMS.txt').read_bytes(), first)
        self.assertEqual(len(first.splitlines()), 7)
        self.assertNotIn(b'  SHA256SUMS.txt', first)
        package.verify_release()


class SourceBundleTests(unittest.TestCase):
    @mock.patch.object(package.subprocess, 'check_output')
    def test_source_includes_the_locked_inputs(self, git_files):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for filename in ['Cargo.toml', 'Cargo.lock', 'THIRD_PARTY_NOTICES.txt']:
                (root / filename).write_text(f'{filename} fixture')
            git_files.return_value = b'Cargo.toml\0'
            with mock.patch.object(package, 'ROOT', root), mock.patch.object(package, 'DIST', root / 'dist'):
                package.source_bundle()
            with zipfile.ZipFile(root / 'dist' / f'Burrow-{package.VERSION}-source.zip') as archive:
                self.assertEqual(set(archive.namelist()), {
                    f'Burrow-{package.VERSION}-source/{name}' for name in
                    ['Cargo.toml', 'Cargo.lock', 'THIRD_PARTY_NOTICES.txt']
                })


if __name__ == '__main__':
    unittest.main()
