import unittest

from inilib import parse


SAMPLE = """
# top comment
[database]
host = localhost
port = 5432
; debug off
enabled = yes

[feature]
flag = true
count = 3
empty =
"""


class TestParse(unittest.TestCase):
    def test_sections_and_keys(self):
        cfg = parse(SAMPLE)
        self.assertTrue(cfg.has_section("database"))
        self.assertEqual(cfg.get("database", "host"), "localhost")
        self.assertEqual(cfg.get("database", "port"), "5432")
        self.assertEqual(cfg.get("feature", "flag"), "true")
        self.assertEqual(cfg.get("feature", "empty"), "")

    def test_missing(self):
        cfg = parse(SAMPLE)
        self.assertIsNone(cfg.get("missing", "x"))
        self.assertEqual(cfg.get("database", "nope", "fallback"), "fallback")
        self.assertFalse(cfg.has_section("nope"))

    def test_bool_int(self):
        cfg = parse(SAMPLE)
        self.assertTrue(cfg.get_bool("database", "enabled"))
        self.assertTrue(cfg.get_bool("feature", "flag"))
        self.assertFalse(cfg.get_bool("database", "absent"))
        self.assertEqual(cfg.get_int("database", "port"), 5432)
        self.assertEqual(cfg.get_int("feature", "count"), 3)
        self.assertEqual(cfg.get_int("feature", "absent", 9), 9)

    def test_comments_only(self):
        cfg = parse("# hi\n; there\n")
        self.assertFalse(cfg.has_section("database"))


if __name__ == "__main__":
    unittest.main()
