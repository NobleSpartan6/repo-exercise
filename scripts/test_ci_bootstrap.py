"""Guard against unrelated runner package feeds blocking native release QA."""
from pathlib import Path
import re
import shutil
import subprocess
import unittest

ROOT = Path(__file__).resolve().parent.parent


class LinuxBootstrapTests(unittest.TestCase):
    def step(self):
        workflow = (ROOT / '.github/workflows/burrow.yml').read_text()
        section = workflow.split('      - name: Install Linux graphics and smoke-test utilities\n', 1)[1]
        return section.split('      - name:', 1)[0]

    def test_only_distribution_sources_are_used_for_both_apt_commands(self):
        step = self.step()
        self.assertIn("if: runner.os == 'Linux'", step)
        self.assertIn('ubuntu_sources=/etc/apt/sources.list.d/ubuntu.sources', step)
        self.assertIn('test -s "$ubuntu_sources"', step)
        self.assertIn('Dir::Etc::sourcelist=$ubuntu_sources', step)
        self.assertIn('Dir::Etc::sourceparts=-', step)
        commands = re.findall(r'^.*sudo apt-get .+$', step, re.M)
        self.assertEqual(len(commands), 2)
        for command in commands:
            self.assertIn('"${apt_options[@]}"', command)
            self.assertIn('timeout 240s', command)
        self.assertIn('Acquire::Retries=3', step)
        self.assertIn('APT::Update::Error-Mode=any', step)

    def test_package_verification_and_failure_gates_are_not_bypassed(self):
        step = self.step().lower()
        for forbidden in ('--allow-unauthenticated', 'allowinsecurerepositories',
                          'trusted=yes', 'check-valid-until=false', '|| true',
                          'continue-on-error', 'sudo rm', 'sudo sed'):
            self.assertNotIn(forbidden, step)
        for package in ('libxkbcommon-dev', 'libegl1-mesa-dev', 'xvfb', 'xdotool',
                        'scrot', 'fonts-dejavu-core'):
            self.assertIn(package, step)

    def test_bootstrap_is_valid_bash(self):
        bash = shutil.which('bash')
        if bash is None:
            self.skipTest('Bash syntax is checked on Linux CI')
        script = self.step().split('        run: |\n', 1)[1]
        script = '\n'.join(line[10:] for line in script.splitlines())
        subprocess.run([bash, '-n'], input=script, text=True, check=True, timeout=10)


if __name__ == '__main__':
    unittest.main()
