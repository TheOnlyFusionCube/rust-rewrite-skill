import unittest

from pipeline.transform import aggregate, filter_rows, parse_line, run_pipeline


class TestTransform(unittest.TestCase):
    def test_parse(self):
        self.assertEqual(parse_line("a,3"), ("a", 3))
        self.assertIsNone(parse_line("# comment"))
        self.assertIsNone(parse_line(""))
        self.assertIsNone(parse_line("nope"))
        self.assertIsNone(parse_line("x,abc"))

    def test_filter(self):
        rows = [("a", 1), ("b", 0), ("c", -2), ("d", 5)]
        self.assertEqual(filter_rows(rows), [("a", 1), ("d", 5)])

    def test_aggregate(self):
        rows = [("b", 1), ("a", 2), ("b", 3)]
        self.assertEqual(aggregate(rows), [("a", 2), ("b", 4)])

    def test_pipeline(self):
        lines = [
            "# c",
            "apple,3",
            "banana,0",
            "apple,2",
            "cherry,-1",
            "banana,5",
        ]
        self.assertEqual(run_pipeline(lines), [("apple", 5), ("banana", 5)])


if __name__ == "__main__":
    unittest.main()
