"""Regression checks for actual failures found while preparing the native release."""
from pathlib import Path
import struct
import tempfile
import tomllib
import unittest
from native_smoke import validate_png

ROOT = Path(__file__).resolve().parent.parent


class RendererLockTests(unittest.TestCase):
    def test_dx12_allocator_uses_the_renderers_windows_types(self):
        lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
        def dep(name):
            packages = [p for p in lock['package'] if p['name'] == name]
            self.assertEqual(len(packages), 1)
            return [d for d in packages[0].get('dependencies', []) if d.startswith('windows ')]
        # gpu-allocator has a wide Windows dependency range. Cargo once reused the
        # older trash dependency, producing incompatible Direct3D12 types.
        self.assertEqual(dep('gpu-allocator'), dep('wgpu-hal'))
        self.assertEqual(dep('wgpu-hal'), ['windows 0.58.0'])

    def test_unsupported_eframe_capture_is_not_enabled(self):
        manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
        self.assertNotIn('__screenshot', manifest['dependencies']['eframe']['features'])
        smoke = (ROOT / 'scripts/native_smoke.py').read_text()
        self.assertNotIn("'EFRAME_SCREENSHOT_TO':", smoke)
        self.assertIn('BURROW_SMOKE_OUTPUT', smoke)
        self.assertIn('ViewportCommand::Screenshot', (ROOT / 'src/qa.rs').read_text())

    def test_platform_backends_and_dependencies_are_explicit(self):
        manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
        windows = manifest['target']['cfg(target_os = "windows")']['dependencies']['wgpu']
        mac = manifest['target']['cfg(target_os = "macos")']['dependencies']['wgpu']
        self.assertEqual(windows['version'], '=27.0.1')
        self.assertIn('dx12', windows['features'])
        self.assertIn('metal', mac['features'])
        self.assertFalse(windows['default-features'])
        self.assertFalse(mac['default-features'])

    def test_release_is_gated_by_real_install_and_recovery_checks(self):
        workflow = (ROOT / '.github/workflows/burrow.yml').read_text()
        self.assertIn('needs: [resolve, build]', workflow)
        self.assertIn('engine::native_tests::native_trash_roundtrip', workflow)
        self.assertIn('python scripts/verify_install.py', workflow)
        self.assertIn('python3 scripts/verify_install.py', workflow)
        self.assertNotIn('continue-on-error:', workflow)
        self.assertIn('cargo test --locked --all-targets', workflow)


class NativeEvidenceTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.path = Path(temp.name) / 'evidence.png'

    def test_missing_capture_fails(self):
        with self.assertRaises(FileNotFoundError):
            validate_png(self.path)

    def test_small_or_non_png_capture_fails(self):
        for data in [b'\x89PNG\r\n\x1a\n', b'not a screenshot' * 1000]:
            self.path.write_bytes(data)
            with self.assertRaises(ValueError):
                validate_png(self.path)

    def test_invalid_capture_dimensions_fail(self):
        self.path.write_bytes(b'\x89PNG\r\n\x1a\n' + b'\x00\x00\x00\x0dIHDR' + struct.pack('>II', 1, 1) + bytes(9000))
        with self.assertRaises(ValueError):
            validate_png(self.path)


if __name__ == '__main__':
    unittest.main()
