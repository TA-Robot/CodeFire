import unittest

from evoagent.external_validity import BenchmarkDomain, ExternalValidityEvaluator
from evoagent.models import ExperimentPlan, ExperimentResult, Hypothesis


# cf-atom: TEST-external-validity-requires-transfer-across-domains
class ExternalValidityEvaluatorTests(unittest.TestCase):
    def test_external_validity_requires_transfer_across_domains(self):
        results = [
            result("toy-tabular", metric_value=0.84, baseline_value=0.80),
            result("synthetic-sequence", metric_value=0.71, baseline_value=0.70),
        ]
        domains = [BenchmarkDomain("toy-tabular", "tabular"), BenchmarkDomain("synthetic-sequence", "sequence")]

        report = ExternalValidityEvaluator().evaluate(results=results, domains=domains, min_domains=2)

        self.assertEqual(report.improved_benchmarks, ("synthetic-sequence", "toy-tabular"))
        self.assertEqual(report.improved_domains, ("sequence", "tabular"))
        self.assertTrue(report.transferable)

    def test_external_validity_rejects_single_domain_gain(self):
        results = [
            result("toy-tabular-a", metric_value=0.84, baseline_value=0.80),
            result("toy-tabular-b", metric_value=0.83, baseline_value=0.80),
        ]
        domains = [BenchmarkDomain("toy-tabular-a", "tabular"), BenchmarkDomain("toy-tabular-b", "tabular")]

        report = ExternalValidityEvaluator().evaluate(results=results, domains=domains, min_domains=2)

        self.assertFalse(report.transferable)


def result(benchmark: str, *, metric_value: float, baseline_value: float) -> ExperimentResult:
    hypothesis = Hypothesis("candidate", "rationale", expected_gain=0.01, novelty=0.3)
    plan = ExperimentPlan(hypothesis, benchmark, "baseline", "accuracy", estimated_cost=0.1)
    return ExperimentResult(plan, metric_value=metric_value, baseline_value=baseline_value, confidence=0.8)


if __name__ == "__main__":
    unittest.main()
