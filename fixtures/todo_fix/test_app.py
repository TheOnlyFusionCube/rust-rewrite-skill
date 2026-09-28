import unittest
from app import answer


class TestApp(unittest.TestCase):
    def test_answer(self):
        self.assertEqual(answer(), 42)


if __name__ == "__main__":
    unittest.main()
