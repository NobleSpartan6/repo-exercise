"""Keep native mini-window discovery strict across X11 title encodings."""
import ast
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class WindowDiscoveryTests(unittest.TestCase):
    def test_pid_and_title_search_requires_both_not_xdotools_default_or(self):
        source = (ROOT / "scripts/smoke_gui.py").read_text(encoding="utf-8")
        searches = []
        for node in ast.walk(ast.parse(source)):
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "output":
                literals = [arg.value for arg in node.args if isinstance(arg, ast.Constant)]
                if literals[:2] == ["xdotool", "search"] and "--pid" in literals and "--name" in literals:
                    searches.append(literals)
        self.assertEqual(len(searches), 1, "Expected an explicit mini-window discovery query")
        self.assertIn("--all", searches[0], "xdotool defaults to OR, which can match the main window")
        self.assertIn("--onlyvisible", searches[0])
        self.assertIn("Mini monitor$", searches[0], "Avoid X11's lossy legacy encoding of the title prefix")
        self.assertIn("mini == window", source, "The mini monitor must remain a separate native window")


if __name__ == "__main__":
    unittest.main()
