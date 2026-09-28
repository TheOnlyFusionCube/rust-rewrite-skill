import tempfile
import unittest
from pathlib import Path

from wcapp.counts import count_file, count_text, count_words, count_lines, count_bytes


class TestCounts(unittest.TestCase):
    def test_empty(self):
        self.assertEqual(count_text(""), (0, 0, 0))

    def test_simple(self):
        text = "hello world\n"
        self.assertEqual(count_lines(text), 1)
        self.assertEqual(count_words(text), 2)
        self.assertEqual(count_bytes(text.encode("utf-8")), 12)
        self.assertEqual(count_text(text), (1, 2, 12))

    def test_no_trailing_newline(self):
        text = "a b c"
        self.assertEqual(count_lines(text), 1)
        self.assertEqual(count_words(text), 3)

    def test_multiline(self):
        text = "one\ntwo three\nfour\n"
        self.assertEqual(count_text(text), (3, 4, 19))

    def test_file(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "f.txt"
            p.write_text("alpha beta\ngamma\n", encoding="utf-8")
            self.assertEqual(count_file(str(p)), (2, 3, 17))


if __name__ == "__main__":
    unittest.main()
