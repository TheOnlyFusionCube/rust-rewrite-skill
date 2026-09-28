import tempfile
import unittest
from pathlib import Path
from io import StringIO
from unittest import mock

from wcapp.cli import main, format_counts


class TestCli(unittest.TestCase):
    def test_format(self):
        self.assertEqual(format_counts(1, 2, 3, "x"), "1 2 3 x")

    def test_main_prints(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "a.txt"
            p.write_text("hi there\n", encoding="utf-8")
            buf = StringIO()
            with mock.patch("sys.stdout", buf):
                code = main([str(p)])
            self.assertEqual(code, 0)
            self.assertEqual(buf.getvalue().strip(), f"1 2 9 {p}")

    def test_usage(self):
        buf = StringIO()
        with mock.patch("sys.stderr", buf):
            code = main([])
        self.assertEqual(code, 2)


if __name__ == "__main__":
    unittest.main()
