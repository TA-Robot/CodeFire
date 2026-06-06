import unittest

from evoagent.learning_curve import LearningCurveAnalyzer


# cf-atom: TEST-learning-curve-analyzer-detects-plateau-and-instability
class LearningCurveAnalyzerTests(unittest.TestCase):
    def test_learning_curve_analyzer_detects_plateau_and_instability(self):
        analyzer = LearningCurveAnalyzer()

        plateau = analyzer.analyze((0.80, 0.82, 0.821, 0.822), min_delta=0.005, instability_tolerance=1.0)
        unstable = analyzer.analyze((0.80, 0.90, 0.78, 0.91), min_delta=0.005, instability_tolerance=0.1)

        self.assertEqual(plateau.trend, "plateau")
        self.assertTrue(plateau.plateau)
        self.assertEqual(unstable.trend, "unstable")
        self.assertTrue(unstable.unstable)


if __name__ == "__main__":
    unittest.main()
