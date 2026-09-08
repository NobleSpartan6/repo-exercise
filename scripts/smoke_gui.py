#!/usr/bin/env python3
"""Linux/Xvfb native-window smoke test. Never selects or removes real files.

Navigation/resize checks are not a visual sign-off; inspect the resulting PNGs.
"""
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
        for width, height, size in [('1060', '760', 'desktop'), ('860', '620', 'compact')]:
            subprocess.check_call(['xdotool', 'windowsize', '--sync', window, width, height])
            time.sleep(0.5)
            for key, title, name in [('1', 'Overview', '01-overview'), ('2', 'Clean up', '02-cleanup'),
                                     ('3', 'Disk explorer', '03-explorer'), ('4', 'About & help', '04-help')]:
                subprocess.check_call(['xdotool', 'key', '--clearmodifiers', f'ctrl+{key}'])
                for _ in range(50):
                    if title in output('xdotool', 'getwindowname', window):
                        break
                    time.sleep(0.1)
                else:
                    raise AssertionError(f'Navigation did not activate {title} at {size} size')
                time.sleep(0.5)
                subprocess.check_call(['scrot', '-u', str(artifacts / f'{name}-{size}.png')])
                assert process.poll() is None, f'App crashed on {title} at {size} size'
        (artifacts / 'SMOKE-TEST.txt').write_text(
            'PASS: native window opened; all four page shortcuts changed the active title '
            'at 1060x760 and 860x620; app remained running; no cleanup performed.\n'
            'Manual image review, high-DPI checks, real Mac/Windows testing, '
            'and native Trash restoration remain separate checks.\n')
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
