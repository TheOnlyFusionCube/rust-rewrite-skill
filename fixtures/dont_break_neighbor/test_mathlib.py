import unittest
from mathlib import add, mul


class TestMathlib(unittest.TestCase):
    def test_add(self):
        self.assertEqual(add(2, 3), 5)
        self.assertEqual(add(0, 0), 0)

    def test_mul(self):
        self.assertEqual(mul(2, 3), 6)
        self.assertEqual(mul(4, 5), 20)


if __name__ == "__main__":
    unittest.main()
