#!/usr/bin/env python3
"""Exercise the exact installed binary; GPU captures, no desktop capture permission."""
from pathlib import Path
import os
import struct
import subprocess
import sys
from package import VERSION

PAGES = ('overview', 'cleanup', 'explorer', 'help')
SIZES = ('desktop', 'compact', 'large-text')


def validate_png(path: Path) -> None:
    data = path.read_bytes()
    if len(data) < 8000 or data[:8] != b'\x89PNG\r\n\x1a\n' or data[12:16] != b'IHDR':
        raise ValueError(f'Missing or invalid PNG evidence: {path.name}')
    width, height = struct.unpack('>II', data[16:24])
    if not (300 <= width <= 8192 and 250 <= height <= 8192):
        raise ValueError(f'Invalid GPU screenshot dimensions: {width}x{height}')


def verify(binary: Path, logs: Path) -> None:
    logs.mkdir(parents=True, exist_ok=True)
    # Every run must produce fresh evidence. Old PNGs cannot turn a failed run green.
    shots = logs / 'captures'
    shots.mkdir(exist_ok=False)
    env = {**os.environ, 'RUST_BACKTRACE': '1', 'BURROW_SMOKE_OUTPUT': str(shots.resolve())}
    env.pop('EFRAME_SCREENSHOT_TO', None)
    version = subprocess.run([str(binary), '--version'], capture_output=True, text=True, timeout=20, check=True)
    if version.stdout.strip() != f'Burrow {VERSION}':
        raise RuntimeError(f'Installed binary version mismatch: {version.stdout!r}')
    try:
        result = subprocess.run([str(binary), '--smoke-test'], capture_output=True, text=True, timeout=90, env=env)
    except subprocess.TimeoutExpired as error:
        (logs / 'native-launch.txt').write_text(f'FAIL: native window timed out: {error}', encoding='utf-8')
        raise
    (logs / 'native-launch.txt').write_text(version.stdout + result.stdout + result.stderr, encoding='utf-8')
    if result.returncode or f'BURROW_UI_SMOKE_OK {VERSION}' not in result.stdout or 'BURROW_UI_SMOKE_FAILED' in result.stderr:
        raise RuntimeError(f'Native UI did not complete on {sys.platform}: {result.returncode}\n{result.stdout}\n{result.stderr}')
    for size in SIZES:
        for page in PAGES:
            if f'BURROW_CAPTURE_OK {page} {size} ' not in result.stdout:
                raise RuntimeError(f'Missing native capture acknowledgement: {page}, {size}')
            validate_png(shots / f'native-{page}-{size}.png')
    (logs / 'NATIVE-QA.txt').write_text(
        f'PASS: {VERSION}: installed release binary; four pages at desktop, compact, and 150% text scale; '
        '12 nonblank native GPU captures; normal shutdown. No cache scan or cleanup performed.\n'
        'Screenshot checks do not certify every GPU, assistive technology, or physical device.\n', encoding='utf-8')
    print(f'PASS: {binary.name}: 12 native GPU view captures and clean shutdown.')


if __name__ == '__main__':
    root = Path(__file__).resolve().parent.parent
    binary = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else root / 'target' / 'release' / ('burrow.exe' if os.name == 'nt' else 'burrow')
    verify(binary, root / 'artifacts')
