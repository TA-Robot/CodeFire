import unittest

from evoagent.claims import ClaimDisciplineClassifier, ClaimType


# cf-atom: TEST-claim-discipline-distinguishes-sota-from-internal-score
class ClaimDisciplineClassifierTests(unittest.TestCase):
    def test_claim_discipline_distinguishes_sota_from_internal_score(self):
        classifier = ClaimDisciplineClassifier()

        internal = classifier.classify(ClaimType.INTERNAL_SCORE, ("score_formula",))
        sota = classifier.classify(
            ClaimType.EXTERNAL_SOTA,
            ("benchmark_definition", "external_baseline", "reproduction_bundle"),
        )

        self.assertTrue(internal.allowed)
        self.assertFalse(sota.allowed)
        self.assertIn("statistical_confidence", sota.missing_evidence)
        self.assertIn("reviewer_clearance", sota.missing_evidence)

    def test_claim_discipline_allows_benchmark_improvement_with_local_evidence(self):
        result = ClaimDisciplineClassifier().classify(
            ClaimType.BENCHMARK_IMPROVEMENT,
            ("benchmark_definition", "baseline_comparison", "valid_run"),
        )

        self.assertTrue(result.allowed)


if __name__ == "__main__":
    unittest.main()
