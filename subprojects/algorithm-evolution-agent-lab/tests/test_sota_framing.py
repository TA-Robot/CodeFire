import unittest

from evoagent.claims import ClaimType
from evoagent.safety import ClaimEvidenceSummary, ClaimReadiness
from evoagent.sota_framing import SotaPursuitFramer


# cf-atom: TEST-sota-pursuit-framer-keeps-weak-evidence-as-hypothesis
class SotaPursuitFramerTests(unittest.TestCase):
    def test_sota_pursuit_framer_keeps_weak_evidence_as_hypothesis(self):
        frame = SotaPursuitFramer().frame(
            requested_claim=ClaimType.EXTERNAL_SOTA,
            evidence_tokens=("benchmark_definition", "external_baseline"),
            evidence_summary=ClaimEvidenceSummary(valid_runs=1, baseline_comparisons=1),
        )

        self.assertEqual(frame.label, "hypothesis")
        self.assertFalse(frame.external_claim_allowed)
        self.assertIn("reproduction_bundle", frame.missing_evidence)

    def test_sota_pursuit_framer_allows_only_mature_external_claim(self):
        frame = SotaPursuitFramer().frame(
            requested_claim=ClaimType.EXTERNAL_SOTA,
            evidence_tokens=(
                "benchmark_definition",
                "external_baseline",
                "reproduction_bundle",
                "statistical_confidence",
                "reviewer_clearance",
                "limitations",
            ),
            evidence_summary=ClaimEvidenceSummary(
                valid_runs=3,
                baseline_comparisons=2,
                repeated_runs=2,
                benchmark_protocol_satisfied=True,
                ablations_included=True,
                limitations_included=True,
                reproduction_bundle_ready=True,
                external_baseline_comparison=True,
            ),
        )

        self.assertEqual(frame.label, "evidence_backed_claim")
        self.assertEqual(frame.readiness, ClaimReadiness.EXTERNAL_SOTA_CLAIM)
        self.assertTrue(frame.external_claim_allowed)


if __name__ == "__main__":
    unittest.main()
