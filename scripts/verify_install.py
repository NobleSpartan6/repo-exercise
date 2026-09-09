#!/usr/bin/env python3
"""Disposable CI installation checks. Never use a real user's install directory."""
from pathlib import Path
import hashlib
import os
import plistlib
import shutil
import subprocess
import sys
import tempfile
import time
from native_smoke import verify
from package import ROOT, DIST, VERSION


def run(*args):
    subprocess.run([str(a) for a in args], check=True, timeout=120)


def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def windows():
    old = Path(os.environ['BURROW_OLD_INSTALLER'])
    if digest(old) != '94dcb42542d1cba4dad1936933d34e3c886de066d68441c0033a469068857e91':
        raise ValueError('Old installer is not the verified 0.1.1 release asset')
    logs = ROOT / 'artifacts'
    logs.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='burrow-install-', dir=os.environ['RUNNER_TEMP']) as temp:
        stage = Path(temp) / 'Burrow'
        args = ['/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', f'/DIR={stage}']
        run(old, *args, f'/LOG={logs / "install-0.1.1.log"}')
        if not (stage / 'burrow.exe').is_file():
            raise RuntimeError('Old installer did not install the executable')
        run(DIST / f'Burrow-{VERSION}-Windows-x64-Setup.exe', *args, f'/LOG={logs / "upgrade.log"}')
        if digest(stage / 'burrow.exe') != digest(ROOT / 'target/release/burrow.exe'):
            raise RuntimeError('Installed executable does not match this release build')
        verify(stage / 'burrow.exe', logs)
        run(stage / 'unins000.exe', '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART')
        for _ in range(60):
            if not (stage / 'burrow.exe').exists(): break
            time.sleep(0.1)
        else: raise RuntimeError('Uninstaller did not remove the application')
        (logs / 'INSTALL-TEST.txt').write_text(
            f'PASS: verified 0.1.1 installer -> {VERSION} in-place upgrade -> byte-identical executable '
            '-> native six-page render -> uninstall, in an isolated CI install directory.\n'
            'Not a physical-device, SmartScreen, or interactive installer-wizard certification.\n', encoding='utf-8')


def macos(architecture):
    label = 'AppleSilicon' if architecture == 'apple-silicon' else 'Intel'
    logs = ROOT / 'artifacts'
    logs.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='burrow-install-', dir=os.environ['RUNNER_TEMP']) as temp:
        mount = Path(temp) / 'mounted'
        mount.mkdir()
        run('hdiutil', 'attach', '-readonly', '-nobrowse', '-mountpoint', mount, DIST / f'Burrow-{VERSION}-macOS-{label}.dmg')
        try:
            destination = Path(temp) / 'Applications' / 'Burrow.app'
            shutil.copytree(mount / 'Burrow.app', destination, symlinks=True)
            with (destination / 'Contents/Info.plist').open('rb') as source:
                assert plistlib.load(source)['CFBundleShortVersionString'] == VERSION
            binary = destination / 'Contents/MacOS/burrow'
            assert digest(binary) == digest(mount / 'Burrow.app/Contents/MacOS/burrow')
            run('codesign', '--verify', '--deep', '--strict', destination)
            verify(binary, logs)
        finally: run('hdiutil', 'detach', mount)
        (logs / 'INSTALL-TEST.txt').write_text(
            f'PASS: {label} DMG mounted read-only -> app copied out -> version and binary hash checked '
            '-> ad-hoc signature verified -> native six-page render.\n'
            'Not a Gatekeeper, notarization, Finder upgrade, or physical-device certification.\n', encoding='utf-8')


if __name__ == '__main__':
    if os.environ.get('GITHUB_ACTIONS') != 'true':
        raise SystemExit('This installation harness runs only in disposable GitHub Actions runners.')
    if sys.platform == 'win32': windows()
    elif sys.platform == 'darwin': macos(sys.argv[1])
    else: raise SystemExit('Supported installation test platforms: Mac and Windows')
