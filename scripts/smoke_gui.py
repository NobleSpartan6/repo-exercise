#!/usr/bin/env python3
"""Linux native-window pointer/keyboard/resize/zoom QA. Never cleans real files."""
from pathlib import Path
import os
import subprocess
import time

root = Path(__file__).resolve().parent.parent
artifacts = root / 'artifacts'
artifacts.mkdir(exist_ok=True)


def output(*args):
    return subprocess.check_output(args, text=True).strip()


with (artifacts / 'gui.log').open('w') as log:
    process = subprocess.Popen([str(root / 'target/release/burrow'), '--interaction-test'], stdout=log, stderr=log)
    try:
        window = None
        for _ in range(100):
            if process.poll() is not None:
                raise RuntimeError((artifacts / 'gui.log').read_text(errors='replace'))
            try:
                window = output('xdotool', 'search', '--onlyvisible', '--name', 'Burrow').splitlines()[0]
                break
            except (subprocess.CalledProcessError, IndexError): time.sleep(0.1)
        if not window: raise RuntimeError('No visible native window was created')
        subprocess.check_call(['xdotool', 'windowfocus', '--sync', window])
        for width, height, size in [('1060', '800', 'desktop'), ('720', '560', 'compact')]:
            subprocess.check_call(['xdotool', 'windowsize', '--sync', window, width, height])
            time.sleep(0.5)
            for key, title, name in [('1', 'Clean', '01-cleanup'), ('2', 'Apps', '02-apps'), ('3', 'Optimize', '03-optimize'), ('4', 'Analyze', '04-explorer'), ('5', 'Status', '05-status'), ('6', 'About & help', '06-help')]:
                subprocess.check_call(['xdotool', 'key', '--clearmodifiers', f'ctrl+{key}'])
                for _ in range(50):
                    if title in output('xdotool', 'getwindowname', window): break
                    time.sleep(0.1)
                else: raise AssertionError(f'Navigation did not activate {title} at {size}')
                time.sleep(2.3 if title == "Status" else 0.5)
                subprocess.check_call(['scrot', '-u', str(artifacts / f'{name}-{size}.png')])
                assert process.poll() is None, f'App crashed on {title}'
        subprocess.check_call(['xdotool', 'windowsize', '--sync', window, '1060', '800'])
        time.sleep(0.5)
        for title in ['Clean', 'Apps', 'Optimize', 'Analyze', 'About & help', 'Status']:
            # Coordinates are emitted by the actual controls, not guessed pixel positions.
            matches = [line.split('\t') for line in (artifacts / 'gui.log').read_text(errors='replace').splitlines() if line.startswith(f'BURROW_HITBOX\t{title}\t')]
            if not matches: raise AssertionError(f'No native hitbox for {title}')
            _, _, x, y = matches[-1]
            subprocess.check_call(['xdotool', 'mousemove', '--window', window, x, y, 'click', '1'])
            for _ in range(50):
                if title in output('xdotool', 'getwindowname', window): break
                time.sleep(0.1)
            else: raise AssertionError(f'Pointer navigation did not activate {title}')
        # A real secondary native viewport must close without taking the main app down.
        def hitbox(label):
            rows = [line.split('\t') for line in (artifacts / 'gui.log').read_text(errors='replace').splitlines() if line.startswith(f'BURROW_HITBOX\t{label}\t')]
            if not rows: raise AssertionError(f'Missing actual control: {label}')
            return rows[-1][2:4]
        x, y = hitbox('Mini monitor')
        subprocess.check_call(['xdotool', 'mousemove', '--window', window, x, y, 'click', '1'])
        mini = None
        for _ in range(60):
            try:
                mini = output('xdotool', 'search', '--onlyvisible', '--pid', str(process.pid), '--name', 'Mini monitor$').splitlines()[0]
                break
            except (subprocess.CalledProcessError, IndexError): time.sleep(0.1)
        # X11's legacy WM_NAME can transliterate the em dash in the title.
        # Match its ASCII suffix and the app's PID, then require a separate window.
        if not mini or mini == window:
            try:
                ids = output('xdotool', 'search', '--onlyvisible', '--pid', str(process.pid)).splitlines()
                names = {wid: output('xdotool', 'getwindowname', wid) for wid in ids}
            except subprocess.CalledProcessError:
                names = {}
            raise AssertionError(f'Mini monitor did not create its own window; visible app windows: {names}')
        assert 'Mini monitor' in output('xdotool', 'getwindowname', mini)
        subprocess.check_call(['xdotool', 'windowfocus', '--sync', mini])
        time.sleep(2.3)
        subprocess.check_call(['scrot', '-u', str(artifacts / '08-mini-monitor.png')])
        x, y = hitbox('Close mini monitor')
        subprocess.check_call(['xdotool', 'mousemove', '--window', mini, x, y, 'click', '1'])
        for _ in range(60):
            if process.poll() is not None: raise AssertionError('Closing mini monitor closed the app')
            try: output('xdotool', 'getwindowname', mini)
            except subprocess.CalledProcessError: break
            time.sleep(0.1)
        else: raise AssertionError('Mini monitor did not close')
        assert 'Status' in output('xdotool', 'getwindowname', window)
        subprocess.check_call(['xdotool', 'windowfocus', '--sync', window])
        subprocess.check_call(['xdotool', 'key', '--clearmodifiers', 'ctrl+plus', 'ctrl+plus'])
        time.sleep(0.5)
        subprocess.check_call(['scrot', '-u', str(artifacts / '07-status-large-text.png')])
        assert process.poll() is None
        # Warm idle measurements from /proc; not a cross-platform benchmark.
        def sample():
            fields = Path(f'/proc/{process.pid}/stat').read_text().rsplit(')', 1)[1].split()
            return int(fields[11]) + int(fields[12]), int(fields[21]) * os.sysconf('SC_PAGE_SIZE')
        subprocess.check_call(['xdotool', 'mousemove', '1150', '850'])
        time.sleep(2)
        a, _ = sample(); start = time.monotonic(); time.sleep(5); b, rss = sample()
        cpu = (b-a) / os.sysconf('SC_CLK_TCK') / (time.monotonic()-start) * 100
        (artifacts / 'IDLE-METRICS.txt').write_text(f'Linux CI warm idle, 5s sample: {cpu:.2f}% of one CPU core; RSS {rss/1048576:.1f} MiB.\nNot a Mac/Windows hardware claim.\n')
        (artifacts / 'SMOKE-TEST.txt').write_text('PASS: native launch; six-page keyboard and pointer navigation; 1060x800 and 720x560; text zoom; secondary monitor open/render/close keeps main window alive. No cleanup performed.\n')
    finally:
        process.terminate()
        try: process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill(); process.wait()
