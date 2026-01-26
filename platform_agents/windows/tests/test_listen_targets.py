import os
import sys
import unittest

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from wxauto_agent import normalize_listen_targets


class ListenTargetsTests(unittest.TestCase):
    def test_normalizes_and_dedupes_targets(self):
        raw = [
            {"name": "  Team A ", "kind": "group"},
            {"name": "Team A", "kind": "direct"},
            {"name": "", "kind": "group"},
            {"name": "Team B", "kind": "unknown"},
        ]
        out = normalize_listen_targets(raw)
        self.assertEqual(
            out,
            [
                {"name": "Team A", "kind": "group"},
                {"name": "Team B", "kind": "unknown"},
            ],
        )

    def test_defaults_unknown_kind(self):
        raw = [{"name": "Team C", "kind": "weird"}]
        out = normalize_listen_targets(raw)
        self.assertEqual(out[0]["kind"], "unknown")


if __name__ == "__main__":
    unittest.main()
