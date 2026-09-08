#!/usr/bin/env python3
"""Launch the exact release binary and capture its native-renderer output."""
from pathlib import Path
import os
import subprocess
import sys


def verify(binary: Path, logs: Path) -> None:
    logs.mkdir(parents=True, exist_ok=True)
    version = subprocess.run([str(binary), '--version'], capture_output=True, text=True, timeout=20, check=True)
    result = subprocess.run([str(binary), '--smoke-test'], capture_output=True, text=True, timeout=45,
                            env={**os.environ, 'RUST_BACKTRACE': '1'})
    (logs / 'native-launch.txt').write_text(version.stdout + result.stdout + result.stderr, encoding='utf-8')
    if result.returncode or 'BURROW_UI_SMOKE_OK' not in result.stdout:
        raise RuntimeError(f'Native UI did not complete on {sys.platform}: {result.returncode}\n{result.stdout}\n{result.stderr}')
    # eframe's pinned diagnostic feature captures the actual GPU surface and exits.
    # No desktop recording permission, mocked HTML or screenshots used as UI.
    shot = (logs / 'native-overview.png').resolve()
    captured = subprocess.run([str(binary)], capture_output=True, text=True, timeout=45,
                             env={**os.environ, 'EFRAME_SCREENSHOT_TO': str(shot), 'RUST_BACKTRACE': '1'})
    (logs / 'native-screenshot.txt').write_text(captured.stdout + captured.stderr, encoding='utf-8')
    if captured.returncode or not shot.is_file() or shot.stat().st_size < 8000:
        raise RuntimeError(f'Native renderer screenshot failed: {captured.returncode}\n{captured.stdout}\n{captured.stderr}')
    if shot.read_bytes()[:8] != b'\x89PNG\r\n\x1a\n':
        raise RuntimeError('Native screenshot is not PNG data')
    print(f'PASS: {binary.name}: native release window rendered all pages, closed normally, and produced a GPU screenshot.')


if __name__ == '__main__':
    root = Path(__file__).resolve().parent.parent
    binary = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else root / 'target' / 'release' / ('burrow.exe' if os.name == 'nt' else 'burrow')
    verify(binary, root / 'artifacts')
