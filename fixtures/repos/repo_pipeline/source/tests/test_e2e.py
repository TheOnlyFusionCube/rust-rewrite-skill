import tempfile
import unittest
from pathlib import Path

from pipeline.cli import main


SAMPLE = """# sample
apple,3
banana,0
apple,2
cherry,-1
banana,5
date,4
bogus
cherry,1
"""

EXPECTED = "apple,5\nbanana,5\ncherry,1\ndate,4\n"


class TestE2E(unittest.TestCase):
    def test_cli(self):
        with tempfile.TemporaryDirectory() as td:
            inp = Path(td) / "in.csv"
            outp = Path(td) / "out.csv"
            inp.write_text(SAMPLE, encoding="utf-8")
            code = main([str(inp), str(outp)])
            self.assertEqual(code, 0)
            self.assertEqual(outp.read_text(encoding="utf-8"), EXPECTED)


if __name__ == "__main__":
    unittest.main()
