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
    process = subprocess.Popen([str(root / 'target/release/burrow')], stdout=log, stderr=log)
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
            for key, title, name in [('1', 'Overview', '01-overview'), ('2', 'Clean up', '02-cleanup'), ('3', 'Disk explorer', '03-explorer'), ('4', 'About & help', '04-help')]:
                subprocess.check_call(['xdotool', 'key', '--clearmodifiers', f'ctrl+{key}'])
                for _ in range(50):
                    if title in output('xdotool', 'getwindowname', window): break
                    time.sleep(0.1)
                else: raise AssertionError(f'Navigation did not activate {title} at {size}')
                time.sleep(0.5)
                subprocess.check_call(['scrot', '-u', str(artifacts / f'{name}-{size}.png')])
                assert process.poll() is None, f'App crashed on {title}'
        subprocess.check_call(['xdotool', 'windowsize', '--sync', window, '1060', '800'])
        time.sleep(0.5)
        for x, title in [(480, 'Clean up'), (606, 'Disk explorer'), (732, 'About & help'), (354, 'Overview')]:
            subprocess.check_call(['xdotool', 'mousemove', '--window', window, str(x), '42', 'click', '1'])
            for _ in range(50):
                if title in output('xdotool', 'getwindowname', window): break
                time.sleep(0.1)
            else: raise AssertionError(f'Pointer navigation did not activate {title}')
        subprocess.check_call(['xdotool', 'key', '--clearmodifiers', 'ctrl+plus', 'ctrl+plus'])
        time.sleep(0.5)
        subprocess.check_call(['scrot', '-u', str(artifacts / '05-overview-large-text.png')])
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
        (artifacts / 'SMOKE-TEST.txt').write_text('PASS: native launch; four-page keyboard and pointer navigation; 1060x800 and 720x560; text zoom. No cleanup performed.\n')
    finally:
        process.terminate()
        try: process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill(); process.wait()
