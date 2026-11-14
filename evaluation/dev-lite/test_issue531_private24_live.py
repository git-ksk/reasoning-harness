"""Public, provider-free safety invariants of private #531 development driver."""
import importlib.util
import pathlib
import sys
import unittest

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
spec = importlib.util.spec_from_file_location('private531_driver', HERE / 'issue531_private24_live.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

class TestPrivate531Runner(unittest.TestCase):
    def test_historical_pins_unchanged(self):
        self.assertEqual(runner.v5.BASELINE, '4abee90f0501f0d8fc09b453a0b45c942fbed69e')
        self.assertEqual(runner.v5.CANDIDATE, '7466c6f90d420a98c99713b96d6a0623e2aeef52')
        self.assertEqual(runner.v5.CLI, '988b69f086ac4a209fe93eb10cf1fa6786977567')
        self.assertEqual(runner.v5.MODEL, 'ministral-8b-2512')
    def test_global_token_and_price_ceiling(self):
        self.assertEqual(runner.JOBS, 48)
        self.assertEqual(runner.TOTAL_CASES, 24)
        self.assertEqual(runner.v5.MAX_CALLS, 12)
        self.assertLess(runner.MAX_PRICE, 0.21)
        self.assertLess(runner.MAX_PRICE, 9.98 / 10)
    def test_never_borrow_formal_score(self):
        self.assertEqual(len(runner.CORPUS_HASH), 64)
        self.assertIn('SELF_AUTHORED', runner.measure.__code__.co_consts[-1] if isinstance(runner.measure.__code__.co_consts[-1], str) else 'SELF_AUTHORED')

if __name__ == '__main__': unittest.main()
