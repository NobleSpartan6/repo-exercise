#!/usr/bin/env python3
"""Linux/Xvfb native-window smoke test. Never selects or removes real files."""
from pathlib import Path
import subprocess
import time

root = Path(__file__).resolve().parent.parent
artifacts = root / 'artifacts'
artifacts.mkdir(exist_ok=True)


def output(*args):
    return subprocess.check_output(args, text=True).strip()


with (artifacts / 'gui.log').open('w') as log:
    process = subprocess.Popen([str(root / 'target/release/burrow')], stdout=log, stderr=log)
    try:
        window = None
        for _ in range(100):
            if process.poll() is not None:
                details = (artifacts / 'gui.log').read_text(errors='replace')
                raise RuntimeError(f'Native app exited before creating a window:\n{details}')
            try:
                window = output('xdotool', 'search', '--onlyvisible', '--name', 'Burrow').splitlines()[0]
                break
            except (subprocess.CalledProcessError, IndexError):
                time.sleep(0.1)
        if not window:
            raise RuntimeError('No visible native window was created')
        subprocess.check_call(['xdotool', 'windowfocus', '--sync', window])
        for key, title, filename in [('1', 'Overview', '01-overview.png'), ('2', 'Clean up', '02-cleanup.png'),
                                     ('3', 'Disk explorer', '03-explorer.png'), ('4', 'About & help', '04-help.png')]:
            subprocess.check_call(['xdotool', 'key', '--clearmodifiers', f'ctrl+{key}'])
            for _ in range(50):
                if title in output('xdotool', 'getwindowname', window):
                    break
                time.sleep(0.1)
            else:
                raise AssertionError(f'Navigation did not activate {title}')
            time.sleep(0.5)
            subprocess.check_call(['scrot', '-u', str(artifacts / filename)])
        assert process.poll() is None, 'App crashed during navigation'
        (artifacts / 'SMOKE-TEST.txt').write_text('PASS: native window opened; all four navigation shortcuts changed the active page; no cleanup performed.\n')
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
