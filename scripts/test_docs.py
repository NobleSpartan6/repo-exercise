"""Keep the nontechnical entry points linked to real docs and the tested release baseline."""
from pathlib import Path
import re
import unittest
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]


class DocumentationTests(unittest.TestCase):
    def test_relative_document_links_resolve_inside_the_repository(self):
        documents = [ROOT / name for name in ("README.md", "SECURITY.md", "CHANGELOG.md")]
        documents += list((ROOT / "docs").glob("*.md"))
        for document in documents:
            for href in re.findall(r"\[[^\]\n]+\]\(([^\s)]+)\)", document.read_text(encoding="utf-8")):
                parsed = urlsplit(href)
                if parsed.scheme or parsed.netloc or not parsed.path:
                    continue
                target = (document.parent / unquote(parsed.path)).resolve()
                with self.subTest(document=document.name, link=href):
                    self.assertTrue(target.is_relative_to(ROOT), "Link escapes the repository")
                    self.assertTrue(target.exists(), f"Missing document target: {target}")

    def test_download_guides_match_the_actual_installer_upgrade_baseline(self):
        workflow = (ROOT / ".github/workflows/burrow.yml").read_text(encoding="utf-8")
        match = re.search(r"gh release download (v[\w.\-]+)", workflow)
        self.assertIsNotNone(match, "The explicit published upgrade baseline is missing")
        tag = match.group(1)
        for name in ("README.md", "docs/INSTALL.md", "docs/FEATURES.md"):
            text = (ROOT / name).read_text(encoding="utf-8")
            with self.subTest(document=name):
                self.assertIn(f"/releases/tag/{tag}", text)
                self.assertIn("development", text.lower())

    def test_readme_keeps_installation_separate_from_developer_commands(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        self.assertIn("docs/INSTALL.md", readme)
        self.assertIn("docs/DEVELOPMENT.md", readme)
        self.assertNotIn("cargo build", readme)
        for suffix in ("macOS-AppleSilicon.dmg", "macOS-Intel.dmg", "Windows-x64-Setup.exe"):
            self.assertIn(suffix, readme)


if __name__ == "__main__":
    unittest.main()
