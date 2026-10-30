import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import sys


sys.dont_write_bytecode = True


spec = importlib.util.spec_from_file_location(
    "compare_walltime", Path(__file__).with_name("compare-walltime.py"))
walltime = importlib.util.module_from_spec(spec)
spec.loader.exec_module(walltime)


class WalltimeComparison(unittest.TestCase):
    def test_criterion_samples(self):
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp) / "benchmark/new"
            directory.mkdir(parents=True)
            (directory / "benchmark.json").write_text(json.dumps({"full_id": "parse/new"}))
            path = directory / "sample.json"
            # Normalize each sample by its own iteration count. Ignore the minimum outlier.
            path.write_text(json.dumps({"iters": [1, 2, 4], "times": [1, 200, 404]}))
            self.assertEqual(walltime.read_samples(Path(temp)), {"parse/new": 100})

            path.write_text(json.dumps({"iters": [1, 0], "times": [1, 200]}))
            with self.assertRaises(ValueError):
                walltime.read_samples(Path(temp))

            path.write_text(json.dumps({"iters": [1], "times": [1, 200]}))
            with self.assertRaises(ValueError):
                walltime.read_samples(Path(temp))

    def test_process_medians_and_suite_ratios(self):
        base = [{"small": 100, "large": 10_000}] * walltime.PAIRS
        head = [{"small": 50, "large": 20_000}] * walltime.PAIRS
        result = walltime.compare(base, head)
        self.assertAlmostEqual(result["geometric_ratio"], 1)
        rows = {row["name"]: row for row in result["rows"]}
        self.assertEqual(rows["small"]["assessment"], "Improvement candidate")
        self.assertEqual(rows["large"]["assessment"], "Regression candidate")

        # One unusually fast process must not replace the typical base time.
        base = [{"visit": value} for value in [1, 100, 100, 100, 100]]
        head = [{"visit": 100}] * walltime.PAIRS
        result = walltime.compare(base, head)
        self.assertEqual(result["rows"][0]["ratio"], 1)
        self.assertEqual(result["rows"][0]["assessment"], "Inconclusive")

    def test_uncertainty_and_threshold(self):
        base = [{"parse": 100}] * walltime.PAIRS
        head = [{"parse": 100.5}] * walltime.PAIRS
        self.assertEqual(walltime.compare(base, head)["rows"][0]["assessment"], "Inconclusive")

        head = [{"parse": value} for value in [90, 95, 105, 110, 115]]
        result = walltime.compare(base, head)
        self.assertGreater(result["rows"][0]["ratio"], 1.01)
        self.assertEqual(result["rows"][0]["assessment"], "Inconclusive")
        self.assertEqual(result, walltime.compare(base, head))

    def test_missing_results(self):
        runs = [{"parse": 100}] * walltime.PAIRS
        with self.assertRaises(ValueError):
            walltime.compare(runs[:-1], runs)
        with self.assertRaises(ValueError):
            walltime.compare(runs, [{"visit": 100}] * walltime.PAIRS)
        with self.assertRaises(ValueError):
            walltime.compare([{}] * walltime.PAIRS, [{}] * walltime.PAIRS)
        with tempfile.TemporaryDirectory() as temp:
            with self.assertRaises(ValueError):
                walltime.read_samples(Path(temp))


if __name__ == "__main__":
    unittest.main()
