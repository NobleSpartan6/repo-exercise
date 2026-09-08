#!/usr/bin/env python3
"""Launch the actual release binary, render all four pages, and exit. Read-only."""
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
    print(f'PASS: {binary.name}: native release window rendered all four pages and closed normally.')


if __name__ == '__main__':
    root = Path(__file__).resolve().parent.parent
    binary = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else root / 'target' / 'release' / ('burrow.exe' if os.name == 'nt' else 'burrow')
    verify(binary, root / 'artifacts')
