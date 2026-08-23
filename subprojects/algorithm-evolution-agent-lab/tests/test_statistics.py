import unittest

from evoagent.statistics import ConfidenceIntervalEstimator


# cf-atom: TEST-confidence-interval-estimator-summarizes-repeated-runs
class ConfidenceIntervalEstimatorTests(unittest.TestCase):
    def test_confidence_interval_estimator_summarizes_repeated_runs(self):
        interval = ConfidenceIntervalEstimator().estimate([0.80, 0.82, 0.84])

        self.assertAlmostEqual(interval.mean, 0.82)
        self.assertEqual(interval.n, 3)
        self.assertLess(interval.lower, interval.mean)
        self.assertGreater(interval.upper, interval.mean)
        self.assertEqual(interval.confidence_level, 0.95)

    def test_confidence_interval_estimator_single_value_has_no_width(self):
        interval = ConfidenceIntervalEstimator().estimate([0.8])

        self.assertEqual(interval.lower, 0.8)
        self.assertEqual(interval.upper, 0.8)

    def test_confidence_interval_estimator_rejects_non_finite_values(self):
        for label, value in (("nan", float("nan")), ("inf", float("inf")), ("-inf", float("-inf"))):
            with self.subTest(value=label):
                with self.assertRaisesRegex(ValueError, "values must be finite"):
                    ConfidenceIntervalEstimator().estimate([0.80, 0.82, value])


if __name__ == "__main__":
    unittest.main()
