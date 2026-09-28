import unittest

from inilib.table import Config


class TestTable(unittest.TestCase):
    def test_set_get(self):
        c = Config()
        c.set("a", "k", "v")
        self.assertEqual(c.get("a", "k"), "v")
        self.assertTrue(c.has_section("a"))

    def test_bool_variants(self):
        c = Config()
        for v in ("true", "YES", "1", "On"):
            c.set("s", "b", v)
            self.assertTrue(c.get_bool("s", "b"))
        for v in ("false", "NO", "0", "off"):
            c.set("s", "b", v)
            self.assertFalse(c.get_bool("s", "b"))

    def test_int_bad(self):
        c = Config()
        c.set("s", "n", "abc")
        self.assertEqual(c.get_int("s", "n", 7), 7)


if __name__ == "__main__":
    unittest.main()
