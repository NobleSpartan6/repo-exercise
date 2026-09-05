#!/usr/bin/env python3
"""Build native preview packages using only Python's standard library.

No signing credentials are invented or requested. macOS receives an ad-hoc
integrity signature, not a trusted Developer ID signature or notarization.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import struct
import subprocess
import sys
import zipfile
import zlib

ROOT = Path(__file__).resolve().parent.parent
os.chdir(ROOT)
VERSION = re.search(r'^version = "([^"]+)"', (ROOT / 'Cargo.toml').read_text(), re.M).group(1)
DIST = ROOT / 'dist'
DIST.mkdir(exist_ok=True)


def run(*args: str | Path) -> None:
    subprocess.run([str(a) for a in args], check=True)


def png(size: int) -> bytes:
    def chunk(kind: bytes, data: bytes) -> bytes:
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
    raw = bytearray()
    for row in range(size):
        raw.append(0)
        for column in range(size):
            channels = [0, 0, 0, 0]
            for sy in (0.25, 0.75):
                for sx in (0.25, 0.75):
                    x = (column + sx) * 64 / size - 32
                    y = (row + sy) * 64 / size - 32
                    outer = x*x + y*y < 29*29
                    tunnel = x*x / (15*15) + (y-6)**2 / (19*19) < 1
                    inner = x*x / (8*8) + (y-10)**2 / (14*14) < 1
                    color = (0, 0, 0, 0) if not outer else ((244, 252, 248, 255) if tunnel and not inner else (19, 124, 102, 255))
                    for i, value in enumerate(color):
                        channels[i] += value
            raw.extend(round(c / 4) for c in channels)
    header = struct.pack('>IIBBBBB', size, size, 8, 6, 0, 0, 0)
    return b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', header) + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b'')


def icons() -> Path:
    directory = ROOT / 'staging' / 'icons'
    directory.mkdir(parents=True, exist_ok=True)
    image = png(256)
    (directory / 'burrow.png').write_bytes(image)
    ico_header = struct.pack('<HHH', 0, 1, 1)
    ico_entry = struct.pack('<BBBBHHII', 0, 0, 0, 0, 1, 32, len(image), 22)
    (directory / 'burrow.ico').write_bytes(ico_header + ico_entry + image)
    chunks = []
    for size, kind in [(32, b'icp5'), (64, b'icp6'), (128, b'ic07'), (256, b'ic08'), (512, b'ic09')]:
        data = png(size)
        chunks.append(kind + struct.pack('>I', len(data) + 8) + data)
    body = b''.join(chunks)
    (directory / 'burrow.icns').write_bytes(b'icns' + struct.pack('>I', len(body) + 8) + body)
    return directory


def notices() -> None:
    metadata = json.loads((ROOT / 'dependency-metadata.json').read_text(encoding='utf-8'))
    sections = ['THIRD-PARTY SOFTWARE NOTICES\nGenerated from the release Cargo.lock and downloaded crate sources.\n']
    for package in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'])):
        if package['name'] == 'burrow':
            continue
        directory = Path(package['manifest_path']).parent
        sections.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\nDeclared license: {package.get('license') or 'See upstream'}\nUpstream: {package.get('repository') or 'https://crates.io/crates/' + package['name']}\n")
        license_files = [p for p in directory.iterdir() if p.is_file() and p.name.upper().startswith(('LICENSE', 'COPYING', 'COPYRIGHT', 'NOTICE', 'UNLICENSE'))]
        if package.get('license_file'):
            specific = directory / package['license_file']
            if specific.is_file() and specific not in license_files:
                license_files.append(specific)
        for path in sorted(license_files):
            sections.append(f"\n--- {path.name} ---\n{path.read_text(encoding='utf-8', errors='replace')}\n")
        if not license_files:
            # Some crate archives place notices in README or the source header.
            for name in ['README.md', 'README', 'src/lib.rs']:
                path = directory / name
                if path.is_file():
                    sections.append(f"\n--- {name} (notice-bearing source) ---\n{path.read_text(encoding='utf-8', errors='replace')}\n")
                    break
    (ROOT / 'THIRD_PARTY_NOTICES.txt').write_text('\n'.join(sections), encoding='utf-8')


def source_bundle() -> None:
    paths = subprocess.check_output(['git', 'ls-files', '-z']).decode().split('\0')
    paths = [p for p in paths if p]
    for extra in ['Cargo.lock', 'THIRD_PARTY_NOTICES.txt']:
        if extra not in paths:
            paths.append(extra)
    prefix = f'Burrow-{VERSION}-source'
    with zipfile.ZipFile(DIST / f'{prefix}.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
        for relative in sorted(paths):
            archive.write(ROOT / relative, f'{prefix}/{relative}')


def macos(architecture: str) -> None:
    if architecture not in ('apple-silicon', 'intel'):
        raise ValueError('Expected apple-silicon or intel')
    label = 'AppleSilicon' if architecture == 'apple-silicon' else 'Intel'
    stage = ROOT / 'staging' / f'macos-{architecture}'
    app = stage / 'Burrow.app' / 'Contents'
    (app / 'MacOS').mkdir(parents=True, exist_ok=True)
    (app / 'Resources').mkdir(exist_ok=True)
    shutil.copy2(ROOT / 'target/release/burrow', app / 'MacOS/burrow')
    shutil.copy2(icons() / 'burrow.icns', app / 'Resources/burrow.icns')
    for name in ['LICENSE', 'THIRD_PARTY_NOTICES.txt']:
        shutil.copy2(ROOT / name, app / 'Resources' / name)
    shutil.copy2(ROOT / 'docs/INSTALL.md', stage / 'READ ME FIRST.txt')
    info = {
        'CFBundleName': 'Burrow', 'CFBundleDisplayName': 'Burrow',
        'CFBundleIdentifier': 'io.github.noblespartan6.burrow',
        'CFBundleExecutable': 'burrow', 'CFBundlePackageType': 'APPL',
        'CFBundleVersion': VERSION, 'CFBundleShortVersionString': VERSION,
        'CFBundleIconFile': 'burrow.icns', 'LSMinimumSystemVersion': '12.0',
        'LSApplicationCategoryType': 'public.app-category.utilities',
        'NSHighResolutionCapable': True,
    }
    with (app / 'Info.plist').open('wb') as output:
        plistlib.dump(info, output)
    applications = stage / 'Applications'
    if not applications.is_symlink():
        applications.symlink_to('/Applications')
    run('codesign', '--force', '--deep', '--sign', '-', app.parent)
    run('codesign', '--verify', '--deep', '--strict', app.parent)
    output = DIST / f'Burrow-{VERSION}-macOS-{label}.dmg'
    run('hdiutil', 'create', '-volname', 'Burrow', '-srcfolder', stage, '-ov', '-fs', 'HFS+', '-format', 'UDZO', output)
    run('hdiutil', 'verify', output)


def windows() -> None:
    icons()
    compiler = Path(r'C:\Program Files (x86)\Inno Setup 6\ISCC.exe')
    if not compiler.is_file():
        found = shutil.which('ISCC.exe')
        if not found:
            raise RuntimeError('Inno Setup 6 is required to build the Windows installer')
        compiler = Path(found)
    run(compiler, f'/DAppVersion={VERSION}', ROOT / 'packaging/windows.iss')
    prefix = f'Burrow-{VERSION}-Windows-x64-portable'
    with zipfile.ZipFile(DIST / f'{prefix}.zip', 'w', zipfile.ZIP_DEFLATED) as archive:
        for source, name in [('target/release/burrow.exe', 'burrow.exe'), ('LICENSE', 'LICENSE.txt'),
                             ('THIRD_PARTY_NOTICES.txt', 'THIRD_PARTY_NOTICES.txt'), ('docs/INSTALL.md', 'READ ME FIRST.txt')]:
            archive.write(ROOT / source, f'{prefix}/{name}')


def checksums() -> None:
    lines = []
    for path in sorted(DIST.iterdir()):
        if path.is_file() and path.name != 'SHA256SUMS.txt':
            with path.open('rb') as source:
                digest = hashlib.file_digest(source, 'sha256').hexdigest()
            lines.append(f'{digest}  {path.name}')
    (DIST / 'SHA256SUMS.txt').write_text('\n'.join(lines) + '\n', encoding='utf-8')


if __name__ == '__main__':
    commands = {'notices': notices, 'source': source_bundle, 'windows': windows, 'checksums': checksums, 'icons': icons}
    try:
        command = sys.argv[1]
        if command == 'macos':
            macos(sys.argv[2])
        else:
            commands[command]()
    except (IndexError, KeyError):
        raise SystemExit('Usage: package.py notices|source|windows|checksums|icons|macos apple-silicon|intel')
