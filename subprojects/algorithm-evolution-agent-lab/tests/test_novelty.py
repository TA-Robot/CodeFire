import unittest

from evoagent.models import CandidateAlgorithm, ExperimentPlan, Hypothesis
from evoagent.novelty import AlgorithmFamilyReference, NoveltyReviewer


# cf-atom: TEST-novelty-reviewer-flags-known-family-overlap
class NoveltyReviewerTests(unittest.TestCase):
    def test_novelty_reviewer_flags_known_family_overlap(self):
        candidate = make_candidate(
            "stacked calibrated ensemble",
            "Use bagging with calibrated threshold aggregation for tabular classification.",
        )
        known = [
            AlgorithmFamilyReference(
                family="bagging ensemble",
                keywords=("bagging", "ensemble", "aggregation"),
                source="internal-literature-notes",
            )
        ]

        findings = NoveltyReviewer().review(candidate=candidate, known_families=known, min_overlap=2)

        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].family, "bagging ensemble")
        self.assertEqual(findings[0].severity, "review_required")

    def test_novelty_reviewer_allows_low_overlap(self):
        candidate = make_candidate("stability filter", "Rank features by perturbation agreement.")
        known = [AlgorithmFamilyReference("boosting", ("weak learner", "residual"), "notes")]

        findings = NoveltyReviewer().review(candidate=candidate, known_families=known)

        self.assertEqual(findings, [])


def make_candidate(title: str, rationale: str) -> CandidateAlgorithm:
    hypothesis = Hypothesis(title, rationale, expected_gain=0.02, novelty=0.5)
    plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", estimated_cost=0.1)
    return CandidateAlgorithm("candidate", hypothesis, plan)


if __name__ == "__main__":
    unittest.main()
