import unittest

from evoagent.models import CandidateAlgorithm, ExperimentPlan, ExperimentResult, Hypothesis
from evoagent.scoring import score_candidate


# cf-atom: TEST-candidate-scoring-uses-quality-and-cost
class CandidateScoringTests(unittest.TestCase):
    def test_candidate_scoring_uses_quality_and_cost(self):
        hypothesis = Hypothesis(
            title="better optimizer",
            rationale="candidate improves convergence",
            expected_gain=0.05,
            novelty=0.40,
        )
        cheap_plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", estimated_cost=0.5)
        expensive_plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", estimated_cost=5.0)
        cheap = CandidateAlgorithm("cheap", hypothesis, cheap_plan)
        expensive = CandidateAlgorithm("expensive", hypothesis, expensive_plan)

        self.assertGreater(score_candidate(cheap), score_candidate(expensive))

    def test_observed_result_replaces_expected_gain(self):
        hypothesis = Hypothesis("idea", "reason", expected_gain=0.01, novelty=0.20)
        plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", estimated_cost=1.0)
        result = ExperimentResult(plan, metric_value=0.80, baseline_value=0.70, confidence=0.90)
        candidate = CandidateAlgorithm("observed", hypothesis, plan, result=result)

        self.assertGreater(score_candidate(candidate), 0.0)


if __name__ == "__main__":
    unittest.main()
